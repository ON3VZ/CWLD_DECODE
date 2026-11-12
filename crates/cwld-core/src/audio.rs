//! The decoder side of the core: starting and stopping audio, recording, retuning, and turning
//! audio events into core events. Decoded text heard while the radio transmits is sorted out
//! here: our own typed text is dropped (it is already in the conversation), anything else is
//! the paddle.

use crate::{AppCore, CoreError, CoreEvent};
use cwld_radio::Rig;
use cwld_audio::{list_inputs, Engine, EngineConfig, EngineEvent, InputDevice, ReplayResult, ReplayTone, Source, Tuning};
use serde::Serialize;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// After the radio stops transmitting, decoded text may still arrive for this long (the
/// decoder lags a little behind the audio).
pub const TX_TAIL: Duration = Duration::from_millis(2500);

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct AudioStatus {
    pub running: bool,
    /// Device name or file path.
    pub source: String,
    pub rate: u32,
    pub tone_hz: f64,
    pub error: Option<String>,
    pub recording: bool,
    pub record_seconds: f64,
    pub record_path: String,
    /// Samples lost because the computer was too busy. Should stay 0.
    pub overruns: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecodeSource {
    /// The other station.
    Rx,
    /// Keyed with the paddle on the radio, heard in the radio's audio.
    Paddle,
}

#[derive(Default)]
pub(crate) struct AudioState {
    engine: Option<Engine>,
    status: AudioStatus,
    /// The paddle trainer wants the key times (kept when the audio restarts).
    keying: bool,
}

impl AudioState {
    pub(crate) fn engine_ref(&self) -> Option<&Engine> {
        self.engine.as_ref()
    }
}

#[derive(Default)]
pub(crate) struct TxTrack {
    /// Until this moment the radio is busy with text the app sent (estimated from the speed).
    app_until: Option<Instant>,
    /// The app started a transmission that is not over yet. It ends when the estimate has run
    /// out and the radio is no longer transmitting.
    app_tx: bool,
    /// Last moment the radio was seen transmitting.
    last_tx: Option<Instant>,
    /// The radio was transmitting at the previous tick.
    was_tx: bool,
    /// While the decoder is being reset: where text decoded before the reset goes (`None`:
    /// dropped), and how many resets are still under way.
    reset_route: Option<DecodeSource>,
    resets_pending: u32,
    /// Why the decoder shows nothing right now (see `DecodeHold`), as last reported.
    hold: Option<DecodeHold>,
    /// The radio was on a phone or data mode at the previous tick.
    mode_held: bool,
    voice: bool,
}

/// Why no text comes out of the decoder although audio is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecodeHold {
    /// The radio is on SSB, AM, FM or RTTY: CW is not expected, nothing is decoded.
    Mode,
    /// The audio sounds like speech or another wide signal, not like a CW tone.
    Voice,
}

/// Modes in which a CW decoder can only make up text.
pub fn mode_holds_decoder(mode: cwld_types::RadioMode) -> bool {
    use cwld_types::RadioMode::*;
    matches!(mode, Usb | Lsb | Am | Fm | Fsk)
}

impl AppCore {
    pub fn list_audio_inputs(&self) -> Vec<InputDevice> {
        list_inputs()
    }

    pub fn audio_status(&self) -> AudioStatus {
        self.audio.lock().unwrap().status.clone()
    }

    /// Starts listening to the input chosen in the settings.
    pub fn start_audio(&self) -> Result<AudioStatus, CoreError> {
        let input = self.settings.lock().unwrap().audio.input.clone();
        self.start_audio_source(Source::Device { name: Some(input) })
    }

    /// Decodes a WAV file at real speed, as if it came from the radio.
    pub fn start_audio_file(&self, path: PathBuf) -> Result<AudioStatus, CoreError> {
        self.start_audio_source(Source::File { path, realtime: true })
    }

