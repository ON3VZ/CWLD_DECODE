//! The connection to a DX cluster, on its own thread.
//!
//! The client connects, waits for the login prompt, sends the callsign, tells the node what to
//! send (skimmer spots on or off) and then hands every spot over as an event. When the node does
//! not answer or the connection drops it tries again, and in `rotate` mode it moves on to the next
//! node of the list. It only ever reads spots: nothing is spotted or announced by CWLD.

use crate::spot::{detect_software, init_commands, parse_spot, Software, Spot};
use crate::telnet::Telnet;
use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub name: String,
    pub host: String,
    pub port: u16,
}

impl Node {
    pub fn new(name: &str, host: &str, port: u16) -> Self {
        Node { name: name.into(), host: host.into(), port }
    }
}

/// What the client tells the outside world.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Connecting { node: String },
    /// Logged in. `software` is what the welcome text showed.
    Online { node: String, software: Software },
    Offline { node: String, reason: String },
    Spot(Spot),
    /// Any other line of the node (welcome text, replies to commands, announcements).
    Line(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub nodes: Vec<Node>,
    /// Index of the node to start with.
    pub first: usize,
    /// On trouble go on to the next node (the choice "automatic"). Otherwise the same node is retried.
    pub rotate: bool,
    /// The callsign of the operator, used to log in.
    pub call: String,
    pub skimmer: bool,
    pub connect_timeout: Duration,
    /// How long the node may be silent after the callsign before the commands are sent anyway.
    pub settle: Duration,
}

impl Config {
    pub fn new(nodes: Vec<Node>, call: &str) -> Self {
        Config { nodes, first: 0, rotate: true, call: call.trim().to_ascii_uppercase(), skimmer: true, connect_timeout: Duration::from_secs(6), settle: Duration::from_millis(1500) }
    }
}

pub struct Client {
    stop: Arc<AtomicBool>,
    cmd: Sender<String>,
    events: Receiver<Event>,
}

impl Client {
    pub fn start(cfg: Config) -> Client {
        let stop = Arc::new(AtomicBool::new(false));
        let (cmd, cmd_rx) = channel();
        let (tx, events) = channel();
        let flag = stop.clone();
        std::thread::spawn(move || run(cfg, flag, cmd_rx, tx));
        Client { stop, cmd, events }
    }

    /// A line typed by the operator (a cluster command). Sent once the login is done.
    pub fn send(&self, line: &str) {
        let _ = self.cmd.send(line.to_string());
    }

    /// Everything that happened since the last call.
    pub fn events(&self) -> Vec<Event> {
        self.events.try_iter().collect()
    }

    /// Asks the thread to end. It does not wait: the thread is gone within a few seconds.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.stop();
    }
}

