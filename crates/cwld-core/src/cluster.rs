//! DX cluster spots for the band scope. CWLD only reads spots; it never spots or announces anything.
//!
//! The client of `cwld-cluster` runs on its own thread. Every tick of the application the core
//! collects what it delivered, keeps the spots of the last minutes (one per callsign and band) and
//! colors them by what the logbook knows: new, worked on another band, or worked on this band.

use crate::{AppCore, CoreError, CoreEvent};
use cwld_cluster::{Client, Config, Event, Node, Software, Spot};
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_SPOTS: usize = 800;
const MAX_LINES: usize = 200;
/// Two spots of one station closer than this are the same spot (the frequency drifts a little).
const SAME_SPOT_HZ: u64 = 300;
const WORKED_CACHE: Duration = Duration::from_secs(5);

fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Where the connection stands, for the status line of the interface.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ClusterStatus {
    /// `off`, `connecting`, `online` or `offline` (waiting for the next try).
    pub state: String,
    pub node: String,
    pub software: String,
    /// Why the last connection ended.
    pub reason: String,
}

/// One spot as the interface gets it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpotOut {
    pub call: String,
    pub freq_hz: u64,
    pub band: String,
    pub spotter: String,
    pub comment: String,
    pub snr: Option<i32>,
    pub wpm: Option<u16>,
    pub skimmer: bool,
    /// Seconds since 1970 when the newest report arrived.
    pub ts: i64,
    /// How many reports of this station came in.
    pub hits: u32,
    /// `new`, `other_band` or `worked`.
    pub status: String,
}

/// The result of trying one node (the button "Test the clusters").
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClusterProbe {
    pub node: String,
    pub host: String,
    pub port: u16,
    pub ok: bool,
    pub millis: u64,
    pub software: String,
    pub error: String,
}

struct Stored {
    spot: Spot,
    band: &'static str,
    ts: i64,
    hits: u32,
}

#[derive(Default)]
pub struct ClusterState {
    client: Option<Client>,
    status: ClusterStatus,
    spots: Vec<Stored>,
    lines: VecDeque<String>,
    /// (call, band) pairs of the logbook and when they were read.
    worked: Option<(Instant, HashSet<(String, String)>)>,
}

/// The first word of a cluster command must be one that only reads or changes our own filters.
pub fn command_allowed(line: &str) -> bool {
    const OK: [&str; 13] = ["sh", "show", "set", "unset", "clear", "accept", "acc", "reject", "rej", "help", "bye", "quit", "q"];
    let first = line.trim().to_ascii_lowercase();
    let first = first.split(|c: char| c == '/' || c.is_whitespace()).next().unwrap_or("");
    OK.contains(&first)
}

fn status_of(worked: &HashSet<(String, String)>, call: &str, band: &str) -> &'static str {
    if worked.contains(&(call.to_string(), band.to_string())) {
        "worked"
    } else if worked.iter().any(|(c, _)| c == call) {
        "other_band"
    } else {
        "new"
    }
}

fn out_of(s: &Stored, worked: &HashSet<(String, String)>) -> SpotOut {
    SpotOut {
        call: s.spot.call.clone(),
        freq_hz: s.spot.freq_hz,
        band: s.band.to_string(),
        spotter: s.spot.spotter.clone(),
        comment: s.spot.comment.clone(),
        snr: s.spot.snr,
        wpm: s.spot.wpm,
        skimmer: s.spot.skimmer,
        ts: s.ts,
        hits: s.hits,
        status: status_of(worked, &s.spot.call, s.band).to_string(),
    }
}

impl AppCore {
    /// The nodes of the settings, and which one to start with.
    fn cluster_nodes(&self) -> (Vec<Node>, usize, bool) {
        let s = self.settings.lock().unwrap();
        let nodes: Vec<Node> = s.cluster.nodes.iter().filter(|n| !n.host.trim().is_empty() && n.port > 0).map(|n| Node::new(&n.name, n.host.trim(), n.port)).collect();
        let key = s.cluster.node.trim().to_string();
        if key.is_empty() || key == "auto" {
            return (nodes, 0, true);
        }
        match nodes.iter().position(|n| format!("{}:{}", n.host, n.port) == key) {
            Some(i) => (nodes, i, false),
            None => (nodes, 0, true),
        }
    }

