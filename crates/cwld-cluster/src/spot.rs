//! One spot line of a DX cluster, and what the cluster software wants to be told.
//!
//! The line formats of DX Spider, CC Cluster and AR-Cluster differ only in spacing:
//! `DX de SPOTTER:  14025.0  CALL  comment  1234Z` with an optional grid after the time.

use serde::{Deserialize, Serialize};

/// The mode a spot is for, as far as the comment tells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpotMode {
    Cw,
    /// Another mode is named in the comment (FT8, RTTY, SSB and so on).
    Other,
    /// The comment does not say.
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spot {
    pub call: String,
    pub freq_hz: u64,
    pub spotter: String,
    /// The free text without the time and grid.
    pub comment: String,
    pub mode: SpotMode,
    pub snr: Option<i32>,
    pub wpm: Option<u16>,
    /// A skimmer (RBN) spotted it: the spotter ends in `-#`.
    pub skimmer: bool,
}

/// The cluster programs CWLD knows commands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Software {
    Spider,
    Cc,
    Ar,
    Unknown,
}

impl Software {
    pub fn name(self) -> &'static str {
        match self {
            Software::Spider => "DX Spider",
            Software::Cc => "CC Cluster",
            Software::Ar => "AR-Cluster",
            Software::Unknown => "unknown",
        }
    }
}

/// Reads the program from the welcome text. Unknown is a normal answer.
pub fn detect_software(text: &str) -> Software {
    let t = text.to_ascii_lowercase();
    if t.contains("dxspider") || t.contains("dx spider") {
        Software::Spider
    } else if t.contains("ar-cluster") || t.contains("arcluster") || t.contains("ar cluster") {
        Software::Ar
    } else if t.contains("cc cluster") || t.contains("ccc") || t.contains("ve7cc") || t.contains("cc user") {
        Software::Cc
    } else {
        Software::Unknown
    }
}

/// Commands sent after the login, one per line. Spots of other modes are also filtered on our
/// side, so a command a node does not know only costs an error line in the raw log.
pub fn init_commands(sw: Software, skimmer: bool) -> Vec<String> {
    let v = |l: &[&str]| l.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    match (sw, skimmer) {
        (Software::Spider, true) => v(&["set/skimmer CW"]),
        (Software::Spider, false) => v(&["unset/skimmer"]),
        (Software::Ar, true) => v(&["set dx filter"]),
        (Software::Ar, false) => v(&["set dx filter not skimmer"]),
        // CC Cluster, and the best guess for a node that does not say what it runs.
        (_, true) => v(&["set/skimmer", "set/cw", "set/noft8", "set/noft4", "set/nortty", "set/nopsk"]),
        (_, false) => v(&["set/noskimmer"]),
    }
}

/// True for a plausible amateur callsign: letters and digits (and `/`), at least one of each.
pub fn valid_call(s: &str) -> bool {
    let n = s.len();
    (3..=14).contains(&n)
        && s.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'/')
        && s.bytes().any(|b| b.is_ascii_digit())
        && s.bytes().any(|b| b.is_ascii_uppercase())
        && !s.starts_with('/')
        && !s.ends_with('/')
}

fn is_time(tok: &str) -> bool {
    tok.len() == 5 && tok.ends_with('Z') && tok[..4].bytes().all(|b| b.is_ascii_digit())
}

const OTHER_MODES: [&str; 22] = [
    "FT8", "FT4", "RTTY", "PSK", "PSK31", "PSK63", "SSB", "USB", "LSB", "FM", "AM", "JT65", "JT9", "MSK144", "FST4", "Q65", "OLIVIA", "DIGI", "DATA", "SSTV", "MFSK", "JS8",
];

/// Reads `<number> dB` and `<number> WPM` out of a skimmer comment such as `CW 24 dB 22 WPM CQ`.
fn numbers_of(comment: &str) -> (Option<i32>, Option<u16>) {
    let toks: Vec<&str> = comment.split_whitespace().collect();
    let (mut snr, mut wpm) = (None, None);
    for w in toks.windows(2) {
        let unit = w[1].to_ascii_uppercase();
        if unit == "DB" {
            snr = w[0].parse::<i32>().ok();
        } else if unit == "WPM" {
            wpm = w[0].parse::<u16>().ok();
        }
    }
    (snr, wpm)
}

/// Parses one line. Anything that is not a spot with a sane callsign and frequency gives `None`.
pub fn parse_spot(line: &str) -> Option<Spot> {
    let line = line.trim();
    let rest = line.strip_prefix("DX de ").or_else(|| line.strip_prefix("DX DE "))?;
    let toks: Vec<&str> = rest.split_whitespace().collect();
    if toks.len() < 3 {
        return None;
    }
    let spotter = toks[0].trim_end_matches(':').to_ascii_uppercase();
    let khz: f64 = toks[1].parse().ok()?;
    if !(100.0..=1_000_000.0).contains(&khz) {
        return None;
    }
    let call = toks[2].to_ascii_uppercase();
    if !valid_call(&call) {
        return None;
    }
    let tail = &toks[3..];
    // The time is the last token that looks like 1234Z; a grid square may follow it.
    let end = tail.iter().rposition(|t| is_time(t)).unwrap_or(tail.len());
    let comment = tail[..end].join(" ");
    let upper: Vec<String> = tail[..end].iter().map(|t| t.to_ascii_uppercase()).collect();
    let (snr, wpm) = numbers_of(&comment);
    let mode = if upper.iter().any(|t| t == "CW") || wpm.is_some() {
        SpotMode::Cw
    } else if upper.iter().any(|t| OTHER_MODES.contains(&t.as_str())) {
        SpotMode::Other
    } else {
        SpotMode::Unknown
    };
    Some(Spot { call, freq_hz: (khz * 1000.0).round() as u64, skimmer: spotter.ends_with("-#"), spotter, comment, mode, snr, wpm })
}