    pub fn start_audio_source(&self, source: Source) -> Result<AudioStatus, CoreError> {
        self.stop_audio();
        if !matches!(source, Source::Practice { .. }) {
            self.forget_practice();
        }
        let s = self.settings.lock().unwrap().clone();
        let mut words: Vec<String> = s.station.calls.clone();
        words.extend(s.station.name.split_whitespace().map(|w| w.to_string()));
        words.extend(s.station.qth.split_whitespace().map(|w| w.to_string()));
        let cfg = EngineConfig { tone_hz: s.audio.safe_tone() as f64, words, min_separation: s.audio.safe_squelch(), auto_tone: s.audio.auto_tone, afc: s.audio.afc, fading: s.audio.fading, tuning: tuning_of(&s.audio), ..Default::default() };
        let mut st = self.audio.lock().unwrap();
        match Engine::start(source, cfg) {
            Ok(e) => {
                if st.keying {
                    e.set_keying(true);
                }
                st.engine = Some(e);
                st.status = AudioStatus { running: true, tone_hz: s.audio.safe_tone() as f64, ..Default::default() };
                Ok(st.status.clone())
            }
            Err(e) => {
                st.status = AudioStatus { error: Some(e.clone()), tone_hz: s.audio.safe_tone() as f64, ..Default::default() };
                Err(CoreError::Audio(e))
            }
        }
    }

    pub fn stop_audio(&self) -> AudioStatus {
        let mut st = self.audio.lock().unwrap();
        // Dropping the engine stops its thread and finishes a recording.
        st.engine = None;
        st.status.running = false;
        st.status.recording = false;
        st.status.clone()
    }

    /// Moves the decoder to another pitch (a click on the waterfall) and remembers it.
    pub fn set_tone(&self, hz: f64) -> Result<(), CoreError> {
        let hz = hz.clamp(200.0, 1500.0);
        self.settings.lock().unwrap().audio.tone_hz = hz as f32;
        let mut st = self.audio.lock().unwrap();
        st.status.tone_hz = hz;
        if let Some(e) = &st.engine {
            e.retune(hz);
        }
        Ok(())
    }

    /// Changes how strict the decoder is about signals in noise, and remembers it.
    pub fn set_squelch(&self, v: f32) -> Result<(), CoreError> {
        let v = if v.is_nan() { cwld_types::SQUELCH_DEFAULT } else { v.clamp(cwld_types::SQUELCH_MIN, cwld_types::SQUELCH_MAX) };
        self.settings.lock().unwrap().audio.squelch = v;
        if let Some(e) = &self.audio.lock().unwrap().engine {
            e.set_squelch(v);
        }
        Ok(())
    }

    /// Weak marks in fading on or off, remembered.
    pub fn set_fading(&self, on: bool) -> Result<(), CoreError> {
        self.settings.lock().unwrap().audio.fading = on;
        if let Some(e) = &self.audio.lock().unwrap().engine {
            e.set_fading(on);
        }
        Ok(())
    }

    /// AFC on or off, remembered.
    pub fn set_afc(&self, on: bool) -> Result<(), CoreError> {
        self.settings.lock().unwrap().audio.afc = on;
        if let Some(e) = &self.audio.lock().unwrap().engine {
            e.set_afc(on);
        }
        Ok(())
    }

    /// Once: the decoder moves to the strongest keyed tone heard lately.
    pub fn go_strongest(&self) -> Result<(), CoreError> {
        if let Some(e) = &self.audio.lock().unwrap().engine {
            e.go_strongest();
        }
        Ok(())
    }

    /// Changes the gain, filter width and speed range of the running decoder, and remembers them.
    #[allow(clippy::too_many_arguments)]
    pub fn set_tuning(&self, gain_db: f32, wide_filter: bool, steep_filter: bool, join_spelled: bool, min_wpm: u8, max_wpm: u8, lock_wpm: u8) -> Result<(), CoreError> {
        let audio = {
            let mut s = self.settings.lock().unwrap();
            s.audio.gain_db = gain_db;
            s.audio.wide_filter = wide_filter;
            s.audio.steep_filter = steep_filter && !wide_filter;
            s.audio.join_spelled = join_spelled;
            s.audio.min_wpm = min_wpm;
            s.audio.max_wpm = max_wpm;
            s.audio.lock_wpm = lock_wpm;
            s.audio.clone()
        };
        if let Some(e) = &self.audio.lock().unwrap().engine {
            e.set_tuning(tuning_of(&audio));
        }
        Ok(())
    }