    /// The (call, band) pairs worked, depending on contest mode. Cached for a few seconds.
    fn cluster_worked(&self, st: &mut ClusterState, fresh: bool) -> HashSet<(String, String)> {
        if !fresh {
            if let Some((t, w)) = &st.worked {
                if t.elapsed() < WORKED_CACHE {
                    return w.clone();
                }
            }
        }
        let (contest_mode, contest, session) = {
            let s = self.settings.lock().unwrap();
            (s.contest_mode, s.contest.active.clone(), s.contest.session)
        };
        let pairs = {
            let lb = self.logbook.lock().unwrap();
            if contest_mode && !contest.is_empty() { lb.worked_pairs(Some((&contest, session))) } else { lb.worked_pairs(None) }
        };
        let pairs = pairs.unwrap_or_default();
        st.worked = Some((Instant::now(), pairs.clone()));
        pairs
    }

    /// Connects to the cluster. A running connection is stopped first.
    pub fn cluster_start(&self) -> Result<(), CoreError> {
        let (nodes, first, rotate) = self.cluster_nodes();
        let (call, skimmer) = {
            let s = self.settings.lock().unwrap();
            (s.station.calls.first().cloned().unwrap_or_default(), s.cluster.skimmer)
        };
        let call = call.trim().to_ascii_uppercase();
        if call.is_empty() {
            return Err(CoreError::Invalid("cluster_call_missing"));
        }
        if nodes.is_empty() {
            return Err(CoreError::Invalid("cluster_no_nodes"));
        }
        let mut cfg = Config::new(nodes, &call);
        cfg.first = first;
        cfg.rotate = rotate;
        cfg.skimmer = skimmer;
        let mut st = self.cluster.lock().unwrap();
        if let Some(old) = st.client.take() {
            old.stop();
        }
        st.spots.clear();
        st.status = ClusterStatus { state: "connecting".into(), ..Default::default() };
        st.client = Some(Client::start(cfg));
        Ok(())
    }

    pub fn cluster_stop(&self) -> ClusterStatus {
        let mut st = self.cluster.lock().unwrap();
        if let Some(c) = st.client.take() {
            c.stop();
        }
        st.spots.clear();
        st.status = ClusterStatus { state: "off".into(), ..Default::default() };
        st.status.clone()
    }

    pub fn cluster_status(&self) -> ClusterStatus {
        let st = self.cluster.lock().unwrap();
        if st.status.state.is_empty() { ClusterStatus { state: "off".into(), ..Default::default() } } else { st.status.clone() }
    }

    /// A line typed in the cluster window, sent as a command. Only commands that read or set
    /// filters go through; anything that would post a spot, a message or an announcement does not.
    pub fn cluster_send(&self, line: &str) -> Result<(), CoreError> {
        if !command_allowed(line) {
            return Err(CoreError::Invalid("cluster_command_not_allowed"));
        }
        match &self.cluster.lock().unwrap().client {
            Some(c) => {
                c.send(line);
                Ok(())
            }
            None => Err(CoreError::Invalid("cluster_not_connected")),
        }
    }

    /// All spots now, colored with the newest state of the logbook. The interface asks after logging a QSO.
    pub fn cluster_spots(&self) -> Vec<SpotOut> {
        let mut st = self.cluster.lock().unwrap();
        let worked = self.cluster_worked(&mut st, true);
        st.spots.iter().map(|s| out_of(s, &worked)).collect()
    }

    /// Tries all nodes of the list (connect and read the welcome text, no login). Takes a few seconds.
    pub fn cluster_probe(&self) -> Vec<ClusterProbe> {
        let nodes: Vec<_> = self.settings.lock().unwrap().cluster.nodes.clone();
        let list: Vec<Node> = nodes.iter().map(|n| Node::new(&n.name, n.host.trim(), n.port)).collect();
        let res = cwld_cluster::probe_all(&list, Duration::from_secs(6));
        nodes
            .into_iter()
            .zip(res)
            .map(|(n, p)| ClusterProbe { node: n.name, host: n.host, port: n.port, ok: p.ok, millis: p.millis, software: p.software.name().to_string(), error: p.error })
            .collect()
    }

