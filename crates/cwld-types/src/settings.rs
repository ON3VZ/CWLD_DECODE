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
    /// Connect to the radio by itself when CWLD starts (not for the simulator).
    pub auto_connect: bool,
}

impl Default for RadioSettings {
    fn default() -> Self {
        RadioSettings { kind: RadioKind::Simulator, port: String::new(), civ_address: 0xB6, baud: 115_200, pipeline: false, auto_connect: true }
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
    /// How clearly a signal must stand out of the noise before the decoder trusts it (the
    /// separation of key-down and key-up levels). Higher means stricter: less text out of noise,
    /// but weak signals are missed.
    pub squelch: f32,
    /// Move to the strongest signal by itself when the current pitch is quiet.
    pub auto_tone: bool,
    /// Extra gain on the incoming audio for decoding and the level meter, -20 to +20 dB. The
    /// recording stays untouched.
    pub gain_db: f32,
    /// Wide decoder filter (double bandwidth) for drifting or chirpy signals.
    pub wide_filter: bool,
    /// Slowest and fastest speed the decoder looks for, in WPM.
    pub min_wpm: u8,
    pub max_wpm: u8,
    /// Fixed speed in WPM (beacons, a station with a steady speed). 0: follow the speed by itself.
    pub lock_wpm: u8,
    /// Follow a drifting tone by itself.
    pub afc: bool,
}

pub const GAIN_MAX_DB: f32 = 20.0;
pub const WPM_FLOOR: u8 = 5;
pub const WPM_CEILING: u8 = 60;
pub const MAX_WPM_DEFAULT: u8 = 50;

/// The range of the squelch setting, and its default.
pub const SQUELCH_MIN: f32 = 1.0;
pub const SQUELCH_MAX: f32 = 8.0;
pub const SQUELCH_DEFAULT: f32 = 3.5;

/// Range and default of the wait before a frequency change starts a new tab (seconds).
pub const NEW_TAB_SECONDS_MIN: u32 = 1;
pub const NEW_TAB_SECONDS_MAX: u32 = 30;
pub const NEW_TAB_SECONDS_DEFAULT: u32 = 5;

impl Default for AudioSettings {
    fn default() -> Self {
        AudioSettings { input: String::new(), tone_hz: 700.0, auto_start: true, record_dir: String::new(), squelch: SQUELCH_DEFAULT, auto_tone: true, gain_db: 0.0, wide_filter: false, min_wpm: WPM_FLOOR, max_wpm: MAX_WPM_DEFAULT, lock_wpm: 0, afc: true }
    }
}

impl AudioSettings {
    /// The squelch, kept in range even after a hand-edited file.
    pub fn safe_squelch(&self) -> f32 {
        if self.squelch.is_nan() {
            SQUELCH_DEFAULT
        } else {
            self.squelch.clamp(SQUELCH_MIN, SQUELCH_MAX)
        }
    }

    /// The gain, kept in range even after a hand-edited file.
    pub fn safe_gain_db(&self) -> f32 {
        if self.gain_db.is_nan() {
            0.0
        } else {
            self.gain_db.clamp(-GAIN_MAX_DB, GAIN_MAX_DB)
        }
    }

    /// Slowest and fastest speed, each inside 5 to 60 WPM and at least 5 WPM apart.
    pub fn safe_wpm_range(&self) -> (u8, u8) {
        let lo = self.min_wpm.clamp(WPM_FLOOR, WPM_CEILING - 5);
        let hi = self.max_wpm.clamp(lo + 5, WPM_CEILING);
        (lo, hi)
    }

