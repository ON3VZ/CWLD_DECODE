//! ADIF writer (ADI format, plain ASCII, UTC).

use crate::Qso;

#[derive(Debug, Clone, Default)]
pub struct AdifOptions {
    /// CONTEST_ID written on every contest QSO. Empty: the field is left out.
    pub contest_id: String,
    pub program_version: String,
    /// Seconds since 1970 for the CREATED_TIMESTAMP in the header.
    pub now_ts: i64,
}

/// The file name WWFF asks for: `ON3VZ-P @ ONFF-0599 20261103.adi` (a slash in the call becomes a dash).
pub fn wwff_file_name(call: &str, reference: &str, started_ts: i64) -> String {
    let (d, _) = date_time(started_ts);
    format!("{} @ {} {}.adi", call.trim().to_uppercase().replace('/', "-"), reference.trim().to_uppercase(), d)
}

/// Year, month, day from days since 1970-01-01 (proleptic Gregorian calendar).
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// ("YYYYMMDD", "HHMMSS") in UTC.
pub fn date_time(ts: i64) -> (String, String) {
    let (y, m, d) = civil(ts.div_euclid(86_400));
    let s = ts.rem_euclid(86_400);
    (format!("{y:04}{m:02}{d:02}"), format!("{:02}{:02}{:02}", s / 3600, (s % 3600) / 60, s % 60))
}

/// ADIF is plain ASCII: anything else becomes '?'. Line breaks become spaces.
fn ascii(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii() && !c.is_ascii_control() { c } else if c.is_ascii_whitespace() { ' ' } else { '?' }).collect()
}

fn field(out: &mut String, name: &str, value: &str) {
    let v = ascii(value.trim());
    if v.is_empty() {
        return;
    }
    out.push_str(&format!("<{name}:{}>{v}", v.len()));
}

pub fn adif_text(qsos: &[Qso], o: &AdifOptions) -> String {
    let mut out = String::new();
    out.push_str("CWLD ADIF export\n");
    let mut hdr = String::new();
    field(&mut hdr, "ADIF_VER", "3.1.4");
    field(&mut hdr, "PROGRAMID", "CWLD");
    field(&mut hdr, "PROGRAMVERSION", &o.program_version);
    let (d, t) = date_time(o.now_ts);
    field(&mut hdr, "CREATED_TIMESTAMP", &format!("{d} {t}"));
    out.push_str(&hdr);
    out.push_str("<EOH>\n");
    // Oldest first, as a logbook reads.
    let mut sorted: Vec<&Qso> = qsos.iter().collect();
    sorted.sort_by_key(|q| (q.ts, q.id));
    for q in sorted {
        out.push_str(&adif_record(q, o));
    }
    out
}