    /// Switches the automatic search for the strongest signal on or off, and remembers it.
    pub fn set_auto_tone(&self, on: bool) -> Result<(), CoreError> {
        self.settings.lock().unwrap().audio.auto_tone = on;
        if let Some(e) = &self.audio.lock().unwrap().engine {
            e.set_auto_tone(on);
        }
        Ok(())
    }

    /// Where recordings go when the settings leave it empty. Set by the shell (Documents/CWLD).
    pub fn set_default_recordings_dir(&self, dir: PathBuf) {
        *self.recordings_dir.lock().unwrap() = Some(dir);
    }

    fn recordings_dir(&self) -> PathBuf {
        let chosen = self.settings.lock().unwrap().audio.record_dir.trim().to_string();
        if !chosen.is_empty() {
            return PathBuf::from(chosen);
        }
        if let Some(d) = self.recordings_dir.lock().unwrap().clone() {
            return d;
        }
        match &self.settings_path {
            Some(p) => p.parent().map(|d| d.join("recordings")).unwrap_or_else(|| PathBuf::from("recordings")),
            None => std::env::temp_dir().join("cwld-recordings"),
        }
    }

    /// Starts a recording of the input. The file name holds the date, time and frequency.
    pub fn start_recording(&self) -> Result<String, CoreError> {
        let khz = self.rig.lock().unwrap().status().frequency_hz as f64 / 1000.0;
        let name = format!("cwld-{}-{:.1}kHz.wav", timestamp(SystemTime::now()), khz);
        let path = self.recordings_dir().join(name);
        let mut st = self.audio.lock().unwrap();
        match &st.engine {
            Some(e) => {
                e.start_recording(path.clone());
                st.status.recording = true;
                st.status.record_seconds = 0.0;
                st.status.record_path = path.display().to_string();
                Ok(st.status.record_path.clone())
            }
            None => Err(CoreError::Audio("audio is not running".into())),
        }
    }

    /// Decodes the last `seconds` again (5 to 60) with a separate decoder: at the pitch of the
    /// live decoder, at `tone_hz`, or at the strongest keyed tone of that stretch. The live
    /// decoder goes on undisturbed. Slow (about a second): call it off the interface thread.
    pub fn replay(&self, seconds: f64, tone_hz: Option<f64>, strongest: bool) -> Result<ReplayResult, CoreError> {
        let rw = self.take_rewind()?;
        let tone = match (tone_hz, strongest) {
            (_, true) => ReplayTone::Strongest,
            (Some(hz), false) => ReplayTone::Fixed(hz.clamp(200.0, 1500.0)),
            (None, false) => ReplayTone::Current,
        };
        let seconds = if seconds.is_finite() { seconds } else { 20.0 };
        Ok(cwld_audio::replay::replay(&rw, seconds, tone))
    }

    /// Saves the last minute of audio (as the decoder heard it, 8 kHz) as a WAV file in the
    /// recordings folder. Returns the path.
    pub fn save_rewind(&self) -> Result<String, CoreError> {
        let rw = self.take_rewind()?;
        if rw.samples.is_empty() {
            return Err(CoreError::Audio("nothing heard yet".into()));
        }
        let khz = self.rig.lock().unwrap().status().frequency_hz as f64 / 1000.0;
        let name = format!("cwld-{}-{:.1}kHz-last{}s.wav", timestamp(SystemTime::now()), khz, (rw.samples.len() as f64 / cwld_audio::engine::DECODE_RATE as f64).round() as u32);
        let path = self.recordings_dir().join(name);
        cwld_audio::replay::save_wav(&rw, &path).map_err(|e| CoreError::Audio(format!("{}: {e}", path.display())))?;
        Ok(path.display().to_string())
    }

    fn take_rewind(&self) -> Result<cwld_audio::Rewind, CoreError> {
        let st = self.audio.lock().unwrap();
        let e = st.engine.as_ref().ok_or_else(|| CoreError::Audio("audio is not running".into()))?;
        e.rewind().ok_or_else(|| CoreError::Audio("the audio thread did not answer".into()))
    }

    pub fn stop_recording(&self) {
        let st = self.audio.lock().unwrap();
        if let Some(e) = &st.engine {
            e.stop_recording();
        }
    }

