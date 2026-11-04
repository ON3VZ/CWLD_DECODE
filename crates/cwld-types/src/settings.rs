//! User settings, stored as one JSON file in the user's config folder.
//! No account, no cloud. Callsigns are filled in at first start, never hard-coded.

use crate::policy::TxPolicy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    En,
    Nl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RadioKind {
    #[default]
    Simulator,
    Icom,
    Qmx,
}

/// How to reach the radio.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RadioSettings {
    pub kind: RadioKind,
    /// Serial port name, for example `COM5` or `/dev/ttyACM0`.
    pub port: String,
    /// CI-V address of the radio. `0xB6` is the default of the IC-7300 MK II.
    pub civ_address: u8,
    /// Baud rate. For a USB serial port it hardly matters, but the Icom menu has a value.
    pub baud: u32,
    /// Send the next CW block before the radio finishes the previous one. Off until tested.
    pub pipeline: bool,
}

impl Default for RadioSettings {
    fn default() -> Self {
        RadioSettings { kind: RadioKind::Simulator, port: String::new(), civ_address: 0xB6, baud: 115_200, pipeline: false }
    }
}

/// Protection against a bad antenna: stop at once when the SWR gets too high.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SwrSettings {
    pub enabled: bool,
    /// Stop when the SWR is above this value.
    pub limit: f32,
    /// Send QRX as the last transmission after the stop. On by default, as the operator wants other
    /// stations to know. It does transmit once more into a bad antenna, so it can be switched off.
    pub qrx_after_stop: bool,
}

pub const SWR_MIN_LIMIT: f32 = 1.1;
pub const SWR_MAX_LIMIT: f32 = 3.0;

impl Default for SwrSettings {
    fn default() -> Self {
        SwrSettings { enabled: true, limit: 1.5, qrx_after_stop: true }
    }
}

impl SwrSettings {
    /// The limit, kept inside a sensible range even if the file was edited by hand.
    pub fn safe_limit(&self) -> f32 {
        if self.limit.is_nan() {
            1.5
        } else {
            self.limit.clamp(SWR_MIN_LIMIT, SWR_MAX_LIMIT)
        }
    }
}

/// Where the decoder listens.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioSettings {
    /// Name of the sound card input. Empty: a radio-looking input ("USB Audio CODEC"), else
    /// the system default.
    pub input: String,
    /// The CW pitch the decoder listens to, in Hz. Clicking the waterfall changes it.
    pub tone_hz: f32,
    /// Start listening when CWLD starts.
    pub auto_start: bool,
    /// Folder for recordings. Empty: the default folder (Documents/CWLD).
    pub record_dir: String,
}

impl Default for AudioSettings {
    fn default() -> Self {
        AudioSettings { input: String::new(), tone_hz: 700.0, auto_start: true, record_dir: String::new() }
    }
}