/// One QSO as a single ADIF record ending in `<EOR>` and a newline, without a header.
/// This is what goes in one UDP datagram to a logger.
pub fn adif_record(q: &Qso, o: &AdifOptions) -> String {
    let mut l = String::new();
    let (d, t) = date_time(q.ts);
    field(&mut l, "CALL", &q.call);
    field(&mut l, "QSO_DATE", &d);
    field(&mut l, "TIME_ON", &t);
    field(&mut l, "BAND", &q.band);
    if q.freq_hz > 0 {
        field(&mut l, "FREQ", &format!("{:.6}", q.freq_hz as f64 / 1e6));
    }
    // Split: where this station listened. Only written when it differs from FREQ.
    if q.freq_rx_hz > 0 && q.freq_rx_hz != q.freq_hz {
        field(&mut l, "FREQ_RX", &format!("{:.6}", q.freq_rx_hz as f64 / 1e6));
        let rx_band = crate::band_of(q.freq_rx_hz);
        if !rx_band.is_empty() && rx_band != q.band {
            field(&mut l, "BAND_RX", rx_band);
        }
    }
    field(&mut l, "MODE", if q.mode.is_empty() { "CW" } else { &q.mode });
    field(&mut l, "RST_SENT", &q.rst_sent);
    field(&mut l, "RST_RCVD", &q.rst_rcvd);
    if q.nr_sent > 0 {
        field(&mut l, "STX", &q.nr_sent.to_string());
    }
    field(&mut l, "STX_STRING", &q.extra_sent);
    if !q.nr_rcvd.is_empty() && q.nr_rcvd.chars().all(|c| c.is_ascii_digit()) {
        // The serial number as a plain integer: 007 becomes 7.
        let n: u64 = q.nr_rcvd.parse().unwrap_or(0);
        field(&mut l, "SRX", &n.to_string());
    }
    field(&mut l, "SRX_STRING", &q.extra_rcvd);
    if !q.contest.is_empty() {
        field(&mut l, "CONTEST_ID", &o.contest_id);
    }
    let station = if q.station_call.is_empty() { &q.my_call } else { &q.station_call };
    let operator = if q.operator.is_empty() { &q.my_call } else { &q.operator };
    field(&mut l, "STATION_CALLSIGN", station);
    field(&mut l, "OPERATOR", operator);
    field(&mut l, "MY_GRIDSQUARE", &q.my_grid.to_uppercase());
    field(&mut l, "TX_PWR", &q.tx_pwr);
    field(&mut l, "MY_RIG", &q.my_rig);
    field(&mut l, "MY_ANTENNA", &q.my_antenna);
    field(&mut l, "NAME", &q.name);
    field(&mut l, "QTH", &q.qth);
    field(&mut l, "COMMENT", &q.comment);
    // Own references. WWFF asks for MY_SIG and MY_SIG_INFO; the ADIF field MY_WWFF_REF is written too.
    if !q.my_wwff.is_empty() {
        field(&mut l, "MY_SIG", "WWFF");
        field(&mut l, "MY_SIG_INFO", &q.my_wwff);
        field(&mut l, "MY_WWFF_REF", &q.my_wwff);
    }
    field(&mut l, "MY_POTA_REF", &q.my_pota);
    field(&mut l, "MY_SOTA_REF", &q.my_sota);
    // Park to park: the reference of the other station, with its own field per program.
    if !q.their_ref.is_empty() {
        match q.their_prog.as_str() {
            "WWFF" => {
                field(&mut l, "SIG", "WWFF");
                field(&mut l, "SIG_INFO", &q.their_ref);
                field(&mut l, "WWFF_REF", &q.their_ref);
            }
            "POTA" => field(&mut l, "POTA_REF", &q.their_ref),
            "SOTA" => field(&mut l, "SOTA_REF", &q.their_ref),
            _ => {}
        }
    }
    l.push_str("<EOR>\n");
    l
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_and_time_in_utc() {
        assert_eq!(date_time(0), ("19700101".into(), "000000".into()));
        // 2026-11-08 14:05:09 UTC
        assert_eq!(date_time(1_794_146_709), ("20261108".into(), "140509".into()));
        assert_eq!(date_time(951_782_400), ("20000229".into(), "000000".into()), "leap day");
    }

    fn sample() -> Qso {
        Qso { id: 1, ts: 1_794_146_709, call: "PA3ABC".into(), band: "40m".into(), freq_hz: 7_030_000, mode: "CW".into(), rst_sent: "599".into(), rst_rcvd: "579".into(), nr_sent: 42, nr_rcvd: "007".into(), extra_sent: "WLD".into(), extra_rcvd: "ANR".into(), my_call: "ON3VZ".into(), contest: "uba-dx-cw".into(), ..Default::default() }
    }

    #[test]
    fn a_split_qso_writes_freq_rx_and_a_normal_one_does_not() {
        let o = AdifOptions { contest_id: String::new(), program_version: "0.20.3".into(), now_ts: 1_794_146_709 };
        assert!(!adif_text(&[sample()], &o).contains("FREQ_RX"));
        let split = Qso { freq_hz: 7_026_000, freq_rx_hz: 7_025_000, ..sample() };
        let s = adif_text(&[split], &o);
        assert!(s.contains("<FREQ:8>7.026000"), "{s}");
        assert!(s.contains("<FREQ_RX:8>7.025000"), "{s}");
        assert!(!s.contains("BAND_RX"), "same band: {s}");
        // The same frequency twice is not split.
        assert!(!adif_text(&[Qso { freq_rx_hz: 7_030_000, ..sample() }], &o).contains("FREQ_RX"));
    }

    #[test]
    fn writes_adif_fields_with_correct_lengths() {
        let o = AdifOptions { contest_id: "UBA-DX-CW".into(), program_version: "0.7.0".into(), now_ts: 1_794_146_709 };
        let s = adif_text(&[sample()], &o);
        assert!(s.contains("<EOH>"));
        assert!(s.contains("<CALL:6>PA3ABC"));
        assert!(s.contains("<QSO_DATE:8>20261108"));
        assert!(s.contains("<TIME_ON:6>140509"));
        assert!(s.contains("<FREQ:8>7.030000"));
        assert!(s.contains("<STX:2>42"));
        assert!(s.contains("<SRX:1>7"), "{s}");
        assert!(s.contains("<STX_STRING:3>WLD"));
        assert!(s.contains("<SRX_STRING:3>ANR"));
        assert!(s.contains("<CONTEST_ID:9>UBA-DX-CW"));
        assert!(s.contains("<STATION_CALLSIGN:5>ON3VZ"));
        assert!(s.trim_end().ends_with("<EOR>"));
    }

    #[test]
    fn no_contest_id_for_a_normal_qso_and_ascii_only() {
        let mut q = sample();
        q.contest.clear();
        q.name = "Jos\u{e9}\nB".into();
        let s = adif_text(&[q], &AdifOptions::default());
        assert!(!s.contains("CONTEST_ID"));
        assert!(s.contains("<NAME:6>Jos? B"), "{s}");
        assert!(s.is_ascii());
    }

    #[test]
    fn writes_station_and_activation_fields() {
        let mut q = sample();
        q.contest.clear();
        q.station_call = "ON3VZ/P".into();
        q.operator = "ON3VZ".into();
        q.my_grid = "jo20qe".into();
        q.tx_pwr = "5".into();
        q.my_rig = "QRP Labs QMX".into();
        q.comment = "5W QRP".into();
        q.my_wwff = "ONFF-0599".into();
        q.their_prog = "WWFF".into();
        q.their_ref = "ONFF-0123".into();
        let s = adif_text(&[q], &AdifOptions::default());
        for f in ["<STATION_CALLSIGN:7>ON3VZ/P", "<OPERATOR:5>ON3VZ", "<MY_GRIDSQUARE:6>JO20QE", "<TX_PWR:1>5", "<MY_RIG:12>QRP Labs QMX", "<COMMENT:6>5W QRP", "<MY_SIG:4>WWFF", "<MY_SIG_INFO:9>ONFF-0599", "<MY_WWFF_REF:9>ONFF-0599", "<SIG:4>WWFF", "<SIG_INFO:9>ONFF-0123", "<WWFF_REF:9>ONFF-0123"] {
            assert!(s.contains(f), "{f} missing in {s}");
        }
        assert!(!s.contains("POTA"));
    }

    #[test]
    fn pota_and_sota_use_their_own_fields_not_sig() {
        let mut q = sample();
        q.my_pota = "ON-00001".into();
        q.their_prog = "POTA".into();
        q.their_ref = "ON-00002".into();
        let s = adif_text(&[q], &AdifOptions::default());
        assert!(s.contains("<MY_POTA_REF:8>ON-00001") && s.contains("<POTA_REF:8>ON-00002"));
        assert!(!s.contains("<SIG:"));
    }

    #[test]
    fn file_name_for_the_wwff_coordinator() {
        assert_eq!(wwff_file_name("on3vz/p", "onff-0599", 1_793_664_000), "ON3VZ-P @ ONFF-0599 20261103.adi");
    }

    #[test]
    fn oldest_first() {
        let mut a = sample();
        a.call = "LATE".into();
        a.ts = 2000;
        let mut b = sample();
        b.call = "EARLY".into();
        b.ts = 1000;
        let s = adif_text(&[a, b], &AdifOptions::default());
        assert!(s.find("EARLY").unwrap() < s.find("LATE").unwrap());
    }
}