    /// The fixed speed, or `None` for automatic.
    pub fn safe_lock_wpm(&self) -> Option<u8> {
        (self.lock_wpm >= WPM_FLOOR).then(|| self.lock_wpm.min(WPM_CEILING))
    }

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
    /// Seconds the frequency must stay put before the app starts a new conversation (tab).
    pub new_tab_seconds: u32,
    /// Show the radio's own spectrum scope (band scope), when the radio can send it.
    pub band_scope: bool,
    /// Show the narrow audio waterfall (what the decoder hears).
    pub audio_waterfall: bool,
    /// The signal report used in chips: `5NN` (default) or `599`.
    pub default_rst: DefaultRst,
    /// Close conversations without activity after this many minutes. 0: never.
    pub tabs_auto_close_minutes: u32,
    /// Contest mode: the serial number `{NR}` is filled in and counted.
    pub contest_mode: bool,
    /// The next serial number in contest mode.
    pub contest_nr: u32,
    /// Words and abbreviations the operator added to the code library.
    pub user_words: Vec<UserWord>,
    /// Contest profiles of the operator, the active profile and own variables.
    pub contest: ContestSettings,
    /// Stations the operator uses (home, portable). The first call of `station` is the base call.
    pub profiles: Vec<StationProfile>,
    /// Id of the station profile in use. Never changed silently.
    pub profile: String,
    /// Height of the band scope outside contest mode and in contest mode.
    pub scope_size: ScopeSize,
    pub scope_size_contest: ScopeSize,
    /// The last choices in the practice panel.
    pub practice: PracticeSettings,
    /// Sending logged QSOs to other programs over UDP (Log4OM, N1MM format).
    pub link: LinkSettings,
    /// Callsign lookup (QRZ.com). The password is never part of the settings: it is kept by the OS.
    pub lookup: LookupSettings,
}

/// QRZ.com XML lookup. Off by default; the password lives in the operating system's credential store.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LookupSettings {
    pub qrz_enabled: bool,
    /// The QRZ.com login name (normally the operator's callsign).
    pub qrz_user: String,
}

/// Where logged QSOs are sent over UDP. Everything is off by default: a logger only gets QSOs when the operator asks for it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LinkSettings {
    /// Send every real QSO as one ADIF record in one datagram (Log4OM, service type ADIF_MESSAGE).
    pub adif_enabled: bool,
    pub adif_host: String,
    pub adif_port: u16,
    /// Send N1MM Logger+ style XML messages (contactinfo, contactreplace, contactdelete).
    pub n1mm_enabled: bool,
    /// Destinations as `host:port`, separated by spaces. N1MM's own default port is 12060.
    pub n1mm_targets: String,
}

pub const LINK_ADIF_PORT_DEFAULT: u16 = 2333;
pub const LINK_N1MM_TARGET_DEFAULT: &str = "127.0.0.1:12060";

impl Default for LinkSettings {
    fn default() -> Self {
        LinkSettings {
            adif_enabled: false,
            adif_host: "127.0.0.1".into(),
            adif_port: LINK_ADIF_PORT_DEFAULT,
            n1mm_enabled: false,
            n1mm_targets: LINK_N1MM_TARGET_DEFAULT.into(),
        }
    }
}

/// Choices of the practice simulator. Only used by the interface; kept in range there.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PracticeSettings {
    /// "they_cq", "you_cq" or "contest".
    pub exercise: String,
    pub wpm_min: u8,
    pub wpm_max: u8,
    /// "weak", "medium" or "strong".
    pub strength: String,
    pub noise: bool,
    pub qsb: bool,
    /// Most stations that call at once in the contest exercise.
    pub pileup: u8,
    /// Length of the contest exercise in minutes, 0 = until you stop.
    pub minutes: u16,
    /// Show the decoded text (off: train your ear).
    pub decoder_help: bool,
    /// Loudness on the computer's output, 0 to 1.
    pub volume: f32,
    /// In practice mode: the radio's speaker is turned down and break-in switched off, and both
    /// are put back when practice mode ends. The paddle then only makes the sidetone.
    pub quiet_radio: bool,
    /// Results of the paddle exercises.
    pub paddle: PaddleProgress,
}