    /// Remembers that the app itself made the radio transmit, for about as long as it takes.
    /// When a new transmission starts, the decoder first hands out what it still holds of the
    /// other station and then starts afresh, so our own sidetone never mixes with it.
    pub(crate) fn note_app_transmission(&self, text: &str, wpm: u8) {
        let secs = cwld_types::morse::duration_seconds(text, wpm.max(1)).unwrap_or(text.len() as f64);
        let now = Instant::now();
        let starting = {
            let mut t = self.tx_track.lock().unwrap();
            // Text queued behind text that is still being sent starts after it.
            let from = t.app_until.filter(|u| *u > now).map_or(now, |u| u.max(now));
            let until = from + Duration::from_secs_f64(secs) + TX_TAIL;
            t.app_until = Some(until);
            let starting = !t.app_tx;
            t.app_tx = true;
            starting
        };
        if starting {
            self.reset_decoder(true, Some(DecodeSource::Rx));
        }
    }

    /// Restarts the decoder. Text it produced before the restart goes to `route`.
    fn reset_decoder(&self, flush: bool, route: Option<DecodeSource>) {
        let st = self.audio.lock().unwrap();
        if let Some(e) = &st.engine {
            let mut t = self.tx_track.lock().unwrap();
            t.reset_route = route;
            t.resets_pending += 1;
            e.reset(flush);
        }
    }

    /// Follows the transmit state of the radio and restarts the decoder at each change, so the
    /// other station, our own app text and the paddle never end up in one piece of text.
    fn track_transmit(&self, transmitting: bool, radio_text: bool, now: Instant) {
        let action = {
            let mut t = self.tx_track.lock().unwrap();
            if transmitting {
                t.last_tx = Some(now);
            }
            let app_time_left = t.app_until.is_some_and(|u| now < u);
            let mut action = None;
            if t.app_tx {
                if !transmitting && !app_time_left {
                    // Our own transmission is over: whatever the sidetone left is dropped.
                    t.app_tx = false;
                    action = Some((false, None));
                }
            } else if transmitting && !t.was_tx {
                // The operator keys the paddle: the other station's text comes out first.
                action = Some((true, Some(DecodeSource::Rx)));
            } else if !transmitting && t.was_tx {
                // The paddle stops: its last characters come out as the paddle.
                action = Some((true, if radio_text { None } else { Some(DecodeSource::Paddle) }));
            }
            t.was_tx = transmitting;
            action
        };
        if let Some((flush, route)) = action {
            self.reset_decoder(flush, route);
        }
    }

    /// The paddle trainer: hand out key down and up times (`CoreEvent::Keying`) or stop.
    pub fn set_keying(&self, on: bool) {
        let mut st = self.audio.lock().unwrap();
        st.keying = on;
        if let Some(e) = &st.engine {
            e.set_keying(on);
        }
    }