fn sleep_checked(stop: &AtomicBool, total: Duration) {
    let end = Instant::now() + total;
    while !stop.load(Ordering::SeqCst) && Instant::now() < end {
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn run(cfg: Config, stop: Arc<AtomicBool>, cmds: Receiver<String>, tx: Sender<Event>) {
    let n = cfg.nodes.len();
    if n == 0 {
        let _ = tx.send(Event::Offline { node: String::new(), reason: "no cluster nodes in the list".into() });
        return;
    }
    let mut idx = cfg.first.min(n - 1);
    let mut fails = 0u32;
    while !stop.load(Ordering::SeqCst) {
        let node = cfg.nodes[idx].clone();
        if tx.send(Event::Connecting { node: node.name.clone() }).is_err() {
            return;
        }
        let started = Instant::now();
        let res = session(&cfg, &node, &stop, &cmds, &tx);
        if stop.load(Ordering::SeqCst) {
            return;
        }
        let reason = match res {
            Ok(()) => "the node closed the connection".to_string(),
            Err(e) => e,
        };
        if tx.send(Event::Offline { node: node.name.clone(), reason }).is_err() {
            return;
        }
        if started.elapsed() >= Duration::from_secs(30) {
            fails = 0;
        }
        fails += 1;
        let wait = if cfg.rotate {
            idx = (idx + 1) % n;
            // A short pause between nodes, a longer one after a whole round without luck.
            if fails as usize % n == 0 { Duration::from_secs(20) } else { Duration::from_secs(2) }
        } else {
            Duration::from_secs((2u64 << (fails - 1).min(4)).min(30))
        };
        sleep_checked(&stop, wait);
    }
}

fn connect(node: &Node, timeout: Duration) -> Result<TcpStream, String> {
    let addrs = (node.host.as_str(), node.port).to_socket_addrs().map_err(|e| format!("unknown host {}: {e}", node.host))?;
    let mut last = format!("no address for {}", node.host);
    for a in addrs {
        match TcpStream::connect_timeout(&a, timeout) {
            Ok(s) => return Ok(s),
            Err(e) => last = e.to_string(),
        }
    }
    Err(last)
}

/// Removes control characters (bell, escape sequences' first byte and so on) from a line.
fn clean(line: &[u8]) -> String {
    let s = String::from_utf8_lossy(line);
    s.chars().filter(|c| !c.is_control() || *c == '\t').collect::<String>().trim().to_string()
}

fn write_line(s: &mut TcpStream, line: &str) -> Result<(), String> {
    let safe: String = line.chars().filter(|c| !c.is_control()).take(200).collect();
    s.write_all(format!("{safe}\r\n").as_bytes()).map_err(|e| e.to_string())
}

fn session(cfg: &Config, node: &Node, stop: &AtomicBool, cmds: &Receiver<String>, tx: &Sender<Event>) -> Result<(), String> {
    let mut s = connect(node, cfg.connect_timeout)?;
    let _ = s.set_nodelay(true);
    s.set_read_timeout(Some(Duration::from_millis(200))).map_err(|e| e.to_string())?;
    let mut tel = Telnet::new();
    let (mut pending, mut welcome) = (Vec::<u8>::new(), String::new());
    let (mut login_sent, mut online) = (false, false);
    let (start, mut last_rx, mut last_tx) = (Instant::now(), Instant::now(), Instant::now());
    let (mut got_any, mut login_at) = (false, Instant::now());
    let mut queue: VecDeque<(Instant, String)> = VecDeque::new();
    let mut buf = [0u8; 2048];
    loop {
        if stop.load(Ordering::SeqCst) {
            return Ok(());
        }
        match s.read(&mut buf) {
            Ok(0) => return Err("connection closed".into()),
            Ok(n) => {
                let (mut text, mut reply) = (Vec::new(), Vec::new());
                tel.feed(&buf[..n], &mut text, &mut reply);
                if !reply.is_empty() {
                    s.write_all(&reply).map_err(|e| e.to_string())?;
                }
                if !text.is_empty() {
                    got_any = true;
                    last_rx = Instant::now();
                    pending.extend_from_slice(&text);
                }
            }
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(e) => return Err(e.to_string()),
        }
        // Complete lines.
        while let Some(pos) = pending.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = pending.drain(..=pos).collect();
            let text = clean(&line);
            if text.is_empty() {
                continue;
            }
            if welcome.len() < 4096 {
                welcome.push_str(&text);
                welcome.push('\n');
            }
            let ev = match parse_spot(&text) {
                Some(sp) => Event::Spot(sp),
                None => Event::Line(text.chars().take(300).collect()),
            };
            if tx.send(ev).is_err() {
                return Ok(());
            }
        }
        if pending.len() > 8192 {
            pending.clear();
        }
        // Login: when a prompt is on the screen, or the node has been quiet for a moment.
        if !login_sent {
            let tail = String::from_utf8_lossy(&pending).to_ascii_lowercase();
            let prompt = tail.contains("call") || tail.contains("login") || tail.trim_end().ends_with(':') || tail.trim_end().ends_with('>');
            let quiet = last_rx.elapsed();
            if got_any && ((prompt && quiet >= Duration::from_millis(250)) || quiet >= Duration::from_millis(1500)) {
                write_line(&mut s, &cfg.call)?;
                last_tx = Instant::now();
                login_sent = true;
                login_at = Instant::now();
                welcome.push_str(&tail);
                pending.clear();
            } else if !got_any && start.elapsed() > Duration::from_secs(10) {
                return Err("the node sent no welcome text".into());
            }
        }
        // After the login: wait until the node has finished its greeting, then set the filters.
        if login_sent && !online {
            let calm = last_rx.elapsed() >= cfg.settle && login_at.elapsed() >= cfg.settle;
            if calm || login_at.elapsed() >= Duration::from_secs(5) {
                online = true;
                let software = detect_software(&welcome);
                if tx.send(Event::Online { node: node.name.clone(), software }).is_err() {
                    return Ok(());
                }
                let mut due = Instant::now();
                for c in init_commands(software, cfg.skimmer) {
                    queue.push_back((due, c));
                    due += Duration::from_millis(400);
                }
            }
        }
        while queue.front().map(|(t, _)| *t <= Instant::now()).unwrap_or(false) {
            let (_, c) = queue.pop_front().unwrap();
            write_line(&mut s, &c)?;
            last_tx = Instant::now();
        }
        // Lines typed by the operator.
        while let Ok(c) = cmds.try_recv() {
            if online && !c.trim().is_empty() {
                write_line(&mut s, &c)?;
                last_tx = Instant::now();
            }
        }
        // Many nodes drop a client that never says anything.
        if online && last_tx.elapsed() > Duration::from_secs(240) {
            s.write_all(b"\r\n").map_err(|e| e.to_string())?;
            last_tx = Instant::now();
        }
        if online && last_rx.elapsed() > Duration::from_secs(1200) {
            return Err("no data for 20 minutes".into());
        }
    }
}

/// The result of trying one node.
#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    pub ok: bool,
    /// Milliseconds until the node accepted the connection.
    pub millis: u64,
    pub software: Software,
    pub error: String,
}

