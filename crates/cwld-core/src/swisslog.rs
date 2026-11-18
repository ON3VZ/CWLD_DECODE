//! Swisslog for Windows over the UDP message format of WSJT-X.
//!
//! Swisslog listens for WSJT-X on its "Main instance" (default 127.0.0.1:2237). CWLD takes the
//! place WSJT-X would have:
//! - Heartbeat (type 0) at start and every 15 seconds, so the UDP button of Swisslog turns green.
//! - Status (type 1) with frequency, mode and the callsign of the running QSO, so Swisslog fills
//!   its QSO entry window. Swisslog takes a callsign from 3 characters on; shorter is sent empty.
//! - QSO Logged (type 5) or Logged ADIF (type 12) when a QSO is logged, never both.
//! - Close (type 6) when the link is switched off or the settings change.
//!
//! Everything that comes back from Swisslog is read, counted and ignored. No message from another
//! program can make CWLD transmit, change a frequency or log anything.
//!
//! The wire format follows the public description of the WSJT-X messages (`NetworkMessage.hpp`):
//! big-endian, every message starts with the magic number and the schema number, then the message
//! type and the client id. Text is a 32-bit byte count and UTF-8 bytes. A date and time is a 64-bit
//! Julian day, 32-bit milliseconds since midnight and one byte for the time spec (1 is UTC).
//! The code here is written for CWLD, nothing is copied from WSJT-X.

use cwld_log::{adif_record, AdifOptions, Qso};
use cwld_types::settings::{LinkSettings, SwisslogMessage};
use cwld_types::{RadioMode, RadioStatus};
use serde::Serialize;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::{Duration, Instant};

pub const MAGIC: u32 = 0xadbc_cbda;
/// Schema 3 is what current WSJT-X versions use. The fields CWLD sends are the same in schema 2.
pub const SCHEMA: u32 = 3;
/// Seconds between two heartbeats (the "pulse" of WSJT-X).
pub const PULSE: Duration = Duration::from_secs(15);

pub const TYPE_HEARTBEAT: u32 = 0;
pub const TYPE_STATUS: u32 = 1;
pub const TYPE_QSO_LOGGED: u32 = 5;
pub const TYPE_CLOSE: u32 = 6;
pub const TYPE_LOGGED_ADIF: u32 = 12;

/// Julian day number of 1970-01-01.
const JD_UNIX_EPOCH: i64 = 2_440_588;

/// Builds one datagram. `put_*` append fields in the order of the message description.
struct Msg(Vec<u8>);

impl Msg {
    fn new(kind: u32, id: &str) -> Msg {
        let mut m = Msg(Vec::with_capacity(128));
        m.u32(MAGIC);
        m.u32(SCHEMA);
        m.u32(kind);
        m.text(id);
        m
    }
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }
    fn i64(&mut self, v: i64) {
        self.0.extend_from_slice(&v.to_be_bytes());
    }
    fn bool(&mut self, v: bool) {
        self.u8(v as u8);
    }
    /// Text as a byte count and UTF-8 bytes. An empty text has count 0 (not the "null" marker).
    fn text(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.0.extend_from_slice(s.as_bytes());
    }
    /// A UTC date and time from seconds since 1970.
    fn date_time(&mut self, ts: i64) {
        self.i64(JD_UNIX_EPOCH + ts.div_euclid(86_400));
        self.u32((ts.rem_euclid(86_400) * 1000) as u32);
        self.u8(1);
    }
}

/// The mode name Swisslog gets: `CW` for both CW modes, else the radio mode.
pub fn mode_name(m: RadioMode) -> &'static str {
    match m {
        RadioMode::Cw | RadioMode::CwReverse => "CW",
        RadioMode::Usb => "USB",
        RadioMode::Lsb => "LSB",
        RadioMode::Am => "AM",
        RadioMode::Fm => "FM",
        RadioMode::Fsk => "RTTY",
        RadioMode::Other => "CW",
    }
}

