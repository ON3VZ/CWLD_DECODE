//! Application core. The Tauri shell is a thin layer on top of [`AppCore`], so all rules
//! (mode switching, the transmit path, validation) are testable without a window.

pub mod audio;

pub use audio::{AudioStatus, DecodeSource};
pub use cwld_audio::{InputDevice, Source as AudioSource, TentativeOut};

use cwld_radio::{
    list_ports, GuardedRig, IcomRig, NullRig, PortInfo, QmxRig, Rig, RigError, SerialTransport, SimRig, SystemClock,
};
use cwld_types::{morse, PolicyHandle, RadioCommand, RadioKind, RadioSettings, RadioStatus, RigEvent, Settings, TxPolicy};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Frequency range accepted by the tuning command (Hz).
pub const MIN_FREQ_HZ: u64 = 30_000;
pub const MAX_FREQ_HZ: u64 = 74_800_000;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CoreError {
    #[error("switching to live needs confirmation")]
    ConfirmationRequired,
    #[error("sending is not possible in this mode")]
    NotLive,
    #[error("sending is blocked after an SWR alert")]
    SwrLatched,
    #[error("frequency out of range")]
    FrequencyOutOfRange,
    #[error("{0}")]
    Rig(RigError),
    #[error("settings could not be saved: {0}")]
    Settings(String),
    #[error("audio: {0}")]
    Audio(String),
}

impl CoreError {
    /// Stable code for the interface (it translates the message itself).
    pub fn code(&self) -> &'static str {
        match self {
            CoreError::ConfirmationRequired => "confirmation_required",
            CoreError::NotLive => "not_live",
            CoreError::SwrLatched => "swr_latched",
            CoreError::FrequencyOutOfRange => "frequency_out_of_range",
            CoreError::Rig(e) => e.code(),
            CoreError::Settings(_) => "settings",
            CoreError::Audio(_) => "audio",
        }
    }
}

impl From<RigError> for CoreError {
    fn from(e: RigError) -> Self {
        CoreError::Rig(e)
    }
}

/// What happened to a text the operator asked to send.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SendOutcome {
    /// Handed to the radio keyer (Live).
    Sent,
    /// Practice: nothing left the application. `seconds` is how long it would have taken.
    Simulated { seconds: f64 },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Snapshot {
    pub policy: TxPolicy,
    pub swr_alert: Option<SwrAlert>,
    pub status: RadioStatus,
    pub radio_name: String,
    pub settings: Settings,
    pub audio: AudioStatus,
}

/// The SWR protection has stopped the radio. Sending stays blocked until the operator
/// acknowledges the alert, so nobody keys into a bad antenna by reflex.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SwrAlert {
    pub swr: f32,
    pub limit: f32,
    pub qrx_sent: bool,
}

/// Number of readings in a row above the limit before the protection acts. Readings come about
/// every 100 ms, so this is roughly 200 ms: one noisy reading at key-down does not trip it.
pub const SWR_READINGS_TO_TRIP: u8 = 2;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CoreEvent {
    SwrAlert { alert: SwrAlert },
    SwrCleared,
    Rig { event: RigEvent },
    Policy { policy: TxPolicy },
    /// Decoder certainty for the meter in the status bar.
    Decoder { stats: cwld_types::DecoderStats },
    /// Decoded text: new final characters with their confidence, and the uncertain tail.
    Decode { source: DecodeSource, text: String, conf: Vec<f32>, tentative: Vec<TentativeOut>, wpm: f32 },
    /// One waterfall row, 0..255 per bin, starting at `lo_hz`.
    Waterfall { bins: Vec<u8>, lo_hz: f64, hz_per_bin: f64 },
    /// Input level, dB below full scale.
    AudioLevel { dbfs: f32, peak: f32 },
    /// The bandwidth of the decoder filter in Hz, for the lines in the audio waterfall.
    Bandwidth { hz: f32 },
    /// Audio started, stopped, failed, or a recording changed.
    Audio { status: AudioStatus },
}

pub struct AppCore {
    policy: PolicyHandle,
    rig: Mutex<GuardedRig<Box<dyn Rig>>>,
    settings: Mutex<Settings>,
    settings_path: Option<PathBuf>,
    swr: Mutex<SwrState>,
    audio: Mutex<audio::AudioState>,
    tx_track: Mutex<audio::TxTrack>,
    recordings_dir: Mutex<Option<PathBuf>>,
    link: Mutex<LinkState>,
}