/// Connects, reads the welcome text and disconnects. It does not log in.
pub fn probe(node: &Node, timeout: Duration) -> Probe {
    let t0 = Instant::now();
    let mut s = match connect(node, timeout) {
        Ok(s) => s,
        Err(e) => return Probe { ok: false, millis: 0, software: Software::Unknown, error: e },
    };
    let millis = t0.elapsed().as_millis() as u64;
    let _ = s.set_read_timeout(Some(Duration::from_millis(250)));
    let mut tel = Telnet::new();
    let (mut text, mut reply) = (Vec::new(), Vec::new());
    let mut buf = [0u8; 1024];
    let end = Instant::now() + timeout.min(Duration::from_secs(4));
    while Instant::now() < end {
        match s.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                tel.feed(&buf[..n], &mut text, &mut reply);
                if !reply.is_empty() {
                    let _ = s.write_all(&reply);
                    reply.clear();
                }
                // A prompt or a few lines are enough to know the node is alive.
                if text.len() > 40 {
                    break;
                }
            }
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                if !text.is_empty() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    if text.is_empty() {
        return Probe { ok: false, millis, software: Software::Unknown, error: "connected, but no welcome text".into() };
    }
    Probe { ok: true, millis, software: detect_software(&String::from_utf8_lossy(&text)), error: String::new() }
}