/// Where CW is normally used, in kHz. Used for spots that do not name their mode.
const CW_SEGMENTS: [(u64, u64); 10] = [
    (1800, 1840),
    (3500, 3570),
    (7000, 7040),
    (10100, 10130),
    (14000, 14070),
    (18068, 18095),
    (21000, 21070),
    (24890, 24915),
    (28000, 28070),
    (50000, 50100),
];

pub fn in_cw_segment(freq_hz: u64) -> bool {
    let khz = freq_hz / 1000;
    CW_SEGMENTS.iter().any(|&(a, b)| khz >= a && khz <= b)
}

impl Spot {
    /// A CW spot: the comment says CW, or it says nothing and the frequency is in the CW part of a band.
    pub fn is_cw(&self) -> bool {
        match self.mode {
            SpotMode::Cw => true,
            SpotMode::Other => false,
            SpotMode::Unknown => in_cw_segment(self.freq_hz),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_skimmer_spot_of_cc_cluster() {
        let s = parse_spot("DX de DL1ABC-#:   7030.1  ON3VZ        CW 24 dB 22 WPM CQ             1234Z JO21").unwrap();
        assert_eq!(s.call, "ON3VZ");
        assert_eq!(s.freq_hz, 7_030_100);
        assert_eq!(s.spotter, "DL1ABC-#");
        assert!(s.skimmer);
        assert_eq!((s.snr, s.wpm), (Some(24), Some(22)));
        assert_eq!(s.comment, "CW 24 dB 22 WPM CQ");
        assert_eq!(s.mode, SpotMode::Cw);
        assert!(s.is_cw());
    }

    #[test]
    fn a_human_spot_of_dx_spider() {
        let s = parse_spot("DX de W3LPL:     14025.0  K1ABC       up 1                           1234Z").unwrap();
        assert_eq!((s.call.as_str(), s.freq_hz, s.skimmer), ("K1ABC", 14_025_000, false));
        assert_eq!(s.comment, "up 1");
        assert_eq!(s.mode, SpotMode::Unknown);
        assert!(s.is_cw(), "no mode named, but 14025 is in the CW part of 20 m");
    }

    #[test]
    fn digital_spots_are_not_cw() {
        let s = parse_spot("DX de DL1ABC-#:  14074.0  JA1XYZ       FT8 -12 dB                     1234Z").unwrap();
        assert_eq!(s.mode, SpotMode::Other);
        assert!(!s.is_cw());
        let s = parse_spot("DX de K1ABC:     7060.0  PA3XYZ       RTTY contest                   1234Z").unwrap();
        assert!(!s.is_cw());
    }

    #[test]
    fn unknown_mode_outside_the_cw_part_is_not_cw() {
        let s = parse_spot("DX de K1ABC:     14250.0  PA3XYZ       big signal                    1234Z").unwrap();
        assert_eq!(s.mode, SpotMode::Unknown);
        assert!(!s.is_cw());
    }

    #[test]
    fn lines_that_are_no_spots_are_ignored() {
        assert!(parse_spot("WWV de VE7CC <18>:   SFI=100").is_none());
        assert!(parse_spot("To ALL de VE7CC: hello").is_none());
        assert!(parse_spot("DX de K1ABC: abc 14025.0").is_none());
        assert!(parse_spot("DX de K1ABC:  14025.0 NOCALL").is_none(), "no digit in the call");
        assert!(parse_spot("DX de K1ABC:  0.5 K1ABC x 1234Z").is_none(), "absurd frequency");
        assert!(parse_spot("").is_none());
    }

    #[test]
    fn portable_calls_are_kept_whole() {
        let s = parse_spot("DX de K1ABC:  7020.0  ON3VZ/P  CQ  0900Z").unwrap();
        assert_eq!(s.call, "ON3VZ/P");
    }

    #[test]
    fn the_program_is_read_from_the_welcome_text() {
        assert_eq!(detect_software("Hello Kristof, this is DXSpider v1.57 at G1FEF"), Software::Spider);
        assert_eq!(detect_software("Welcome to the AR-Cluster node"), Software::Ar);
        assert_eq!(detect_software("VE7CC-1 CC Cluster Version 4.0"), Software::Cc);
        assert_eq!(detect_software("login:"), Software::Unknown);
    }

    #[test]
    fn commands_follow_the_program_and_the_skimmer_choice() {
        assert_eq!(init_commands(Software::Spider, true), vec!["set/skimmer CW"]);
        assert_eq!(init_commands(Software::Cc, false), vec!["set/noskimmer"]);
        assert!(init_commands(Software::Unknown, true).contains(&"set/skimmer".to_string()));
        assert_eq!(init_commands(Software::Ar, false), vec!["set dx filter not skimmer"]);
    }
}