/// Watches the link to a real radio, so a switched-off radio or a pulled cable shows as
/// not connected and CWLD tries again by itself.
#[derive(Default)]
struct LinkState {
    /// A real radio was opened and the operator did not disconnect it.
    wanted: bool,
    /// The radio stopped answering or the port failed.
    lost: bool,
    last_try: Option<Instant>,
}

/// Pause between two tries to open the radio again.
pub const RECONNECT_EVERY: Duration = Duration::from_secs(5);

#[derive(Default)]
struct SwrState {
    over: u8,
    alert: Option<SwrAlert>,
}

impl AppCore {
    /// Creates the core. `settings_path` is where settings are loaded from and saved to.
    pub fn new(settings_path: Option<PathBuf>) -> Self {
        let settings = settings_path.as_deref().map(Settings::load).unwrap_or_default();
        let policy = PolicyHandle::new(settings.startup_policy());
        // A simulator starts at once. A real radio waits for `connect`, and until then
        // nothing is connected (no fake data that looks real).
        let first: Box<dyn Rig> = match settings.radio.kind {
            RadioKind::Simulator => Box::new(SimRig::new()),
            _ => Box::new(NullRig::new()),
        };
        let mut rig = GuardedRig::new(first, policy.clone());
        if settings.band_scope && rig.capabilities().scope {
            let _ = rig.execute(&RadioCommand::SetScope(true));
        }
        AppCore {
            policy,
            rig: Mutex::new(rig),
            settings: Mutex::new(settings),
            settings_path,
            swr: Mutex::new(SwrState::default()),
            audio: Mutex::new(audio::AudioState::default()),
            tx_track: Mutex::new(audio::TxTrack::default()),
            recordings_dir: Mutex::new(None),
            link: Mutex::new(LinkState::default()),
        }
    }

    pub fn list_ports(&self) -> Vec<PortInfo> {
        list_ports()
    }

    /// Opens the radio described by `cfg` and makes it the active one. The old radio is
    /// stopped first, so nothing keeps sending. On failure the old radio stays.
    pub fn connect(&self, cfg: &RadioSettings) -> Result<(), CoreError> {
        let new: Box<dyn Rig> = match cfg.kind {
            RadioKind::Simulator => Box::new(SimRig::new()),
            RadioKind::Icom | RadioKind::Qmx => {
                if cfg.port.trim().is_empty() {
                    return Err(RigError::InvalidArgument("port".into()).into());
                }
                let t = SerialTransport::open(cfg.port.trim(), cfg.baud).map_err(|e| RigError::Io(e.to_string()))?;
                if cfg.kind == RadioKind::Icom {
                    Box::new(IcomRig::new(t, Box::new(SystemClock::new()), cfg.civ_address, cfg.pipeline))
                } else {
                    Box::new(QmxRig::new(t, Box::new(SystemClock::new())))
                }
            }
        };
        let mut rig = self.rig.lock().unwrap();
        let _ = rig.execute(&RadioCommand::StopCw);
        let _ = rig.execute(&RadioCommand::SetScope(false));
        *rig.inner_mut() = new;
        if self.settings.lock().unwrap().band_scope && rig.capabilities().scope {
            let _ = rig.execute(&RadioCommand::SetScope(true));
        }
        drop(rig);
        {
            let mut link = self.link.lock().unwrap();
            link.wanted = cfg.kind != RadioKind::Simulator;
            link.lost = false;
        }
        self.settings.lock().unwrap().radio = cfg.clone();
        self.save_settings()
    }

    /// Connects to the saved radio when CWLD starts. `None`: nothing to do (setting off,
    /// simulator, or no port chosen yet).
    pub fn auto_connect(&self) -> Option<Result<(), CoreError>> {
        let cfg = self.settings.lock().unwrap().radio.clone();
        if !cfg.auto_connect || cfg.kind == RadioKind::Simulator || cfg.port.trim().is_empty() {
            return None;
        }
        Some(self.connect(&cfg))
    }

    /// Lets go of everything before the program ends or an update is installed: stops sending,
    /// switches the scope off, closes the radio port, stops recording and the sound card.
    pub fn prepare_exit(&self) {
        self.stop_recording();
        self.stop_audio();
        self.disconnect();
    }

