//! Sending logged QSOs to other programs over UDP.
//!
//! Two plain datagram formats, both one-way (CWLD never receives anything here):
//! - ADIF: one record per datagram, for Log4OM (UDP INBOUND, service type ADIF_MESSAGE).
//! - N1MM Logger+ XML (`contactinfo`, `contactreplace`, `contactdelete`), for listeners of those broadcasts.
//!   N1MM itself cannot take QSOs over UDP, so this is not a way to log into N1MM.
//!
//! Test QSOs are never sent. A failing send never blocks logging: the result is only reported.

use crate::{AppCore, CoreError};
use cwld_log::{adif_date_time, adif_record, AdifOptions, Qso};
use cwld_types::settings::LinkSettings;
use serde::Serialize;
use std::net::{ToSocketAddrs, UdpSocket};

/// What happened to one send, for the interface.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct LinkReport {
    /// Datagrams handed to the network.
    pub sent: usize,
    /// Destinations that failed, as `host:port: reason`.
    pub errors: Vec<String>,
}

fn xml_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            c if c.is_ascii_control() => " ".to_string(),
            c => c.to_string(),
        })
        .collect()
}

fn tag(out: &mut String, name: &str, value: &str) {
    out.push_str(&format!("<{name}>{}</{name}>", xml_escape(value.trim())));
}

/// A stable id for a QSO so replace and delete messages find it again (N1MM wants a GUID-like string).
pub fn qso_guid(q: &Qso) -> String {
    let mut h: u128 = 0xcbf2_9ce4_8422_2325;
    for b in format!("{}|{}|{}|{}", q.ts, q.call, q.station_call, q.id).bytes() {
        h = (h ^ b as u128).wrapping_mul(0x0100_0000_01b3).rotate_left(7);
    }
    let x = format!("{h:032x}");
    format!("{}-{}-{}-{}-{}", &x[0..8], &x[8..12], &x[12..16], &x[16..20], &x[20..32])
}

/// N1MM message for one QSO. `kind` is `contactinfo`, `contactreplace` or `contactdelete`.
pub fn n1mm_message(kind: &str, q: &Qso, contest_id: &str) -> String {
    let (d, t) = adif_date_time(q.ts);
    let stamp = format!("{}-{}-{} {}:{}:{}", &d[0..4], &d[4..6], &d[6..8], &t[0..2], &t[2..4], &t[4..6]);
    let mut o = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>");
    o.push_str(&format!("<{kind}>"));
    tag(&mut o, "app", "CWLD");
    tag(&mut o, "contestname", if contest_id.is_empty() { "CW" } else { contest_id });
    tag(&mut o, "timestamp", &stamp);
    tag(&mut o, "mycall", if q.station_call.is_empty() { &q.my_call } else { &q.station_call });
    // N1MM counts frequency in units of 10 Hz.
    tag(&mut o, "band", &format!("{:.3}", q.freq_hz as f64 / 1e6));
    tag(&mut o, "rxfreq", &(q.freq_hz / 10).to_string());
    tag(&mut o, "txfreq", &(q.freq_hz / 10).to_string());
    tag(&mut o, "operator", &q.operator);
    tag(&mut o, "mode", if q.mode.is_empty() { "CW" } else { &q.mode });
    tag(&mut o, "call", &q.call);
    tag(&mut o, "snt", &q.rst_sent);
    tag(&mut o, "rcv", &q.rst_rcvd);
    tag(&mut o, "sntnr", &q.nr_sent.to_string());
    tag(&mut o, "rcvnr", &q.nr_rcvd);
    tag(&mut o, "exchange1", &q.extra_rcvd);
    tag(&mut o, "name", &q.name);
    tag(&mut o, "comment", &q.comment);
    tag(&mut o, "ID", &qso_guid(q));
    o.push_str(&format!("</{kind}>"));
    o
}

/// Splits `a:1 b:2` into destinations. Entries without a valid port are skipped.
pub fn parse_targets(s: &str) -> Vec<String> {
    s.split_whitespace().filter(|t| t.rsplit_once(':').map(|(h, p)| !h.is_empty() && p.parse::<u16>().map(|n| n > 0).unwrap_or(false)).unwrap_or(false)).map(|t| t.to_string()).collect()
}

fn send_to(sock: &UdpSocket, target: &str, data: &[u8], report: &mut LinkReport) {
    match target.to_socket_addrs().ok().and_then(|mut a| a.next()) {
        Some(addr) => match sock.send_to(data, addr) {
            Ok(_) => report.sent += 1,
            Err(e) => report.errors.push(format!("{target}: {e}")),
        },
        None => report.errors.push(format!("{target}: unknown host")),
    }
}