/// What the paddle trainer remembers between sessions. Kept by the interface.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaddleProgress {
    /// The last exercise chosen: "chars", "weak", "words", "calls" or "phrases".
    pub exercise: String,
    pub stats: Vec<CharStat>,
    pub sessions: Vec<PaddleSession>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CharStat {
    pub ch: String,
    pub tries: u32,
    pub errors: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaddleSession {
    /// Seconds since 1970, UTC.
    pub ts: i64,
    pub exercise: String,
    /// 0 to 100.
    pub score: u8,
    pub wpm: f32,
    /// Share of characters keyed right, 0 to 1.
    pub accuracy: f32,
}

impl Default for PracticeSettings {
    fn default() -> Self {
        PracticeSettings { exercise: "they_cq".into(), wpm_min: 18, wpm_max: 26, strength: "medium".into(), noise: true, qsb: false, pileup: 3, minutes: 10, decoder_help: true, volume: 0.5, quiet_radio: true, paddle: PaddleProgress::default() }
    }
}

/// Height of the band scope on the main screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ScopeSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// A station the operator transmits from. Callsign on the air = base call + `suffix`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct StationProfile {
    pub id: String,
    pub name: String,
    /// Added to the base call: empty at home, `/P` portable.
    pub suffix: String,
    /// Maidenhead locator. Empty: the home locator of the station, or the one of the activation.
    pub grid: String,
    /// Output power in watts as written in the log, for example `5`.
    pub power: String,
    pub rig: String,
    pub antenna: String,
    /// Written in the COMMENT field of every QSO, for example `5W QRP`.
    pub comment: String,
}

impl StationProfile {
    /// The callsign on the air for a base call.
    pub fn on_air(&self, base: &str) -> String {
        format!("{}{}", base.trim().to_uppercase(), self.suffix.trim().to_uppercase())
    }
}

/// The two profiles a new installation starts with.
pub fn default_profiles() -> Vec<StationProfile> {
    vec![
        StationProfile { id: "home".into(), name: "Home".into(), ..Default::default() },
        StationProfile { id: "portable".into(), name: "Portable".into(), suffix: "/P".into(), ..Default::default() },
    ]
}

/// An own variable for message templates, for example `SEC` = `WLD`. Used as `{SEC}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ContestVar {
    pub name: String,
    pub value: String,
}

/// One way to send the exchange, for example from Belgium or from abroad.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SentTemplate {
    pub label: String,
    pub template: String,
}

/// The one extra field that is received from the other station (logged from the logbook on).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RecvField {
    /// Short name, for example `SEC`.
    pub name: String,
    pub label: String,
    /// `text`, `number` or `choice`.
    pub kind: String,
    pub choices: Vec<String>,
    /// The field applies only to stations whose call starts with one of these. Empty: always.
    pub only_prefixes: Vec<String>,
}

/// A contest profile of the operator (the built-in ones live in the interface).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ContestProfile {
    pub id: String,
    pub name: String,
    pub note: String,
    pub calls: Vec<String>,
    pub sent: Vec<SentTemplate>,
    pub recv: Option<RecvField>,
    /// Send the serial number as cut numbers (0 as T, 9 as N).
    pub cut_numbers: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ContestSettings {
    /// Id of the active profile, a built-in one or one of `profiles`. Empty: none.
    pub active: String,
    /// Which of the sent templates of the active profile is used.
    pub sent_index: usize,
    pub profiles: Vec<ContestProfile>,
    pub vars: Vec<ContestVar>,
    /// Which run of the contest this is. A new contest raises it, so dupes and the serial
    /// number start again. Starts at 1.
    pub session: u32,
}

impl Default for ContestSettings {
    fn default() -> Self {
        let v = |n: &str, x: &str| ContestVar { name: n.into(), value: x.into() };
        ContestSettings {
            active: String::new(),
            sent_index: 0,
            session: 1,
            profiles: Vec::new(),
            // Belgium: CQ zone 14, ITU zone 27. The section and power are the operator's own.
            vars: vec![v("SEC", "WLD"), v("ZONE", "14"), v("ITU", "27"), v("PWR", "20")],
        }
    }
}

/// An own abbreviation with its meaning, shown in tooltips and in the code library.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UserWord {
    pub code: String,
    pub meaning: String,
}

pub const TABS_AUTO_CLOSE_DEFAULT: u32 = 15;
pub const TABS_AUTO_CLOSE_MAX: u32 = 240;

/// How the signal report is sent: cut number `5NN` or in full `599`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DefaultRst {
    #[default]
    #[serde(rename = "5NN")]
    Nn,
    #[serde(rename = "599")]
    Full,
}