    /// Lets go of the radio (the port is closed when the driver is dropped).
    pub fn disconnect(&self) {
        *self.link.lock().unwrap() = LinkState::default();
        let mut rig = self.rig.lock().unwrap();
        let _ = rig.execute(&RadioCommand::StopCw);
        let _ = rig.execute(&RadioCommand::SetScope(false));
        *rig.inner_mut() = Box::new(NullRig::new());
    }

    pub fn policy(&self) -> TxPolicy {
        self.policy.get()
    }

    pub fn snapshot(&self) -> Snapshot {
        let rig = self.rig.lock().unwrap();
        Snapshot {
            policy: self.policy.get(),
            swr_alert: self.swr.lock().unwrap().alert.clone(),
            status: rig.status(),
            radio_name: rig.info().name,
            settings: self.settings.lock().unwrap().clone(),
            audio: self.audio_status(),
        }
    }

    /// Changes the mode. Going to `Live` needs `confirmed = true`; leaving `Live` never does.
    /// Leaving `Live` stops anything that is being sent.
    pub fn set_policy(&self, policy: TxPolicy, confirmed: bool) -> Result<(), CoreError> {
        let current = self.policy.get();
        if policy == TxPolicy::Live && current != TxPolicy::Live && !confirmed {
            return Err(CoreError::ConfirmationRequired);
        }
        if current == TxPolicy::Live && policy != TxPolicy::Live {
            // Stop is always allowed by the guard, also in the new mode.
            let _ = self.rig.lock().unwrap().execute(&RadioCommand::StopCw);
        }
        self.policy.set(policy);
        let mut s = self.settings.lock().unwrap();
        s.last_policy = policy;
        Ok(())
    }