    /// Called about every 100 ms by the shell: everything the audio thread reported.
    pub fn audio_tick(&self) -> Vec<CoreEvent> {
        let (transmitting, connected, mode) = {
            let r = self.rig.lock().unwrap();
            let s = r.status();
            (s.transmitting, s.connected, s.mode)
        };
        let radio_text = self.rig.lock().unwrap().capabilities().radio_decoder_text;
        let now = Instant::now();
        self.track_transmit(transmitting, radio_text, now);
        // In practice mode the audio is the practice band, whatever the radio is set to.
        let mode_hold = connected && self.policy() != cwld_types::TxPolicy::Practice && mode_holds_decoder(mode);
        let back_on_cw = {
            let mut t = self.tx_track.lock().unwrap();
            let back = t.mode_held && !mode_hold;
            t.mode_held = mode_hold;
            back
        };
        if back_on_cw {
            // What the decoder heard on SSB is of no use: start clean on CW.
            self.reset_decoder(false, None);
        }
        let events = {
            let st = self.audio.lock().unwrap();
            match &st.engine {
                Some(e) => e.drain(),
                None => return Vec::new(),
            }
        };
        let mut out = Vec::new();
        let mut status_changed = false;
        for ev in events {
            match ev {
                EngineEvent::ResetDone => {
                    let mut t = self.tx_track.lock().unwrap();
                    t.resets_pending = t.resets_pending.saturating_sub(1);
                    if t.resets_pending == 0 {
                        t.reset_route = None;
                    }
                }
                EngineEvent::Decode { text, conf, tentative, wpm } => {
                    if mode_hold {
                        continue;
                    }
                    if let Some(source) = self.route_decode(now, radio_text) {
                        out.push(CoreEvent::Decode { source, text, conf, tentative, wpm });
                    }
                }
                EngineEvent::Waterfall { bins, lo_hz, hz_per_bin } => out.push(CoreEvent::Waterfall { bins, lo_hz, hz_per_bin }),
                EngineEvent::Level { dbfs, peak } => out.push(CoreEvent::AudioLevel { dbfs, peak }),
                EngineEvent::PracticeDone { ids } => out.push(CoreEvent::PracticeDone { ids }),
                EngineEvent::Keying { runs } => out.push(CoreEvent::Keying { runs }),
                EngineEvent::Stats { confidence, snr_db, bandwidth_hz, voice, hole_hz } => {
                    self.tx_track.lock().unwrap().voice = voice;
                    // Report a hole (or its end) once, rounded so it does not flicker.
                    let now = hole_hz.map(|h| ((h / 50.0).round() * 50.0) as i32);
                    let mut last = self.hole_reported.lock().unwrap();
                    if *last != now {
                        *last = now;
                        out.push(CoreEvent::AudioHole { hz: now.map(|h| h as f32) });
                    }
                    drop(last);
                    out.push(CoreEvent::Bandwidth { hz: bandwidth_hz });
                    if let Some(c) = confidence {
                        out.push(CoreEvent::Decoder { stats: cwld_types::DecoderStats { confidence: c, snr_db, accuracy: None, bandwidth_hz: Some(bandwidth_hz) } });
                    }
                }
                other => {
                    let mut st = self.audio.lock().unwrap();
                    match other {
                        EngineEvent::Started { source, rate } => {
                            st.status.source = source;
                            st.status.rate = rate;
                            st.status.running = true;
                            st.status.error = None;
                        }
                        EngineEvent::Tone { hz } => st.status.tone_hz = hz,
                        EngineEvent::Recording { active, seconds, path } => {
                            st.status.recording = active;
                            st.status.record_seconds = seconds;
                            st.status.record_path = path;
                        }
                        EngineEvent::Overrun { samples } => st.status.overruns += samples,
                        EngineEvent::Stopped { reason } => {
                            st.status.running = false;
                            st.status.recording = false;
                            st.status.error = reason;
                        }
                        _ => {}
                    }
                    status_changed = true;
                }
            }
        }
        if status_changed {
            out.push(CoreEvent::Audio { status: self.audio_status() });
        }
        let hold_change = {
            let mut t = self.tx_track.lock().unwrap();
            let hold = if mode_hold { Some(DecodeHold::Mode) } else if t.voice { Some(DecodeHold::Voice) } else { None };
            (hold != t.hold).then(|| {
                t.hold = hold;
                hold
            })
        };
        if let Some(hold) = hold_change {
            out.push(CoreEvent::DecodeHold { hold, mode: format!("{mode:?}").to_uppercase() });
        }
        out
    }

    /// Who keyed the decoded text? `None`: drop it (our own typed text, already shown).
    fn route_decode(&self, now: Instant, radio_gives_text: bool) -> Option<DecodeSource> {
        let t = self.tx_track.lock().unwrap();
        if t.resets_pending > 0 {
            return t.reset_route;
        }
        let tx_recent = t.last_tx.is_some_and(|l| now.duration_since(l) < TX_TAIL);
        let app_busy = t.app_tx || t.app_until.is_some_and(|u| now < u);
        if app_busy {
            // The radio is (or was just) sending what the app typed.
            return None;
        }
        if tx_recent {
            // The paddle. A radio that reports its own paddle text (QMX) already sent it.
            return if radio_gives_text { None } else { Some(DecodeSource::Paddle) };
        }
        Some(DecodeSource::Rx)
    }
}

/// `YYYYMMDD-HHMMSS` in UTC, without a date library.
/// The decoder tuning from the settings, kept inside its ranges.
pub(crate) fn tuning_of(a: &cwld_types::AudioSettings) -> Tuning {
    let (lo, hi) = a.safe_wpm_range();
    Tuning { gain_db: a.safe_gain_db(), wide: a.wide_filter, steep: a.steep_filter && !a.wide_filter, join: a.join_spelled, wpm_min: lo as f32, wpm_max: hi as f32, fixed_wpm: a.safe_lock_wpm().map(|w| w as f32) }
}