/// Tries all nodes at the same time. The answers are in the order of `nodes`.
pub fn probe_all(nodes: &[Node], timeout: Duration) -> Vec<Probe> {
    std::thread::scope(|sc| {
        let hs: Vec<_> = nodes.iter().map(|n| sc.spawn(move || probe(n, timeout))).collect();
        hs.into_iter().map(|h| h.join().unwrap_or(Probe { ok: false, millis: 0, software: Software::Unknown, error: "test failed".into() })).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;

    /// Reads one line the way a Telnet server does: the answers to option offers are dropped.
    fn read_clean(r: &mut BufReader<TcpStream>) -> Option<String> {
        let mut raw = Vec::new();
        if r.read_until(b'\n', &mut raw).ok()? == 0 {
            return None;
        }
        let mut out = Vec::new();
        let mut i = 0;
        while i < raw.len() {
            if raw[i] == 255 && i + 2 < raw.len() {
                i += 3;
            } else {
                out.push(raw[i]);
                i += 1;
            }
        }
        Some(String::from_utf8_lossy(&out).trim().to_string())
    }

    /// A node that behaves like a small cluster: prompt, welcome, answers to commands, spots.
    fn fake_node(software_line: &'static str, spots: Vec<&'static str>) -> (u16, std::thread::JoinHandle<Vec<String>>) {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let h = std::thread::spawn(move || {
            let (mut c, _) = l.accept().unwrap();
            c.set_read_timeout(Some(Duration::from_secs(8))).unwrap();
            c.write_all(&[255, 253, 1]).unwrap(); // IAC DO ECHO, must be declined
            c.write_all(b"login: ").unwrap();
            let mut r = BufReader::new(c.try_clone().unwrap());
            let mut got = Vec::new();
            got.push(read_clean(&mut r).unwrap());
            c.write_all(format!("\r\nHello {}, {software_line}\r\n", got[0]).as_bytes()).unwrap();
            for sp in spots {
                c.write_all(format!("{sp}\r\n").as_bytes()).unwrap();
            }
            // Collect the commands the client sends.
            while let Some(t) = read_clean(&mut r) {
                if t.is_empty() {
                    continue;
                }
                got.push(t.clone());
                if t == "bye" {
                    break;
                }
            }
            got
        });
        (port, h)
    }

    fn wait_for<F: FnMut(&Event) -> bool>(c: &Client, secs: u64, mut pred: F) -> Vec<Event> {
        let end = Instant::now() + Duration::from_secs(secs);
        let mut all = Vec::new();
        while Instant::now() < end {
            for e in c.events() {
                let hit = pred(&e);
                all.push(e);
                if hit {
                    return all;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        all
    }

    fn quick(nodes: Vec<Node>) -> Config {
        Config { settle: Duration::from_millis(300), connect_timeout: Duration::from_secs(2), ..Config::new(nodes, "on3vz") }
    }

    #[test]
    fn logs_in_sets_the_filters_and_delivers_spots() {
        let (port, server) = fake_node(
            "this is CC Cluster version 4",
            vec!["DX de DL1ABC-#:   7030.1  PA3XYZ        CW 24 dB 22 WPM CQ             1234Z", "WWV de VE7CC <18>:   SFI=100"],
        );
        let c = Client::start(quick(vec![Node::new("fake", "127.0.0.1", port)]));
        let ev = wait_for(&c, 10, |e| matches!(e, Event::Online { .. }));
        assert!(ev.iter().any(|e| matches!(e, Event::Spot(s) if s.call == "PA3XYZ" && s.freq_hz == 7_030_100)), "{ev:?}");
        assert!(ev.iter().any(|e| matches!(e, Event::Line(l) if l.contains("SFI=100"))));
        match ev.last().unwrap() {
            Event::Online { node, software } => {
                assert_eq!(node, "fake");
                assert_eq!(*software, Software::Cc);
            }
            other => panic!("expected Online, got {other:?}"),
        }
        c.send("sh/dx 5");
        std::thread::sleep(Duration::from_millis(3000));
        c.send("bye");
        let got = server.join().unwrap();
        assert_eq!(got[0], "ON3VZ", "the callsign is sent in capitals");
        assert!(got.contains(&"set/skimmer".to_string()), "{got:?}");
        assert!(got.contains(&"sh/dx 5".to_string()), "{got:?}");
    }

    #[test]
    fn skimmer_off_sends_the_other_command() {
        let (port, server) = fake_node("DXSpider v1.57", vec![]);
        let mut cfg = quick(vec![Node::new("fake", "127.0.0.1", port)]);
        cfg.skimmer = false;
        let c = Client::start(cfg);
        wait_for(&c, 10, |e| matches!(e, Event::Online { .. }));
        std::thread::sleep(Duration::from_millis(800));
        c.send("bye");
        let got = server.join().unwrap();
        assert!(got.contains(&"unset/skimmer".to_string()), "{got:?}");
    }

    #[test]
    fn a_dead_node_makes_the_client_try_the_next_one() {
        // A port nobody listens on: bind, note the port, drop the listener.
        let dead = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let (port, server) = fake_node("DXSpider", vec![]);
        let c = Client::start(quick(vec![Node::new("dead", "127.0.0.1", dead), Node::new("alive", "127.0.0.1", port)]));
        let ev = wait_for(&c, 15, |e| matches!(e, Event::Online { .. }));
        assert!(ev.iter().any(|e| matches!(e, Event::Offline { node, .. } if node == "dead")), "{ev:?}");
        assert!(matches!(ev.last().unwrap(), Event::Online { node, .. } if node == "alive"));
        c.send("bye");
        let _ = server.join();
    }

    #[test]
    fn stop_ends_the_thread_and_an_empty_list_is_reported() {
        let c = Client::start(Config::new(vec![], "on3vz"));
        let ev = wait_for(&c, 3, |e| matches!(e, Event::Offline { .. }));
        assert!(matches!(ev.last(), Some(Event::Offline { .. })));
    }

    #[test]
    fn the_test_reports_a_live_and_a_dead_node() {
        let dead = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let h = std::thread::spawn(move || {
            let (mut c, _) = l.accept().unwrap();
            c.write_all(b"Welcome to the DXSpider node of TEST\r\nlogin: ").unwrap();
            std::thread::sleep(Duration::from_millis(600));
        });
        let r = probe_all(&[Node::new("a", "127.0.0.1", port), Node::new("b", "127.0.0.1", dead)], Duration::from_secs(3));
        assert!(r[0].ok && r[0].software == Software::Spider, "{:?}", r[0]);
        assert!(!r[1].ok && !r[1].error.is_empty());
        let _ = h.join();
    }
}