pub fn heartbeat(id: &str, version: &str) -> Vec<u8> {
    let mut m = Msg::new(TYPE_HEARTBEAT, id);
    m.u32(SCHEMA);
    m.text(version);
    m.text("");
    m.0
}

pub fn close(id: &str) -> Vec<u8> {
    Msg::new(TYPE_CLOSE, id).0
}

/// What the status message says. Two equal statuses are not sent twice.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct StatusInfo {
    pub dial_hz: u64,
    pub mode: String,
    /// Callsign of the running QSO, empty when there is none (or fewer than 3 characters).
    pub dx_call: String,
    pub report: String,
    pub transmitting: bool,
    pub de_call: String,
    pub de_grid: String,
    pub dx_grid: String,
}

pub fn status(id: &str, s: &StatusInfo) -> Vec<u8> {
    let mut m = Msg::new(TYPE_STATUS, id);
    m.u64(s.dial_hz);
    m.text(&s.mode);
    m.text(&s.dx_call);
    m.text(&s.report);
    m.text(&s.mode); // Tx mode
    m.bool(s.transmitting); // Tx enabled: CWLD only sends while the operator asks it to
    m.bool(s.transmitting);
    m.bool(false); // decoding (a time slot of FT8, not used for CW)
    m.u32(0); // Rx DF
    m.u32(0); // Tx DF
    m.text(&s.de_call);
    m.text(&s.de_grid);
    m.text(&s.dx_grid);
    m.bool(false); // Tx watchdog
    m.text(""); // sub-mode
    m.bool(false); // fast mode
    m.u8(0); // special operation mode: none
    m.u32(0); // frequency tolerance
    m.u32(0); // T/R period
    m.text(""); // configuration name
    m.text(""); // Tx message
    m.0
}

/// The callsign as Swisslog should get it: trimmed, upper case, and only from 3 characters on.
pub fn dx_call_for_status(call: &str) -> String {
    let c = call.trim().to_uppercase();
    if c.chars().count() >= 3 && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '/') {
        c
    } else {
        String::new()
    }
}

/// QSO Logged (type 5). The time on and off are the same: the logbook keeps one time per QSO.
pub fn qso_logged(id: &str, q: &Qso) -> Vec<u8> {
    let mut m = Msg::new(TYPE_QSO_LOGGED, id);
    m.date_time(q.ts);
    m.text(q.call.trim());
    m.text(""); // DX grid: the logbook does not keep the locator of the other station
    m.u64(q.freq_hz);
    m.text(if q.mode.trim().is_empty() { "CW" } else { q.mode.trim() });
    m.text(q.rst_sent.trim());
    m.text(q.rst_rcvd.trim());
    m.text(q.tx_pwr.trim());
    m.text(q.comment.trim());
    m.text(q.name.trim());
    m.date_time(q.ts);
    m.text(q.operator.trim());
    m.text(if q.station_call.trim().is_empty() { q.my_call.trim() } else { q.station_call.trim() });
    m.text(q.my_grid.trim());
    m.text(&exchange(q.nr_sent.to_string().as_str(), &q.extra_sent, q.nr_sent > 0));
    m.text(&exchange(&q.nr_rcvd, &q.extra_rcvd, !q.nr_rcvd.trim().is_empty()));
    m.text(""); // ADIF propagation mode
    m.0
}

fn exchange(nr: &str, extra: &str, has_nr: bool) -> String {
    let mut parts = Vec::new();
    if has_nr {
        parts.push(nr.trim().to_string());
    }
    if !extra.trim().is_empty() {
        parts.push(extra.trim().to_string());
    }
    parts.join(" ")
}

/// Logged ADIF (type 12): a small ADIF file with one record.
pub fn logged_adif(id: &str, q: &Qso, version: &str) -> Vec<u8> {
    let rec = adif_record(q, &AdifOptions { contest_id: String::new(), program_version: version.to_string(), now_ts: q.ts });
    let text = format!("<ADIF_VER:5>3.1.4 <PROGRAMID:4>CWLD <PROGRAMVERSION:{}>{} <EOH>\n{}", version.len(), version, rec);
    let mut m = Msg::new(TYPE_LOGGED_ADIF, id);
    m.text(&text);
    m.0
}