pub fn timestamp(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rem / 3600, (rem % 3600) / 60, rem % 60)
}


#[cfg(test)]
mod tests {
    use super::*;
    use cwld_dsp::{generate, CwSpec};
    use cwld_types::TxPolicy;

    fn core() -> AppCore {
        AppCore::new(None)
    }

    fn run_until_stopped(c: &AppCore) -> Vec<CoreEvent> {
        let mut all = Vec::new();
        for _ in 0..400 {
            std::thread::sleep(Duration::from_millis(25));
            let ev = c.audio_tick();
            let stopped = ev.iter().any(|e| matches!(e, CoreEvent::Audio { status } if !status.running));
            all.extend(ev);
            if stopped {
                break;
            }
        }
        all
    }

    fn decoded(ev: &[CoreEvent], want: DecodeSource) -> String {
        ev.iter()
            .filter_map(|e| match e {
                CoreEvent::Decode { source, text, .. } if *source == want => Some(text.as_str()),
                _ => None,
            })
            .collect::<String>()
            .trim()
            .to_string()
    }

    #[test]
    fn timestamps_are_right() {
        // 2026-11-04 21:05:09 UTC
        let t = UNIX_EPOCH + Duration::from_secs(1_793_826_309);
        assert_eq!(timestamp(t), "20261104-210509");
        assert_eq!(timestamp(UNIX_EPOCH), "19700101-000000");
    }

    #[test]
    fn decoded_text_reaches_the_interface_as_rx() {
        let c = core();
        let g = generate(&CwSpec { text: "CQ CQ DE DL1ABC K".into(), snr_db: Some(15.0), ..Default::default() });
        c.start_audio_source(Source::Samples { rate: 8000, data: g.samples, realtime: false }).unwrap();
        let ev = run_until_stopped(&c);
        assert_eq!(decoded(&ev, DecodeSource::Rx), g.truth);
        assert!(ev.iter().any(|e| matches!(e, CoreEvent::Waterfall { .. })));
        assert!(ev.iter().any(|e| matches!(e, CoreEvent::Decoder { .. })));
    }

    #[test]
    fn own_typed_text_is_not_shown_twice() {
        let c = core();
        c.set_policy(TxPolicy::Live, true).unwrap();
        c.send_text("TEST DE ON3VZ").unwrap();
        let g = generate(&CwSpec { text: "TEST DE ON3VZ".into(), snr_db: Some(20.0), ..Default::default() });
        c.start_audio_source(Source::Samples { rate: 8000, data: g.samples, realtime: false }).unwrap();
        let ev = run_until_stopped(&c);
        assert_eq!(decoded(&ev, DecodeSource::Rx), "");
        assert_eq!(decoded(&ev, DecodeSource::Paddle), "");
    }

    #[test]
    fn nothing_is_decoded_while_the_radio_is_on_ssb() {
        let c = core();
        c.connect(&cwld_types::RadioSettings::default()).unwrap();
        c.rig.lock().unwrap().execute(&cwld_types::RadioCommand::SetMode(cwld_types::RadioMode::Usb)).unwrap();
        let g = generate(&CwSpec { text: "CQ CQ DE DL1ABC K".into(), snr_db: Some(15.0), ..Default::default() });
        c.start_audio_source(Source::Samples { rate: 8000, data: g.samples.clone(), realtime: false }).unwrap();
        let ev = run_until_stopped(&c);
        assert_eq!(decoded(&ev, DecodeSource::Rx), "");
        assert!(ev.iter().any(|e| matches!(e, CoreEvent::DecodeHold { hold: Some(DecodeHold::Mode), mode } if mode == "USB")));
        // In practice mode the radio mode does not matter.
        let c = core();
        c.connect(&cwld_types::RadioSettings::default()).unwrap();
        c.rig.lock().unwrap().execute(&cwld_types::RadioCommand::SetMode(cwld_types::RadioMode::Lsb)).unwrap();
        c.set_policy(TxPolicy::Practice, false).unwrap();
        c.start_audio_source(Source::Samples { rate: 8000, data: g.samples, realtime: false }).unwrap();
        assert_eq!(decoded(&run_until_stopped(&c), DecodeSource::Rx), g.truth);
    }