    /// Sends text. Only `Live` reaches the radio. `Practice` simulates, `ListenOnly` refuses.
    pub fn send_text(&self, text: &str) -> Result<SendOutcome, CoreError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(RigError::InvalidArgument("empty text".into()).into());
        }
        match self.policy.get() {
            TxPolicy::Live => {
                if self.swr.lock().unwrap().alert.is_some() {
                    return Err(CoreError::SwrLatched);
                }
                let wpm = {
                    let mut rig = self.rig.lock().unwrap();
                    rig.execute(&RadioCommand::SendCw(text.to_string()))?;
                    rig.status().cw_wpm
                };
                self.note_app_transmission(text, wpm);
                Ok(SendOutcome::Sent)
            }
            TxPolicy::Practice => {
                let wpm = self.rig.lock().unwrap().status().cw_wpm.max(1);
                Ok(SendOutcome::Simulated { seconds: morse::duration_seconds(text, wpm).unwrap_or(0.0) })
            }
            TxPolicy::ListenOnly => Err(CoreError::NotLive),
        }
    }

    /// Emergency stop. Always allowed.
    pub fn stop_tx(&self) -> Result<(), CoreError> {
        self.rig.lock().unwrap().execute(&RadioCommand::StopCw)?;
        Ok(())
    }

    pub fn set_speed(&self, wpm: u8) -> Result<(), CoreError> {
        self.rig.lock().unwrap().execute(&RadioCommand::SetCwSpeed(wpm))?;
        Ok(())
    }

    /// Tuning is allowed in every mode, also while only listening.
    pub fn set_frequency(&self, hz: u64) -> Result<(), CoreError> {
        if !(MIN_FREQ_HZ..=MAX_FREQ_HZ).contains(&hz) {
            return Err(CoreError::FrequencyOutOfRange);
        }
        self.rig.lock().unwrap().execute(&RadioCommand::SetFrequency(hz))?;
        Ok(())
    }

    /// Called about every 500 ms by the shell.
    pub fn tick(&self) -> Vec<CoreEvent> {
        let events = self.rig.lock().unwrap().poll();
        let out = self.process(events);
        if self.reconnect_due(Instant::now()) {
            // The result is not an error for the operator: the next try follows in a few seconds.
            let cfg = self.settings.lock().unwrap().radio.clone();
            let _ = self.reopen(&cfg);
        }
        out
    }

    /// Updates the link state from the newest events and says whether it is time to open the
    /// radio again. Only for a real radio that the operator has not disconnected, and only when
    /// "connect automatically" is on.
    fn reconnect_due(&self, now: Instant) -> bool {
        let mut link = self.link.lock().unwrap();
        if !link.wanted || !link.lost {
            return false;
        }
        let cfg = self.settings.lock().unwrap().radio.clone();
        if !cfg.auto_connect || cfg.kind == RadioKind::Simulator || cfg.port.trim().is_empty() {
            return false;
        }
        match link.last_try {
            Some(t) if now.duration_since(t) < RECONNECT_EVERY => false,
            _ => {
                link.last_try = Some(now);
                true
            }
        }
    }

    /// Opens the radio port again without touching the link state (keeps "wanted" and "lost"
    /// until the radio answers).
    fn reopen(&self, cfg: &RadioSettings) -> Result<(), CoreError> {
        let (wanted, lost, last) = {
            let l = self.link.lock().unwrap();
            (l.wanted, l.lost, l.last_try)
        };
        let r = self.connect(cfg);
        let mut l = self.link.lock().unwrap();
        l.wanted = wanted;
        l.lost = lost;
        l.last_try = last;
        r
    }

    /// Called about every 100 ms: safety readings only (SWR).
    pub fn fast_tick(&self) -> Vec<CoreEvent> {
        let events = self.rig.lock().unwrap().fast_poll();
        self.process(events)
    }

    /// Turns radio events into core events and runs the SWR protection on the way.
    fn process(&self, events: Vec<RigEvent>) -> Vec<CoreEvent> {
        let mut out = Vec::new();
        for event in events {
            let swr = if let RigEvent::Swr(v) = &event { Some(*v) } else { None };
            if let RigEvent::Connected(up) = &event {
                self.link.lock().unwrap().lost = !*up;
            }
            out.push(CoreEvent::Rig { event });
            if let Some(v) = swr {
                if let Some(alert) = self.check_swr(v) {
                    out.push(CoreEvent::SwrAlert { alert });
                }
            }
        }
        out
    }

    /// Looks at one SWR reading. Returns an alert when the protection has just acted.
    fn check_swr(&self, swr: f32) -> Option<SwrAlert> {
        let cfg = self.settings.lock().unwrap().swr.clone();
        let mut st = self.swr.lock().unwrap();
        if !cfg.enabled || st.alert.is_some() {
            st.over = 0;
            return None;
        }
        let limit = cfg.safe_limit();
        if swr > limit {
            st.over = st.over.saturating_add(1);
        } else {
            st.over = 0;
        }
        if st.over < SWR_READINGS_TO_TRIP {
            return None;
        }
        // Stop first, always. Stop is allowed in every mode.
        let mut rig = self.rig.lock().unwrap();
        let _ = rig.execute(&RadioCommand::StopCw);
        // Optional: one short QRX, only in Live and only when the operator asked for it.
        let qrx_sent = cfg.qrx_after_stop && self.policy.get() == TxPolicy::Live && rig.execute(&RadioCommand::SendCw("QRX".into())).is_ok();
        let alert = SwrAlert { swr, limit, qrx_sent };
        st.alert = Some(alert.clone());
        st.over = 0;
        Some(alert)
    }

    /// The operator has seen the alert and checked the antenna. Sending is possible again.
    pub fn ack_swr(&self) -> Vec<CoreEvent> {
        let mut st = self.swr.lock().unwrap();
        let had = st.alert.take().is_some();
        st.over = 0;
        if had { vec![CoreEvent::SwrCleared] } else { Vec::new() }
    }

    /// Test and demo helper: the simulated radio reports this SWR while it transmits.
    pub fn simulate_swr(&self, swr: f32) {
        self.rig.lock().unwrap().simulate_swr(swr);
    }

    pub fn settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn update_settings(&self, new: Settings) -> Result<(), CoreError> {
        // New own callsigns, name or QTH: tell the decoder's word model.
        let mut words: Vec<String> = new.station.calls.clone();
        words.extend(new.station.name.split_whitespace().map(|w| w.to_string()));
        words.extend(new.station.qth.split_whitespace().map(|w| w.to_string()));
        if let Some(e) = &self.audio.lock().unwrap().engine_ref() {
            e.add_words(words);
        }
        let audio_cfg = new.audio.clone();
        let scope = new.band_scope;
        let was = std::mem::replace(&mut *self.settings.lock().unwrap(), new).band_scope;
        if scope != was {
            let mut rig = self.rig.lock().unwrap();
            if rig.capabilities().scope {
                let _ = rig.execute(&RadioCommand::SetScope(scope));
            }
        }
        if let Some(e) = &self.audio.lock().unwrap().engine_ref() {
            e.set_tuning(audio::tuning_of(&audio_cfg));
        }
        self.save_settings()
    }

    pub fn save_settings(&self) -> Result<(), CoreError> {
        match &self.settings_path {
            Some(p) => self.settings.lock().unwrap().save(p).map_err(|e| CoreError::Settings(e.to_string())),
            None => Ok(()),
        }
    }

    /// Diagnostics: transmit commands that the guard passed on to the radio.
    pub fn transmit_commands_reached_radio(&self) -> u64 {
        self.rig.lock().unwrap().passed_transmit_count()
    }

    /// Simulates the operator turning the knob on the radio.
    pub fn simulate_knob(&self, hz: u64) {
        self.rig.lock().unwrap().simulate_knob(hz);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("cwld-core-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d.join("settings.json")
    }

    #[test]
    fn a_lost_radio_is_opened_again_every_five_seconds_and_a_disconnect_stops_it() {
        let c = AppCore::new(None);
        c.settings.lock().unwrap().radio = RadioSettings { kind: RadioKind::Icom, port: "/dev/definitely-not-here".into(), ..RadioSettings::default() };
        let t0 = Instant::now();
        // Nothing wanted yet: no tries.
        assert!(!c.reconnect_due(t0));
        {
            let mut l = c.link.lock().unwrap();
            l.wanted = true;
        }
        // Still connected as far as we know: no tries.
        assert!(!c.reconnect_due(t0));
        // The radio stops answering.
        c.process(vec![RigEvent::Connected(false)]);
        assert!(c.reconnect_due(t0));
        assert!(!c.reconnect_due(t0 + Duration::from_secs(2)));
        assert!(c.reconnect_due(t0 + Duration::from_secs(6)));
        // The radio answers again.
        c.process(vec![RigEvent::Connected(true)]);
        assert!(!c.reconnect_due(t0 + Duration::from_secs(20)));
        // Lost again, then the operator disconnects: no more tries.
        c.process(vec![RigEvent::Connected(false)]);
        c.disconnect();
        assert!(!c.reconnect_due(t0 + Duration::from_secs(40)));
        // Switched off in the settings: no tries either.
        c.link.lock().unwrap().wanted = true;
        c.link.lock().unwrap().lost = true;
        c.settings.lock().unwrap().radio.auto_connect = false;
        assert!(!c.reconnect_due(t0 + Duration::from_secs(60)));
    }

    #[test]
    fn auto_connect_only_for_a_saved_real_radio() {
        let c = AppCore::new(None);
        // Simulator: nothing to do.
        assert!(c.auto_connect().is_none());
        // A real radio without a port: nothing to do.
        c.settings.lock().unwrap().radio = RadioSettings { kind: RadioKind::Icom, ..RadioSettings::default() };
        assert!(c.auto_connect().is_none());
        // A port that is not there: tried, and an error comes back (the app starts anyway).
        c.settings.lock().unwrap().radio = RadioSettings { kind: RadioKind::Qmx, port: "/dev/definitely-not-here".into(), ..RadioSettings::default() };
        assert!(matches!(c.auto_connect(), Some(Err(_))));
        // Switched off: not tried.
        c.settings.lock().unwrap().radio.auto_connect = false;
        assert!(c.auto_connect().is_none());
        assert!(RadioSettings::default().auto_connect);
    }

    #[test]
    fn prepare_exit_lets_go_of_the_radio() {
        let c = AppCore::new(None);
        c.connect(&RadioSettings::default()).unwrap();
        assert!(c.snapshot().status.connected);
        c.prepare_exit();
        assert!(!c.snapshot().status.connected);
    }

    #[test]
    fn starts_listening_by_default() {
        assert_eq!(AppCore::new(None).policy(), TxPolicy::ListenOnly);
    }

    #[test]
    fn live_needs_confirmation() {
        let c = AppCore::new(None);
        assert_eq!(c.set_policy(TxPolicy::Live, false).unwrap_err().code(), "confirmation_required");
        assert_eq!(c.policy(), TxPolicy::ListenOnly);
        c.set_policy(TxPolicy::Live, true).unwrap();
        assert_eq!(c.policy(), TxPolicy::Live);
    }

    #[test]
    fn leaving_live_needs_no_confirmation() {
        let c = AppCore::new(None);
        c.set_policy(TxPolicy::Live, true).unwrap();
        c.set_policy(TxPolicy::Practice, false).unwrap();
        assert_eq!(c.policy(), TxPolicy::Practice);
    }

    #[test]
    fn nothing_transmits_outside_live() {
        let c = AppCore::new(None);
        assert_eq!(c.send_text("CQ CQ DE ON3VZ").unwrap_err(), CoreError::NotLive);
        c.set_policy(TxPolicy::Practice, false).unwrap();
        assert!(matches!(c.send_text("CQ").unwrap(), SendOutcome::Simulated { .. }));
        assert_eq!(c.transmit_commands_reached_radio(), 0);
    }

    #[test]
    fn live_sends_to_radio() {
        let c = AppCore::new(None);
        c.set_policy(TxPolicy::Live, true).unwrap();
        assert_eq!(c.send_text("CQ").unwrap(), SendOutcome::Sent);
        assert_eq!(c.transmit_commands_reached_radio(), 1);
    }

    #[test]
    fn empty_text_is_refused_in_every_mode() {
        let c = AppCore::new(None);
        for p in [TxPolicy::ListenOnly, TxPolicy::Practice] {
            c.set_policy(p, false).unwrap();
            assert!(c.send_text("   ").is_err());
        }
    }

    #[test]
    fn stop_is_always_allowed() {
        let c = AppCore::new(None);
        assert!(c.stop_tx().is_ok());
    }

    #[test]
    fn tuning_works_while_listening_and_is_validated() {
        let c = AppCore::new(None);
        c.set_frequency(14_030_000).unwrap();
        assert_eq!(c.snapshot().status.frequency_hz, 14_030_000);
        assert_eq!(c.set_frequency(5).unwrap_err().code(), "frequency_out_of_range");
    }

    #[test]
    fn knob_turn_is_reported() {
        let c = AppCore::new(None);
        c.tick();
        c.simulate_knob(7_030_000);
        let ev = c.tick();
        assert!(ev.contains(&CoreEvent::Rig { event: RigEvent::Frequency(7_030_000) }));
    }

    #[test]
    fn startup_after_live_is_listen_only() {
        let p = tmp("startup");
        let c = AppCore::new(Some(p.clone()));
        c.set_policy(TxPolicy::Live, true).unwrap();
        c.save_settings().unwrap();
        assert_eq!(AppCore::new(Some(p)).policy(), TxPolicy::ListenOnly);
    }

    #[test]
    fn startup_restores_practice() {
        let p = tmp("practice");
        let c = AppCore::new(Some(p.clone()));
        c.set_policy(TxPolicy::Practice, false).unwrap();
        c.save_settings().unwrap();
        assert_eq!(AppCore::new(Some(p)).policy(), TxPolicy::Practice);
    }

    #[test]
    fn settings_persist() {
        let p = tmp("persist");
        let c = AppCore::new(Some(p.clone()));
        let mut s = c.settings();
        s.station.calls = vec!["ON3VZ".into()];
        c.update_settings(s).unwrap();
        assert_eq!(AppCore::new(Some(p)).settings().station.calls, vec!["ON3VZ".to_string()]);
    }
}