    /// Collects what the cluster thread delivered and drops old spots. Called every tick.
    pub fn cluster_tick(&self) -> Vec<CoreEvent> {
        let mut st = self.cluster.lock().unwrap();
        let Some(client) = &st.client else {
            return Vec::new();
        };
        let incoming = client.events();
        let max_age = self.settings.lock().unwrap().cluster.max_age_minutes.clamp(1, cwld_types::CLUSTER_AGE_MAX) as i64 * 60;
        let now = now_secs();
        let mut events = Vec::new();
        let mut lines = Vec::new();
        let mut changed: Vec<usize> = Vec::new();
        let mut status_changed = false;
        for ev in incoming {
            match ev {
                Event::Connecting { node } => {
                    st.status = ClusterStatus { state: "connecting".into(), node, software: String::new(), reason: st.status.reason.clone() };
                    status_changed = true;
                }
                Event::Online { node, software } => {
                    let sw = if software == Software::Unknown { String::new() } else { software.name().to_string() };
                    st.status = ClusterStatus { state: "online".into(), node, software: sw, reason: String::new() };
                    status_changed = true;
                }
                Event::Offline { node, reason } => {
                    st.status = ClusterStatus { state: "offline".into(), node, software: String::new(), reason };
                    status_changed = true;
                }
                Event::Line(l) => {
                    st.lines.push_back(l.clone());
                    while st.lines.len() > MAX_LINES {
                        st.lines.pop_front();
                    }
                    lines.push(l);
                }
                Event::Spot(sp) => {
                    if !sp.is_cw() {
                        continue;
                    }
                    let band = cwld_log::band_of(sp.freq_hz);
                    if band.is_empty() {
                        continue;
                    }
                    let same = st.spots.iter().position(|s| s.spot.call == sp.call && s.band == band && s.spot.freq_hz.abs_diff(sp.freq_hz) <= SAME_SPOT_HZ);
                    match same {
                        Some(i) => {
                            let hits = st.spots[i].hits + 1;
                            st.spots[i] = Stored { spot: sp, band, ts: now, hits };
                            changed.push(i);
                        }
                        None => {
                            st.spots.push(Stored { spot: sp, band, ts: now, hits: 1 });
                            changed.push(st.spots.len() - 1);
                        }
                    }
                }
            }
        }
        // Old spots go; indexes of `changed` are looked up again after that, so work with calls.
        let changed_calls: Vec<(String, u64)> = changed.iter().filter_map(|&i| st.spots.get(i)).map(|s| (s.spot.call.clone(), s.spot.freq_hz)).collect();
        st.spots.retain(|s| now - s.ts <= max_age);
        if st.spots.len() > MAX_SPOTS {
            let extra = st.spots.len() - MAX_SPOTS;
            st.spots.drain(..extra);
        }
        if status_changed {
            events.push(CoreEvent::ClusterStatus { status: st.status.clone() });
        }
        if !lines.is_empty() {
            events.push(CoreEvent::ClusterLines { lines });
        }
        if !changed_calls.is_empty() {
            let worked = self.cluster_worked(&mut st, false);
            let want: HashMap<(String, u64), ()> = changed_calls.into_iter().map(|k| (k, ())).collect();
            let out: Vec<SpotOut> = st.spots.iter().filter(|s| want.contains_key(&(s.spot.call.clone(), s.spot.freq_hz))).map(|s| out_of(s, &worked)).collect();
            if !out.is_empty() {
                events.push(CoreEvent::ClusterSpots { spots: out });
            }
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cwld_log::Qso;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;

    #[test]
    fn colors_follow_the_logbook() {
        let mut w = HashSet::new();
        w.insert(("PA3ABC".to_string(), "40m".to_string()));
        assert_eq!(status_of(&w, "PA3ABC", "40m"), "worked");
        assert_eq!(status_of(&w, "PA3ABC", "20m"), "other_band");
        assert_eq!(status_of(&w, "DL1XYZ", "40m"), "new");
    }

    #[test]
    fn only_reading_commands_are_allowed() {
        for ok in ["sh/dx 10", "SHOW/DX", "set/skimmer", "unset/skimmer", "set dx filter", "acc/spots on 20-cw", "rej/spots 1 call ra", "clear/spots all", "help", "bye"] {
            assert!(command_allowed(ok), "{ok}");
        }
        for no in ["dx 14025.0 PA3ABC up", "DX 7030 K1ABC", "announce hello", "ann/full hi", "talk pa3abc hi", "wwv 100 5 0", "wx rain", "wcy", "", "  ", "dxx"] {
            assert!(!command_allowed(no), "{no}");
        }
        let core = AppCore::new(None);
        assert_eq!(core.cluster_send("dx 7030 PA3ABC"), Err(CoreError::Invalid("cluster_command_not_allowed")));
        assert_eq!(core.cluster_send("sh/dx"), Err(CoreError::Invalid("cluster_not_connected")));
    }

    #[test]
    fn start_needs_an_own_callsign() {
        let core = AppCore::new(None);
        assert_eq!(core.cluster_start(), Err(CoreError::Invalid("cluster_call_missing")));
        assert_eq!(core.cluster_status().state, "off");
    }

    /// A node on this machine that logs in, sends three spots and then waits.
    fn node(spots: Vec<String>) -> u16 {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let (mut c, _) = l.accept().unwrap();
            c.write_all(b"login: ").unwrap();
            let mut r = BufReader::new(c.try_clone().unwrap());
            let mut line = String::new();
            let _ = r.read_line(&mut line);
            c.write_all(b"\r\nWelcome, this is CC Cluster\r\n").unwrap();
            for s in spots {
                c.write_all(format!("{s}\r\n").as_bytes()).unwrap();
            }
            std::thread::sleep(Duration::from_secs(6));
        });
        port
    }