    #[test]
    fn the_key_times_of_a_clean_sidetone_come_out_for_the_trainer() {
        let c = core();
        c.set_keying(true);
        // 20 WPM: a dot is 60 ms, a dash 180 ms, a gap inside a character 60 ms.
        let g = generate(&CwSpec { text: "PARIS PARIS".into(), wpm: 20.0, snr_db: None, lead_s: 0.5, trail_s: 3.0, ..Default::default() });
        c.start_audio_source(Source::Samples { rate: 8000, data: g.samples, realtime: false }).unwrap();
        let ev = run_until_stopped(&c);
        let runs: Vec<(bool, f32)> = ev.iter().filter_map(|e| if let CoreEvent::Keying { runs } = e { Some(runs.clone()) } else { None }).flatten().collect();
        let downs: Vec<f32> = runs.iter().filter(|r| r.0).map(|r| r.1).collect();
        // PARIS has 14 elements: 10 dots and 4 dashes, twice.
        assert_eq!(downs.len(), 28, "{runs:?}");
        let dots = downs.iter().filter(|d| **d < 120.0).count();
        assert_eq!(dots, 20, "{downs:?}");
        for d in &downs {
            let near = if *d < 120.0 { 60.0 } else { 180.0 };
            assert!((d - near).abs() <= 20.0, "{d} ms, {downs:?}");
        }
    }

    #[test]
    fn phone_and_data_modes_hold_the_decoder() {
        use cwld_types::RadioMode::*;
        for m in [Usb, Lsb, Am, Fm, Fsk] {
            assert!(mode_holds_decoder(m));
        }
        for m in [Cw, CwReverse, Other] {
            assert!(!mode_holds_decoder(m));
        }
    }

    #[test]
    fn text_while_the_radio_transmits_by_itself_is_the_paddle() {
        let c = core();
        let now = Instant::now();
        c.tx_track.lock().unwrap().last_tx = Some(now);
        assert_eq!(c.route_decode(now, false), Some(DecodeSource::Paddle));
        // A QMX reports the paddle text itself: the audio copy is dropped.
        assert_eq!(c.route_decode(now, true), None);
        // Long after the transmission: the other station again.
        assert_eq!(c.route_decode(now + Duration::from_secs(5), false), Some(DecodeSource::Rx));
    }

    #[test]
    fn our_own_transmission_stays_ours_until_the_radio_is_back_on_receive() {
        let c = core();
        let t0 = Instant::now();
        c.note_app_transmission("CQ CQ DE ON3VZ ON3VZ K", 20);
        // The estimate ran out but the radio still transmits (slower than we thought).
        c.tx_track.lock().unwrap().app_until = Some(t0);
        let later = t0 + Duration::from_secs(1);
        c.track_transmit(true, false, later);
        assert_eq!(c.route_decode(later, false), None);
        // Back on receive: our transmission is over, the other station is Rx again.
        let rx = later + TX_TAIL + Duration::from_millis(100);
        c.track_transmit(false, false, rx);
        assert!(!c.tx_track.lock().unwrap().app_tx);
        assert_eq!(c.route_decode(rx, false), Some(DecodeSource::Rx));
    }

    #[test]
    fn a_new_transmission_flushes_the_other_station_first() {
        let c = core();
        let g = generate(&CwSpec { text: "CQ CQ DE DL1ABC DL1ABC K".into(), snr_db: Some(20.0), lead_s: 1.0, trail_s: 6.0, ..Default::default() });
        c.start_audio_source(Source::Samples { rate: 8000, data: g.samples.clone(), realtime: true }).unwrap();
        // Let the other station finish its last word, then start sending ourselves.
        let mut all = Vec::new();
        let mut sent = false;
        let started = Instant::now();
        let keyed_until = Duration::from_secs_f64(g.samples.len() as f64 / 8000.0 - 6.0 + 0.4);
        for _ in 0..1200 {
            std::thread::sleep(Duration::from_millis(25));
            let ev = c.audio_tick();
            let stopped = ev.iter().any(|e| matches!(e, CoreEvent::Audio { status } if !status.running));
            all.extend(ev);
            // The message has been keyed completely, the last part is still held by the decoder.
            if !sent && started.elapsed() > keyed_until {
                c.note_app_transmission("TEST", 20);
                sent = true;
            }
            if stopped {
                break;
            }
        }
        assert!(sent, "{:?}", decoded(&all, DecodeSource::Rx));
        // The K that the decoder still held came out as the other station, not dropped.
        assert_eq!(decoded(&all, DecodeSource::Rx), g.truth);
        assert_eq!(c.tx_track.lock().unwrap().resets_pending, 0);
    }