/// For the settings window: is the link working, and what came back.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct SwisslogStatus {
    pub active: bool,
    /// `host:port` the messages go to.
    pub target: String,
    pub sent: u32,
    /// Datagrams that came back from Swisslog (read and ignored).
    pub received: u32,
    /// Message type of the last datagram that came back.
    pub last_received_type: Option<u32>,
    pub last_error: Option<String>,
}

/// The settings that need a new start of the link when they change.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Config {
    host: String,
    port: u16,
    id: String,
}

/// The running link. Lives in the core behind a mutex; every call is short and never blocks.
#[derive(Default)]
pub struct SwisslogState {
    sock: Option<UdpSocket>,
    dest: Option<SocketAddr>,
    config: Option<Config>,
    last_pulse: Option<Instant>,
    last_status: Option<StatusInfo>,
    /// Callsign and report of the running QSO, as the interface last told.
    pub dx_call: String,
    pub report: String,
    pub dx_grid: String,
    pub status: SwisslogStatus,
}

impl SwisslogState {
    fn id(&self) -> String {
        self.config.as_ref().map(|c| c.id.clone()).unwrap_or_default()
    }

    fn send(&mut self, data: &[u8]) -> bool {
        let (Some(sock), Some(dest)) = (&self.sock, self.dest) else { return false };
        match sock.send_to(data, dest) {
            Ok(_) => {
                self.status.sent = self.status.sent.wrapping_add(1);
                self.status.last_error = None;
                true
            }
            Err(e) => {
                self.status.last_error = Some(e.to_string());
                false
            }
        }
    }

    /// Says goodbye and closes the socket.
    pub fn stop(&mut self) {
        if self.sock.is_some() {
            let id = self.id();
            self.send(&close(&id));
        }
        self.sock = None;
        self.dest = None;
        self.config = None;
        self.last_pulse = None;
        self.last_status = None;
        self.status.active = false;
    }

    /// Opens (or reopens) the socket when the settings ask for it. Returns false when the link is off.
    fn ensure(&mut self, link: &LinkSettings) -> bool {
        if !link.swisslog_enabled || link.adif_enabled {
            if self.sock.is_some() {
                self.stop();
            }
            return false;
        }
        let id = if link.swisslog_id.trim().is_empty() { "CWLD".to_string() } else { link.swisslog_id.trim().to_string() };
        let want = Config { host: link.swisslog_host.trim().to_string(), port: link.swisslog_port, id };
        if self.config.as_ref() == Some(&want) && self.sock.is_some() {
            return true;
        }
        self.stop();
        self.status = SwisslogStatus { target: format!("{}:{}", want.host, want.port), ..Default::default() };
        let dest = format!("{}:{}", want.host, want.port).to_socket_addrs().ok().and_then(|mut a| a.next());
        let Some(dest) = dest.filter(|_| want.port > 0) else {
            self.status.last_error = Some("unknown host".into());
            self.config = Some(want);
            return false;
        };
        let bind = if dest.is_ipv6() { "[::]:0" } else { "0.0.0.0:0" };
        match UdpSocket::bind(bind) {
            Ok(s) => {
                let _ = s.set_nonblocking(true);
                self.sock = Some(s);
                self.dest = Some(dest);
                self.config = Some(want);
                self.status.active = true;
                true
            }
            Err(e) => {
                self.status.last_error = Some(format!("socket: {e}"));
                self.config = Some(want);
                false
            }
        }
    }