#[cfg(test)]
mod radio_tests {
    use super::*;

    #[test]
    fn a_real_radio_setting_starts_disconnected_not_simulated() {
        let d = std::env::temp_dir().join(format!("cwld-core-radio-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("settings.json");
        let mut s = Settings::default();
        s.radio.kind = RadioKind::Icom;
        s.radio.port = "COM9".into();
        s.save(&p).unwrap();
        let c = AppCore::new(Some(p));
        let snap = c.snapshot();
        assert!(!snap.status.connected);
        assert_eq!(snap.status.frequency_hz, 0);
    }

    #[test]
    fn connecting_without_a_port_is_refused_and_keeps_the_old_radio() {
        let c = AppCore::new(None);
        let cfg = RadioSettings { kind: RadioKind::Icom, ..RadioSettings::default() };
        assert_eq!(c.connect(&cfg).unwrap_err().code(), "port_missing");
        assert!(c.snapshot().status.connected, "simulator must still be there");
    }

    #[test]
    fn a_port_that_does_not_exist_gives_an_io_error() {
        let c = AppCore::new(None);
        let cfg = RadioSettings { kind: RadioKind::Qmx, port: "/dev/definitely-not-here".into(), ..RadioSettings::default() };
        assert_eq!(c.connect(&cfg).unwrap_err().code(), "io");
    }

    #[test]
    fn the_band_scope_comes_through_and_can_be_switched_off() {
        let c = AppCore::new(None);
        let scope = |c: &AppCore| c.fast_tick().into_iter().filter(|e| matches!(e, CoreEvent::Rig { event: RigEvent::Scope(_) })).count();
        assert_eq!(scope(&c), 1);
        let mut s = c.settings();
        s.band_scope = false;
        c.update_settings(s).unwrap();
        assert_eq!(scope(&c), 0);
        let mut s = c.settings();
        s.band_scope = true;
        c.update_settings(s).unwrap();
        assert_eq!(scope(&c), 1);
    }

    #[test]
    fn disconnect_leaves_nothing_connected_and_sending_fails_politely() {
        let c = AppCore::new(None);
        c.set_policy(TxPolicy::Live, true).unwrap();
        c.disconnect();
        assert!(!c.snapshot().status.connected);
        assert_eq!(c.send_text("CQ").unwrap_err().code(), "not_connected");
        assert!(c.stop_tx().is_ok());
    }

    #[test]
    fn leaving_live_while_sending_stops_the_radio() {
        let c = AppCore::new(None);
        c.set_policy(TxPolicy::Live, true).unwrap();
        c.send_text("CQ CQ CQ DE ON3VZ K").unwrap();
        c.tick();
        assert!(c.snapshot().status.transmitting);
        c.set_policy(TxPolicy::ListenOnly, false).unwrap();
        assert!(!c.snapshot().status.transmitting);
    }

    #[test]
    fn connecting_the_simulator_again_works() {
        let c = AppCore::new(None);
        c.connect(&RadioSettings::default()).unwrap();
        assert!(c.snapshot().status.connected);
    }

    #[test]
    fn the_chosen_radio_is_saved() {
        let d = std::env::temp_dir().join(format!("cwld-core-save-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let p = d.join("settings.json");
        let c = AppCore::new(Some(p.clone()));
        c.connect(&RadioSettings { kind: RadioKind::Simulator, port: "COM3".into(), ..RadioSettings::default() }).unwrap();
        assert_eq!(Settings::load(&p).radio.port, "COM3");
    }
}

#[cfg(test)]
mod swr_tests {
    use super::*;

    fn live_core() -> AppCore {
        let c = AppCore::new(None);
        c.set_policy(TxPolicy::Live, true).unwrap();
        c
    }

    /// Starts a transmission and returns the events of `n` fast ticks.
    fn run(c: &AppCore, n: usize) -> Vec<CoreEvent> {
        (0..n).flat_map(|_| c.fast_tick()).collect()
    }

    fn start_tx(c: &AppCore) {
        c.send_text("CQ CQ CQ DE ON3VZ K").unwrap();
        assert!(c.snapshot().status.transmitting);
    }

    fn alerts(ev: &[CoreEvent]) -> usize {
        ev.iter().filter(|e| matches!(e, CoreEvent::SwrAlert { .. })).count()
    }

    #[test]
    fn a_good_antenna_never_trips() {
        let c = live_core();
        c.simulate_swr(1.2);
        start_tx(&c);
        assert_eq!(alerts(&run(&c, 50)), 0);
        assert!(c.snapshot().status.transmitting);
    }

    #[test]
    fn swr_exactly_at_the_limit_does_not_trip_but_above_does() {
        let c = live_core();
        c.simulate_swr(1.5);
        start_tx(&c);
        assert_eq!(alerts(&run(&c, 10)), 0);
        c.simulate_swr(1.6);
        assert_eq!(alerts(&run(&c, 10)), 1);
    }

    #[test]
    fn a_bad_antenna_stops_the_radio_and_blocks_sending_until_acknowledged() {
        let c = live_core();
        let mut cfg = c.settings();
        cfg.swr.qrx_after_stop = false; // here the radio must be completely quiet after the stop
        c.update_settings(cfg).unwrap();
        c.simulate_swr(3.0);
        start_tx(&c);
        let ev = run(&c, 5);
        assert_eq!(alerts(&ev), 1, "exactly one alert, not one per reading");
        let snap = c.snapshot();
        assert!(!snap.status.transmitting, "the radio must be stopped");
        assert_eq!(snap.swr_alert.as_ref().unwrap().limit, 1.5);
        // Blocked, in Live.
        assert_eq!(c.send_text("CQ").unwrap_err().code(), "swr_latched");
        // Stop still works.
        assert!(c.stop_tx().is_ok());
        // After the operator acknowledges, sending works again.
        assert_eq!(c.ack_swr(), vec![CoreEvent::SwrCleared]);
        c.simulate_swr(1.2);
        assert!(c.send_text("CQ").is_ok());
        assert!(c.ack_swr().is_empty());
    }

    #[test]
    fn with_qrx_the_radio_only_sends_that_after_the_stop_and_then_ends() {
        let c = live_core();
        c.simulate_swr(3.0);
        c.send_text("CQ CQ CQ CQ CQ CQ CQ CQ CQ CQ CQ CQ DE ON3VZ K").unwrap();
        run(&c, 5);
        // QRX is short: the simulated radio ends the transmission within a few seconds.
        for _ in 0..8 {
            c.tick();
        }
        assert!(!c.snapshot().status.transmitting);
        assert_eq!(c.transmit_commands_reached_radio(), 2);
    }

    #[test]
    fn one_noisy_reading_does_not_trip() {
        let c = live_core();
        start_tx(&c);
        c.simulate_swr(4.0);
        assert_eq!(alerts(&c.fast_tick()), 0);
        c.simulate_swr(1.2);
        assert_eq!(alerts(&run(&c, 10)), 0);
        assert!(c.snapshot().status.transmitting);
    }

    #[test]
    fn it_trips_within_a_few_hundred_milliseconds() {
        // Two readings in a row, one every 100 ms.
        let c = live_core();
        c.simulate_swr(2.5);
        start_tx(&c);
        assert_eq!(alerts(&c.fast_tick()), 0);
        assert_eq!(alerts(&c.fast_tick()), 1);
    }

    #[test]
    fn switched_off_it_does_nothing() {
        let c = live_core();
        let mut s = c.settings();
        s.swr.enabled = false;
        c.update_settings(s).unwrap();
        c.simulate_swr(5.0);
        start_tx(&c);
        assert_eq!(alerts(&run(&c, 10)), 0);
    }

    #[test]
    fn the_limit_is_adjustable() {
        let c = live_core();
        let mut s = c.settings();
        s.swr.limit = 2.0;
        c.update_settings(s).unwrap();
        c.simulate_swr(1.8);
        start_tx(&c);
        assert_eq!(alerts(&run(&c, 10)), 0);
        c.simulate_swr(2.2);
        assert_eq!(alerts(&run(&c, 10)), 1);
    }

    #[test]
    fn a_silly_limit_in_the_file_cannot_switch_the_protection_off() {
        let c = live_core();
        let mut s = c.settings();
        s.swr.limit = 99.0;
        c.update_settings(s).unwrap();
        c.simulate_swr(3.5);
        start_tx(&c);
        assert_eq!(alerts(&run(&c, 10)), 1, "limit is clamped to 3.0");
    }

    #[test]
    fn qrx_is_the_last_transmission_and_can_be_switched_off() {
        // Default: on.
        let on = live_core();
        on.simulate_swr(3.0);
        start_tx(&on);
        run(&on, 5);
        assert_eq!(on.transmit_commands_reached_radio(), 2, "text and QRX");
        assert!(on.snapshot().swr_alert.unwrap().qrx_sent);
        // QRX is sent once, not again for every later reading.
        run(&on, 20);
        assert_eq!(on.transmit_commands_reached_radio(), 2);

        let off = live_core();
        let mut s = off.settings();
        s.swr.qrx_after_stop = false;
        off.update_settings(s).unwrap();
        off.simulate_swr(3.0);
        start_tx(&off);
        run(&off, 5);
        assert_eq!(off.transmit_commands_reached_radio(), 1, "only the original text");
        assert!(!off.snapshot().swr_alert.unwrap().qrx_sent);
    }

    #[test]
    fn nothing_trips_outside_a_transmission() {
        let c = live_core();
        c.simulate_swr(5.0);
        assert_eq!(alerts(&run(&c, 20)), 0);
    }

    #[test]
    fn practice_and_listen_never_have_an_swr_alert() {
        let c = AppCore::new(None);
        c.simulate_swr(5.0);
        assert_eq!(alerts(&run(&c, 20)), 0);
        c.set_policy(TxPolicy::Practice, false).unwrap();
        c.send_text("CQ").unwrap();
        assert_eq!(alerts(&run(&c, 20)), 0);
    }
}