    #[test]
    fn replay_reads_the_last_seconds_again_and_saves_them() {
        let c = core();
        let dir = std::env::temp_dir().join(format!("cwld-core-rewind-{}", std::process::id()));
        c.set_default_recordings_dir(dir.clone());
        assert!(c.replay(20.0, None, false).is_err(), "no audio, no replay");
        let g = generate(&CwSpec { text: "CQ DE ON3VZ K".into(), snr_db: Some(15.0), lead_s: 1.0, ..Default::default() });
        c.start_audio_source(Source::Samples { rate: 8000, data: g.samples, realtime: false }).unwrap();
        let _ = run_until_stopped(&c);
        let r = c.replay(20.0, Some(700.0), false).unwrap();
        assert_eq!(r.text, g.truth);
        let path = c.save_rewind().unwrap();
        assert!(path.contains("kHz-last") && path.ends_with("s.wav"), "{path}");
        assert!(std::path::Path::new(&path).exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn recording_writes_a_file_with_date_and_frequency() {
        let c = core();
        let dir = std::env::temp_dir().join(format!("cwld-core-rec-{}", std::process::id()));
        c.set_default_recordings_dir(dir.clone());
        let g = generate(&CwSpec { text: "QRZ".into(), lead_s: 2.0, ..Default::default() });
        c.start_audio_source(Source::Samples { rate: 8000, data: g.samples, realtime: true }).unwrap();
        let path = c.start_recording().unwrap();
        assert!(path.contains("kHz.wav"), "{path}");
        let _ = run_until_stopped(&c);
        assert!(std::path::Path::new(&path).exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn no_sound_card_is_a_clear_error() {
        let c = core();
        // Tests build without sound card support: starting the device must fail cleanly.
        let r = c.start_audio();
        if let Err(e) = r {
            assert_eq!(e.code(), "audio");
            assert!(c.audio_status().error.is_some());
            assert!(!c.audio_status().running);
        }
    }

    #[test]
    fn tuning_is_remembered_and_kept_in_range() {
        let c = core();
        c.set_tuning(6.0, true, true, false, 8, 40, 0).unwrap();
        let a = c.settings().audio;
        assert_eq!((a.gain_db, a.wide_filter, a.min_wpm, a.max_wpm, a.lock_wpm), (6.0, true, 8, 40, 0));
        // Wide and steep do not go together: wide wins.
        assert!(!a.steep_filter && !a.join_spelled);
        let t = tuning_of(&a);
        assert!(t.wide && !t.steep && !t.join && t.fixed_wpm.is_none() && t.wpm_min == 8.0 && t.wpm_max == 40.0);
        c.set_tuning(99.0, false, true, true, 1, 200, 18).unwrap();
        let t = tuning_of(&c.settings().audio);
        assert!(t.steep && !t.wide && t.join);
        assert_eq!(t.gain_db, 20.0);
        assert_eq!((t.wpm_min, t.wpm_max), (5.0, 60.0));
        assert_eq!(t.fixed_wpm, Some(18.0));
    }

    #[test]
    fn squelch_is_remembered_and_clamped() {
        let c = core();
        c.set_squelch(50.0).unwrap();
        assert_eq!(c.settings().audio.squelch, cwld_types::SQUELCH_MAX);
        c.set_squelch(f32::NAN).unwrap();
        assert_eq!(c.settings().audio.squelch, cwld_types::SQUELCH_DEFAULT);
        c.set_squelch(3.5).unwrap();
        assert_eq!(c.settings().audio.squelch, 3.5);
    }

    #[test]
    fn retune_is_remembered_and_clamped() {
        let c = core();
        c.set_tone(3000.0).unwrap();
        assert_eq!(c.settings().audio.tone_hz, 1500.0);
        c.set_tone(620.0).unwrap();
        assert_eq!(c.audio_status().tone_hz, 620.0);
    }
}