    /// Reads everything that came back and throws it away (counted for the settings window).
    fn drain(&mut self) {
        let Some(sock) = &self.sock else { return };
        let mut buf = [0u8; 2048];
        loop {
            match sock.recv_from(&mut buf) {
                Ok((n, _)) => {
                    self.status.received = self.status.received.wrapping_add(1);
                    if n >= 12 && u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) == MAGIC {
                        self.status.last_received_type = Some(u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]));
                    }
                }
                Err(_) => break,
            }
        }
    }

    /// Called about twice a second: heartbeat, status when something changed, and reading back.
    pub fn tick(&mut self, link: &LinkSettings, radio: &RadioStatus, de_call: &str, de_grid: &str, version: &str, now: Instant) {
        if !self.ensure(link) {
            return;
        }
        self.drain();
        let id = self.id();
        let pulse_due = self.last_pulse.map(|t| now.duration_since(t) >= PULSE).unwrap_or(true);
        if pulse_due {
            self.send(&heartbeat(&id, version));
            self.last_pulse = Some(now);
        }
        if !link.swisslog_live {
            return;
        }
        let info = StatusInfo {
            dial_hz: if radio.connected { radio.frequency_hz } else { 0 },
            mode: mode_name(radio.mode).to_string(),
            dx_call: self.dx_call.clone(),
            report: self.report.clone(),
            transmitting: radio.transmitting,
            de_call: de_call.trim().to_uppercase(),
            de_grid: de_grid.trim().to_uppercase(),
            dx_grid: self.dx_grid.clone(),
        };
        if pulse_due || self.last_status.as_ref() != Some(&info) {
            self.send(&status(&id, &info));
            self.last_status = Some(info);
        }
    }

    /// Hands one logged QSO to Swisslog. Afterwards the callsign of the running QSO is cleared,
    /// so the entry window of Swisslog is empty for the next station.
    pub fn log(&mut self, link: &LinkSettings, q: &Qso, version: &str) -> Result<(), String> {
        if !self.ensure(link) {
            return Err(self.status.last_error.clone().unwrap_or_else(|| "swisslog: off".into()));
        }
        let id = self.id();
        if self.last_pulse.is_none() {
            self.send(&heartbeat(&id, version));
            self.last_pulse = Some(Instant::now());
        }
        let data = match link.swisslog_message {
            SwisslogMessage::QsoLogged => qso_logged(&id, q),
            SwisslogMessage::LoggedAdif => logged_adif(&id, q, version),
        };
        let ok = self.send(&data);
        self.dx_call.clear();
        self.report.clear();
        self.dx_grid.clear();
        if ok {
            Ok(())
        } else {
            Err(self.status.last_error.clone().unwrap_or_else(|| "send failed".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reads the fields back, as a receiver of the WSJT-X format would.
    struct Reader<'a>(&'a [u8], usize);
    impl<'a> Reader<'a> {
        fn u8(&mut self) -> u8 {
            self.1 += 1;
            self.0[self.1 - 1]
        }
        fn u32(&mut self) -> u32 {
            let v = u32::from_be_bytes(self.0[self.1..self.1 + 4].try_into().unwrap());
            self.1 += 4;
            v
        }
        fn u64(&mut self) -> u64 {
            let v = u64::from_be_bytes(self.0[self.1..self.1 + 8].try_into().unwrap());
            self.1 += 8;
            v
        }
        fn i64(&mut self) -> i64 {
            self.u64() as i64
        }
        fn text(&mut self) -> String {
            let n = self.u32() as usize;
            let s = String::from_utf8(self.0[self.1..self.1 + n].to_vec()).unwrap();
            self.1 += n;
            s
        }
        fn done(&self) -> bool {
            self.1 == self.0.len()
        }
    }

    fn header(r: &mut Reader, kind: u32) {
        assert_eq!(r.u32(), MAGIC);
        assert_eq!(r.u32(), SCHEMA);
        assert_eq!(r.u32(), kind);
        assert_eq!(r.text(), "CWLD");
    }

    fn qso() -> Qso {
        Qso {
            id: 7,
            ts: 1_794_146_709, // 2026-11-08 14:05:09 UTC
            call: "DL1ABC".into(),
            freq_hz: 7_030_120,
            mode: "CW".into(),
            rst_sent: "599".into(),
            rst_rcvd: "579".into(),
            name: "Hans".into(),
            tx_pwr: "100".into(),
            comment: "tnx".into(),
            operator: "ON3VZ".into(),
            station_call: "ON3VZ/P".into(),
            my_call: "ON3VZ".into(),
            my_grid: "JO20EE".into(),
            ..Default::default()
        }
    }

    #[test]
    fn heartbeat_has_the_header_and_the_schema() {
        let b = heartbeat("CWLD", "0.21.0");
        let mut r = Reader(&b, 0);
        header(&mut r, TYPE_HEARTBEAT);
        assert_eq!(r.u32(), 3);
        assert_eq!(r.text(), "0.21.0");
        assert_eq!(r.text(), "");
        assert!(r.done());
        // The first bytes are the magic number, big-endian.
        assert_eq!(&b[0..4], &[0xad, 0xbc, 0xcb, 0xda]);
    }

    #[test]
    fn status_fields_are_in_the_order_of_the_description() {
        let s = StatusInfo { dial_hz: 7_025_400, mode: "CW".into(), dx_call: "DL1ABC".into(), report: "599".into(), transmitting: true, de_call: "ON3VZ".into(), de_grid: "JO20EE".into(), dx_grid: "".into() };
        let b = status("CWLD", &s);
        let mut r = Reader(&b, 0);
        header(&mut r, TYPE_STATUS);
        assert_eq!(r.u64(), 7_025_400);
        assert_eq!(r.text(), "CW");
        assert_eq!(r.text(), "DL1ABC");
        assert_eq!(r.text(), "599");
        assert_eq!(r.text(), "CW");
        assert_eq!(r.u8(), 1);
        assert_eq!(r.u8(), 1);
        assert_eq!(r.u8(), 0);
        assert_eq!((r.u32(), r.u32()), (0, 0));
        assert_eq!(r.text(), "ON3VZ");
        assert_eq!(r.text(), "JO20EE");
        assert_eq!(r.text(), "");
        assert_eq!(r.u8(), 0);
        assert_eq!(r.text(), "");
        assert_eq!(r.u8(), 0);
        assert_eq!(r.u8(), 0);
        assert_eq!((r.u32(), r.u32()), (0, 0));
        assert_eq!(r.text(), "");
        assert_eq!(r.text(), "");
        assert!(r.done());
    }

    #[test]
    fn qso_logged_has_utc_times_and_the_fields_swisslog_takes() {
        let b = qso_logged("CWLD", &qso());
        let mut r = Reader(&b, 0);
        header(&mut r, TYPE_QSO_LOGGED);
        // 2026-11-08 is Julian day 2461353; 14:05:09 is 50709 s after midnight.
        assert_eq!(r.i64(), 2_461_353);
        assert_eq!(r.u32(), 50_709_000);
        assert_eq!(r.u8(), 1);
        assert_eq!(r.text(), "DL1ABC");
        assert_eq!(r.text(), "");
        assert_eq!(r.u64(), 7_030_120);
        assert_eq!(r.text(), "CW");
        assert_eq!(r.text(), "599");
        assert_eq!(r.text(), "579");
        assert_eq!(r.text(), "100");
        assert_eq!(r.text(), "tnx");
        assert_eq!(r.text(), "Hans");
        assert_eq!((r.i64(), r.u32(), r.u8()), (2_461_353, 50_709_000, 1));
        assert_eq!(r.text(), "ON3VZ");
        assert_eq!(r.text(), "ON3VZ/P");
        assert_eq!(r.text(), "JO20EE");
        assert_eq!(r.text(), "");
        assert_eq!(r.text(), "");
        assert_eq!(r.text(), "");
        assert!(r.done());
    }

    #[test]
    fn contest_exchange_goes_along() {
        let mut q = qso();
        q.nr_sent = 12;
        q.extra_sent = "WLD".into();
        q.nr_rcvd = "023".into();
        let b = qso_logged("CWLD", &q);
        let s = String::from_utf8_lossy(&b);
        assert!(s.contains("12 WLD"));
        assert!(s.contains("023"));
    }

    #[test]
    fn julian_day_of_the_unix_epoch_and_before() {
        let mut m = Msg(Vec::new());
        m.date_time(0);
        assert_eq!(&m.0[0..8], &2_440_588i64.to_be_bytes());
        let mut m = Msg(Vec::new());
        m.date_time(-1);
        assert_eq!(&m.0[0..8], &2_440_587i64.to_be_bytes());
        assert_eq!(&m.0[8..12], &86_399_000u32.to_be_bytes());
    }

    #[test]
    fn logged_adif_is_a_small_adif_file() {
        let b = logged_adif("CWLD", &qso(), "0.21.0");
        let mut r = Reader(&b, 0);
        header(&mut r, TYPE_LOGGED_ADIF);
        let t = r.text();
        assert!(t.starts_with("<ADIF_VER:5>3.1.4"));
        assert!(t.contains("<PROGRAMVERSION:6>0.21.0"));
        assert!(t.contains("<EOH>"));
        assert!(t.contains("<CALL:6>DL1ABC"));
        assert!(t.trim_end().ends_with("<EOR>"));
        assert!(r.done());
    }

    #[test]
    fn short_or_strange_calls_are_not_sent() {
        assert_eq!(dx_call_for_status("dl"), "");
        assert_eq!(dx_call_for_status(" dl1abc "), "DL1ABC");
        assert_eq!(dx_call_for_status("ea8/on3vz"), "EA8/ON3VZ");
        assert_eq!(dx_call_for_status("DL1?BC"), "");
    }

    fn link(port: u16) -> LinkSettings {
        LinkSettings { swisslog_enabled: true, swisslog_port: port, ..LinkSettings::default() }
    }

    fn radio() -> RadioStatus {
        RadioStatus { connected: true, frequency_hz: 7_025_000, mode: RadioMode::Cw, filter: None, cw_wpm: 22, transmitting: false, s_meter: 0, split: Default::default() }
    }

    fn recv_all(rx: &UdpSocket) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let mut buf = [0u8; 4096];
        while let Ok(n) = rx.recv(&mut buf) {
            out.push(buf[..n].to_vec());
        }
        out
    }

    fn kind(b: &[u8]) -> u32 {
        u32::from_be_bytes(b[8..12].try_into().unwrap())
    }

    #[test]
    fn a_fake_swisslog_gets_heartbeat_status_and_the_qso() {
        let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
        rx.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
        let l = link(rx.local_addr().unwrap().port());
        let mut st = SwisslogState::default();
        let t0 = Instant::now();
        st.tick(&l, &radio(), "ON3VZ", "JO20EE", "0.21.0", t0);
        let got = recv_all(&rx);
        assert_eq!(got.iter().map(|b| kind(b)).collect::<Vec<_>>(), vec![TYPE_HEARTBEAT, TYPE_STATUS]);

        // Nothing changed: nothing is sent.
        st.tick(&l, &radio(), "ON3VZ", "JO20EE", "0.21.0", t0 + Duration::from_millis(500));
        assert!(recv_all(&rx).is_empty());

        // A callsign in the QSO panel: a new status with it.
        st.dx_call = dx_call_for_status("dl1abc");
        st.tick(&l, &radio(), "ON3VZ", "JO20EE", "0.21.0", t0 + Duration::from_secs(1));
        let got = recv_all(&rx);
        assert_eq!(got.len(), 1);
        assert!(String::from_utf8_lossy(&got[0]).contains("DL1ABC"));

        // Logging: one QSO Logged, and the callsign is cleared for the next status.
        st.log(&l, &qso(), "0.21.0").unwrap();
        let got = recv_all(&rx);
        assert_eq!(got.iter().map(|b| kind(b)).collect::<Vec<_>>(), vec![TYPE_QSO_LOGGED]);
        st.tick(&l, &radio(), "ON3VZ", "JO20EE", "0.21.0", t0 + Duration::from_secs(2));
        let got = recv_all(&rx);
        assert_eq!(got.len(), 1);
        assert!(!String::from_utf8_lossy(&got[0]).contains("DL1ABC"));

        // After the pulse a heartbeat and the status again.
        st.tick(&l, &radio(), "ON3VZ", "JO20EE", "0.21.0", t0 + Duration::from_secs(20));
        let got = recv_all(&rx);
        assert_eq!(got.iter().map(|b| kind(b)).collect::<Vec<_>>(), vec![TYPE_HEARTBEAT, TYPE_STATUS]);

        // Switched off: a Close, then silence.
        let mut off = l.clone();
        off.swisslog_enabled = false;
        st.tick(&off, &radio(), "ON3VZ", "JO20EE", "0.21.0", t0 + Duration::from_secs(21));
        let got = recv_all(&rx);
        assert_eq!(got.iter().map(|b| kind(b)).collect::<Vec<_>>(), vec![TYPE_CLOSE]);
        st.tick(&off, &radio(), "ON3VZ", "JO20EE", "0.21.0", t0 + Duration::from_secs(40));
        assert!(recv_all(&rx).is_empty());
    }

    #[test]
    fn logged_adif_is_sent_instead_of_qso_logged_when_chosen() {
        let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
        rx.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
        let mut l = link(rx.local_addr().unwrap().port());
        l.swisslog_message = SwisslogMessage::LoggedAdif;
        let mut st = SwisslogState::default();
        st.log(&l, &qso(), "1").unwrap();
        let kinds: Vec<u32> = recv_all(&rx).iter().map(|b| kind(b)).collect();
        assert_eq!(kinds, vec![TYPE_HEARTBEAT, TYPE_LOGGED_ADIF]);
    }

    #[test]
    fn without_live_status_only_the_heartbeat_goes() {
        let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
        rx.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
        let mut l = link(rx.local_addr().unwrap().port());
        l.swisslog_live = false;
        let mut st = SwisslogState::default();
        st.dx_call = "DL1ABC".into();
        st.tick(&l, &radio(), "ON3VZ", "", "1", Instant::now());
        let kinds: Vec<u32> = recv_all(&rx).iter().map(|b| kind(b)).collect();
        assert_eq!(kinds, vec![TYPE_HEARTBEAT]);
    }

    #[test]
    fn log4om_wins_so_nothing_goes_to_swisslog() {
        let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
        rx.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
        let mut l = link(rx.local_addr().unwrap().port());
        l.adif_enabled = true;
        let mut st = SwisslogState::default();
        st.tick(&l, &radio(), "ON3VZ", "", "1", Instant::now());
        assert!(st.log(&l, &qso(), "1").is_err());
        assert!(recv_all(&rx).is_empty());
    }

    #[test]
    fn what_comes_back_is_counted_and_ignored() {
        // A fake Swisslog that answers with a Reply (4) and a Halt Tx (8).
        let peer = UdpSocket::bind("127.0.0.1:0").unwrap();
        peer.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
        let l = link(peer.local_addr().unwrap().port());
        let mut st = SwisslogState::default();
        let t0 = Instant::now();
        st.tick(&l, &radio(), "ON3VZ", "", "1", t0);
        let mut buf = [0u8; 512];
        let (_, from) = peer.recv_from(&mut buf).unwrap();
        for k in [4u32, 8] {
            let mut m = Msg::new(k, "SWISSLOG");
            m.u8(0);
            peer.send_to(&m.0, from).unwrap();
        }
        std::thread::sleep(Duration::from_millis(50));
        st.tick(&l, &radio(), "ON3VZ", "", "1", t0 + Duration::from_millis(500));
        assert_eq!(st.status.received, 2);
        assert_eq!(st.status.last_received_type, Some(8));
    }
}