    fn collect(core: &AppCore, secs: u64, want: usize) -> Vec<SpotOut> {
        let end = Instant::now() + Duration::from_secs(secs);
        let mut got: Vec<SpotOut> = Vec::new();
        while Instant::now() < end && got.len() < want {
            for e in core.cluster_tick() {
                if let CoreEvent::ClusterSpots { spots } = e {
                    got.extend(spots);
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        got
    }

    #[test]
    fn spots_arrive_filtered_and_colored() {
        let port = node(vec![
            "DX de DL1ABC-#:   7030.1  PA3ABC        CW 24 dB 22 WPM CQ             1234Z".into(),
            "DX de DL1ABC-#:  14030.0  PA3ABC        CW 20 dB 25 WPM CQ             1234Z".into(),
            "DX de DL1ABC-#:  14074.0  JA1XYZ        FT8 -12 dB                     1234Z".into(),
            "DX de DL1ABC-#:  14031.0  DL1XYZ        CW 12 dB 20 WPM CQ             1234Z".into(),
        ]);
        let core = AppCore::new(None);
        let mut s = core.settings();
        s.station.calls = vec!["ON3VZ".into()];
        s.cluster.nodes = vec![cwld_types::ClusterNode { name: "local".into(), host: "127.0.0.1".into(), port }];
        core.update_settings(s).unwrap();
        core.log_add(Qso { call: "PA3ABC".into(), band: "40m".into(), freq_hz: 7_030_000, ..Default::default() }).unwrap();
        core.cluster_start().unwrap();
        let got = collect(&core, 12, 3);
        let by = |c: &str, b: &str| got.iter().find(|x| x.call == c && x.band == b).cloned();
        assert_eq!(by("PA3ABC", "40m").map(|x| x.status), Some("worked".into()), "{got:?}");
        assert_eq!(by("PA3ABC", "20m").map(|x| x.status), Some("other_band".into()));
        assert_eq!(by("DL1XYZ", "20m").map(|x| x.status), Some("new".into()));
        assert!(got.iter().all(|x| x.call != "JA1XYZ"), "FT8 is not CW");
        // The node counts as online a moment after the login, when its greeting is finished.
        let end = Instant::now() + Duration::from_secs(8);
        while core.cluster_status().state != "online" && Instant::now() < end {
            core.cluster_tick();
            std::thread::sleep(Duration::from_millis(100));
        }
        assert_eq!(core.cluster_status().state, "online");
        assert_eq!(core.cluster_spots().len(), 3);
        core.cluster_stop();
        assert_eq!(core.cluster_status().state, "off");
        assert!(core.cluster_spots().is_empty());
    }

    #[test]
    fn a_repeated_spot_updates_the_old_one() {
        let port = node(vec![
            "DX de DL1ABC-#:   7030.1  PA3ABC        CW 10 dB 22 WPM CQ             1234Z".into(),
            "DX de OK1XX-#:    7030.2  PA3ABC        CW 18 dB 22 WPM CQ             1235Z".into(),
        ]);
        let core = AppCore::new(None);
        let mut s = core.settings();
        s.station.calls = vec!["ON3VZ".into()];
        s.cluster.nodes = vec![cwld_types::ClusterNode { name: "local".into(), host: "127.0.0.1".into(), port }];
        core.update_settings(s).unwrap();
        core.cluster_start().unwrap();
        let _ = collect(&core, 12, 2);
        let all = core.cluster_spots();
        assert_eq!(all.len(), 1, "one station on one frequency is one spot: {all:?}");
        assert_eq!(all[0].hits, 2);
        assert_eq!(all[0].snr, Some(18), "the newest report");
        core.cluster_stop();
    }

    #[test]
    fn old_spots_expire() {
        let core = AppCore::new(None);
        {
            let mut st = core.cluster.lock().unwrap();
            let sp = cwld_cluster::parse_spot("DX de K1ABC: 7030.0 PA3ABC CW 1234Z").unwrap();
            st.spots.push(Stored { spot: sp, band: "40m", ts: now_secs() - 3600, hits: 1 });
            st.client = Some(Client::start(Config::new(vec![], "ON3VZ")));
        }
        let mut s = core.settings();
        s.cluster.max_age_minutes = 30;
        core.update_settings(s).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        core.cluster_tick();
        assert!(core.cluster.lock().unwrap().spots.is_empty());
    }
}