impl AudioSettings {
    /// The pitch, kept in a range where CW tones live, even after a hand-edited file.
    pub fn safe_tone(&self) -> f32 {
        if self.tone_hz.is_nan() {
            700.0
        } else {
            self.tone_hz.clamp(200.0, 1500.0)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Station {
    /// Callsigns of the operator, the first one is the primary callsign.
    pub calls: Vec<String>,
    pub name: String,
    pub qth: String,
    /// Maidenhead locator, 4 or 6 characters.
    pub locator: String,
}

impl Station {
    /// Checks a Maidenhead locator such as `JO20` or `JO20ee`.
    pub fn is_valid_locator(loc: &str) -> bool {
        let b = loc.as_bytes();
        if b.len() != 4 && b.len() != 6 {
            return false;
        }
        let field = |c: u8| (b'A'..=b'R').contains(&c.to_ascii_uppercase());
        let digit = |c: u8| c.is_ascii_digit();
        let sub = |c: u8| (b'A'..=b'X').contains(&c.to_ascii_uppercase());
        field(b[0]) && field(b[1]) && digit(b[2]) && digit(b[3]) && (b.len() == 4 || (sub(b[4]) && sub(b[5])))
    }

    /// Checks a callsign: letters, digits and at most one `/` suffix part, e.g. `ON3VZ/P`.
    pub fn is_valid_call(call: &str) -> bool {
        let up = call.to_ascii_uppercase();
        if up.len() < 3 || up.len() > 14 {
            return false;
        }
        let parts: Vec<&str> = up.split('/').collect();
        if parts.len() > 3 || parts.iter().any(|p| p.is_empty()) {
            return false;
        }
        up.chars().all(|c| c.is_ascii_alphanumeric() || c == '/') && up.chars().any(|c| c.is_ascii_digit())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub schema_version: u32,
    pub language: Language,
    pub theme: Theme,
    pub station: Station,
    pub radio: RadioSettings,
    pub swr: SwrSettings,
    pub audio: AudioSettings,
    /// The policy that was active when the app was last closed.
    pub last_policy: TxPolicy,
    pub first_run_done: bool,
    pub signals_panel: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            schema_version: SCHEMA_VERSION,
            language: Language::default(),
            theme: Theme::default(),
            station: Station::default(),
            radio: RadioSettings::default(),
            swr: SwrSettings::default(),
            audio: AudioSettings::default(),
            last_policy: TxPolicy::ListenOnly,
            first_run_done: false,
            signals_panel: false,
        }
    }
}

impl Settings {
    /// The policy to start with. Never `Live`, see [`TxPolicy::startup_from_last`].
    pub fn startup_policy(&self) -> TxPolicy {
        TxPolicy::startup_from_last(self.last_policy)
    }

    /// Loads settings. A missing file gives defaults. An unreadable file is kept as
    /// `<name>.bad` and defaults are used, so a user never loses data silently.
    pub fn load(path: &Path) -> Settings {
        match fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str::<Settings>(&text) {
                Ok(mut s) => {
                    s.schema_version = SCHEMA_VERSION;
                    s
                }
                Err(_) => {
                    let mut bad: PathBuf = path.to_path_buf();
                    let name = format!("{}.bad", path.file_name().and_then(|n| n.to_str()).unwrap_or("settings.json"));
                    bad.set_file_name(name);
                    let _ = fs::rename(path, bad);
                    Settings::default()
                }
            },
            Err(_) => Settings::default(),
        }
    }

    /// Saves atomically: write a temporary file, then rename it over the old one.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        let text = serde_json::to_string_pretty(self).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&tmp, text)?;
        fs::rename(&tmp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cwld-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn swr_protection_is_on_at_1_5_by_default_and_qrx_too() {
        let s = Settings::default().swr;
        assert!(s.enabled);
        assert_eq!(s.limit, 1.5);
        assert!(s.qrx_after_stop);
    }

    #[test]
    fn swr_limit_is_kept_in_range() {
        let mut s = SwrSettings::default();
        s.limit = 9.0;
        assert_eq!(s.safe_limit(), SWR_MAX_LIMIT);
        s.limit = 0.2;
        assert_eq!(s.safe_limit(), SWR_MIN_LIMIT);
        s.limit = f32::NAN;
        assert_eq!(s.safe_limit(), 1.5);
    }

    #[test]
    fn no_callsign_is_hard_coded() {
        assert!(Settings::default().station.calls.is_empty());
    }

    #[test]
    fn round_trip() {
        let d = tmpdir("rt");
        let p = d.join("settings.json");
        let mut s = Settings::default();
        s.language = Language::Nl;
        s.station.calls = vec!["ON3VZ".into()];
        s.last_policy = TxPolicy::Practice;
        s.save(&p).unwrap();
        assert_eq!(Settings::load(&p), s);
    }

    #[test]
    fn missing_file_gives_defaults() {
        let d = tmpdir("missing");
        assert_eq!(Settings::load(&d.join("nope.json")), Settings::default());
    }

    #[test]
    fn corrupt_file_is_kept_as_bad() {
        let d = tmpdir("bad");
        let p = d.join("settings.json");
        fs::write(&p, "{ this is not json").unwrap();
        assert_eq!(Settings::load(&p), Settings::default());
        assert!(d.join("settings.json.bad").exists());
    }

    #[test]
    fn unknown_and_missing_fields_are_tolerated() {
        let d = tmpdir("fwd");
        let p = d.join("settings.json");
        fs::write(&p, r#"{"language":"nl","future_field":123}"#).unwrap();
        let s = Settings::load(&p);
        assert_eq!(s.language, Language::Nl);
        assert_eq!(s.theme, Theme::Dark);
    }

    #[test]
    fn live_is_never_a_startup_policy() {
        let mut s = Settings::default();
        s.last_policy = TxPolicy::Live;
        assert_eq!(s.startup_policy(), TxPolicy::ListenOnly);
    }

    #[test]
    fn locator_and_call_validation() {
        assert!(Station::is_valid_locator("JO20"));
        assert!(Station::is_valid_locator("JO20ee"));
        assert!(Station::is_valid_locator("jo20QE"));
        assert!(!Station::is_valid_locator("JO2"));
        assert!(!Station::is_valid_locator("ZZ20"));
        assert!(!Station::is_valid_locator("JO20zz"));
        assert!(Station::is_valid_call("ON3VZ"));
        assert!(Station::is_valid_call("on3vz/p"));
        assert!(!Station::is_valid_call("ON3VZ//P"));
        assert!(!Station::is_valid_call("HELLO"));
        assert!(!Station::is_valid_call("ON"));
    }
}