impl DefaultRst {
    pub fn text(self) -> &'static str {
        match self {
            DefaultRst::Nn => "5NN",
            DefaultRst::Full => "599",
        }
    }
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
            new_tab_seconds: NEW_TAB_SECONDS_DEFAULT,
            band_scope: true,
            audio_waterfall: true,
            default_rst: DefaultRst::default(),
            tabs_auto_close_minutes: TABS_AUTO_CLOSE_DEFAULT,
            contest_mode: false,
            contest_nr: 1,
            user_words: Vec::new(),
            contest: ContestSettings::default(),
            profiles: default_profiles(),
            profile: "home".into(),
            scope_size: ScopeSize::Medium,
            scope_size_contest: ScopeSize::Small,
            practice: PracticeSettings::default(),
            link: LinkSettings::default(),
            lookup: LookupSettings::default(),
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

    #[test]
    fn report_is_5nn_by_default_and_old_files_still_load() {
        let s: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(s.default_rst, DefaultRst::Nn);
        assert_eq!(s.default_rst.text(), "5NN");
        assert_eq!(s.tabs_auto_close_minutes, 15);
        assert!(!s.contest_mode);
        let s: Settings = serde_json::from_str(r#"{"default_rst":"599"}"#).unwrap();
        assert_eq!(s.default_rst.text(), "599");
    }

    #[test]
    fn decoder_tuning_is_kept_in_range() {
        let mut a = AudioSettings::default();
        assert_eq!(a.safe_gain_db(), 0.0);
        assert_eq!(a.safe_wpm_range(), (5, 50));
        assert_eq!(a.safe_lock_wpm(), None);
        a.gain_db = 99.0;
        assert_eq!(a.safe_gain_db(), 20.0);
        a.gain_db = f32::NAN;
        assert_eq!(a.safe_gain_db(), 0.0);
        a.min_wpm = 58;
        a.max_wpm = 10;
        let (lo, hi) = a.safe_wpm_range();
        assert!(lo <= 55 && hi >= lo + 5 && hi <= 60);
        a.lock_wpm = 4;
        assert_eq!(a.safe_lock_wpm(), None);
        a.lock_wpm = 99;
        assert_eq!(a.safe_lock_wpm(), Some(60));
    }

    #[test]
    fn a_new_tab_waits_five_seconds_and_an_old_file_still_loads() {
        assert_eq!(Settings::default().new_tab_seconds, 5);
        let s: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(s.new_tab_seconds, NEW_TAB_SECONDS_DEFAULT);
    }
    #[test]
    fn station_profiles_and_scope_sizes_have_defaults_and_an_old_file_still_loads() {
        let s: Settings = serde_json::from_str(r#"{"contest_mode":true}"#).unwrap();
        assert_eq!(s.profiles.len(), 2);
        assert_eq!(s.profile, "home");
        assert_eq!(s.scope_size, ScopeSize::Medium);
        assert_eq!(s.scope_size_contest, ScopeSize::Small);
        assert_eq!(s.profiles[1].on_air("on3vz"), "ON3VZ/P");
        assert_eq!(s.profiles[0].on_air("ON3VZ"), "ON3VZ");
    }

    #[test]
    fn contest_defaults_have_own_variables_and_an_old_file_still_loads() {
        let s: Settings = serde_json::from_str(r#"{"contest_mode":true,"contest_nr":7}"#).unwrap();
        assert_eq!(s.contest_nr, 7);
        assert!(s.contest.active.is_empty());
        let get = |n: &str| s.contest.vars.iter().find(|v| v.name == n).map(|v| v.value.clone());
        assert_eq!(get("SEC").as_deref(), Some("WLD"));
        assert_eq!(get("ZONE").as_deref(), Some("14"));
        assert_eq!(get("ITU").as_deref(), Some("27"));
        assert_eq!(get("PWR").as_deref(), Some("20"));
    }

    #[test]
    fn contest_profiles_survive_a_round_trip() {
        let mut s = Settings::default();
        s.contest.active = "mine".into();
        s.contest.sent_index = 1;
        s.contest.profiles.push(ContestProfile {
            id: "mine".into(),
            name: "Club".into(),
            note: "n".into(),
            calls: vec!["TEST CLUB".into()],
            sent: vec![SentTemplate { label: "a".into(), template: "{CALL} {RST} {NR}".into() }],
            recv: Some(RecvField { name: "AGE".into(), label: "Age".into(), kind: "number".into(), choices: vec![], only_prefixes: vec!["ON".into()] }),
            cut_numbers: true,
        });
        let text = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&text).unwrap();
        assert_eq!(back.contest, s.contest);
    }
}