/// Sends one QSO to every enabled destination. Test, deleted and empty-call QSOs are skipped.
pub fn send_qso(link: &LinkSettings, q: &Qso, contest_name: &str, version: &str, kind: &str) -> LinkReport {
    let mut report = LinkReport::default();
    if q.test || q.call.trim().is_empty() || !(link.adif_enabled || link.n1mm_enabled) {
        return report;
    }
    let sock = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => {
            report.errors.push(format!("socket: {e}"));
            return report;
        }
    };
    if link.adif_enabled && kind != "contactdelete" && link.adif_port > 0 {
        // CONTEST_ID of ADIF has fixed names, so the free name of a profile is not sent there.
        let rec = adif_record(q, &AdifOptions { contest_id: String::new(), program_version: version.to_string(), now_ts: q.ts });
        send_to(&sock, &format!("{}:{}", link.adif_host.trim(), link.adif_port), rec.as_bytes(), &mut report);
    }
    if link.n1mm_enabled {
        let msg = n1mm_message(kind, q, contest_name);
        for t in parse_targets(&link.n1mm_targets) {
            send_to(&sock, &t, msg.as_bytes(), &mut report);
        }
    }
    report
}

impl AppCore {
    fn link_send(&self, q: &Qso, kind: &str) -> LinkReport {
        let link = self.settings.lock().unwrap().link.clone();
        let contest_id = self.settings.lock().unwrap().contest.profiles.iter().find(|p| p.id == q.contest).map(|p| p.name.clone()).unwrap_or_default();
        send_qso(&link, q, &contest_id, env!("CARGO_PKG_VERSION"), kind)
    }

    /// Sends the newest real QSO again, as a test of the connection. Never makes up a QSO.
    pub fn link_resend_last(&self) -> Result<LinkReport, CoreError> {
        let q = {
            let lb = self.logbook.lock().unwrap();
            lb.list(&cwld_log::LogFilter { test: Some(false), limit: 1, ..Default::default() })?.into_iter().next()
        };
        match q {
            Some(q) => Ok(self.link_send(&q, "contactinfo")),
            None => Err(CoreError::Invalid("link_no_qso")),
        }
    }

    /// Called after a QSO was stored. Sends it as new, or as a replacement when it was edited.
    pub(crate) fn link_after_add(&self, q: &Qso) -> LinkReport {
        self.link_send(q, "contactinfo")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qso() -> Qso {
        Qso { id: 3, ts: 1_794_146_709, call: "DL1ABC".into(), freq_hz: 7_030_120, rst_sent: "599".into(), rst_rcvd: "579".into(), my_call: "ON3VZ".into(), name: "A&B".into(), ..Default::default() }
    }

    #[test]
    fn n1mm_message_has_the_fields_and_escapes() {
        let m = n1mm_message("contactinfo", &qso(), "");
        assert!(m.contains("<call>DL1ABC</call>"));
        assert!(m.contains("<rxfreq>703012</rxfreq>"));
        assert!(m.contains("<timestamp>2026-11-08 14:05:09</timestamp>"));
        assert!(m.contains("<name>A&amp;B</name>"));
        assert!(m.starts_with("<?xml"));
        assert!(m.ends_with("</contactinfo>"));
    }

    #[test]
    fn guid_is_stable_and_shaped() {
        let g = qso_guid(&qso());
        assert_eq!(g, qso_guid(&qso()));
        assert_eq!(g.len(), 36);
        assert_eq!(g.matches('-').count(), 4);
    }

    #[test]
    fn targets_are_parsed() {
        assert_eq!(parse_targets("127.0.0.1:12060 pc2:9 bad nope:0 :5"), vec!["127.0.0.1:12060", "pc2:9"]);
    }

    #[test]
    fn nothing_is_sent_when_off_or_test() {
        let off = LinkSettings::default();
        assert_eq!(send_qso(&off, &qso(), "", "x", "contactinfo"), LinkReport::default());
        let mut on = LinkSettings::default();
        on.adif_enabled = true;
        let mut t = qso();
        t.test = true;
        assert_eq!(send_qso(&on, &t, "", "x", "contactinfo"), LinkReport::default());
    }

    #[test]
    fn adif_and_n1mm_arrive_over_udp() {
        let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
        rx.set_read_timeout(Some(std::time::Duration::from_secs(2))).unwrap();
        let port = rx.local_addr().unwrap().port();
        let mut l = LinkSettings::default();
        l.adif_enabled = true;
        l.adif_port = port;
        l.n1mm_enabled = true;
        l.n1mm_targets = format!("127.0.0.1:{port}");
        let r = send_qso(&l, &qso(), "", "1", "contactinfo");
        assert_eq!((r.sent, r.errors.len()), (2, 0));
        let mut buf = [0u8; 4096];
        let mut got = Vec::new();
        for _ in 0..2 {
            let n = rx.recv(&mut buf).unwrap();
            got.push(String::from_utf8_lossy(&buf[..n]).to_string());
        }
        assert!(got.iter().any(|g| g.contains("<CALL:6>DL1ABC") && g.trim_end().ends_with("<EOR>") && !g.contains("<EOH>")));
        assert!(got.iter().any(|g| g.contains("<contactinfo>")));
    }

    #[test]
    fn delete_is_not_sent_as_adif() {
        let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
        let mut l = LinkSettings::default();
        l.adif_enabled = true;
        l.adif_port = rx.local_addr().unwrap().port();
        assert_eq!(send_qso(&l, &qso(), "", "1", "contactdelete").sent, 0);
    }
}
