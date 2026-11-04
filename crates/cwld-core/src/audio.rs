//! The decoder side of the core: starting and stopping audio, recording, retuning, and turning
//! audio events into core events. Decoded text heard while the radio transmits is sorted out
//! here: our own typed text is dropped (it is already in the conversation), anything else is
//! the paddle.

use crate::{AppCore, CoreError, CoreEvent};
use cwld_radio::Rig;
use cwld_audio::{list_inputs, Engine, EngineConfig, EngineEvent, InputDevice, Source};
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
}

impl AudioState {
    pub(crate) fn engine_ref(&self) -> Option<&Engine> {
        self.engine.as_ref()
    }
}

#[derive(Default)]
pub(crate) struct TxTrack {
    /// Until this moment the radio is busy with text the app sent.
    app_until: Option<Instant>,
    /// Last moment the radio was seen transmitting.
    last_tx: Option<Instant>,
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
        let s = self.settings.lock().unwrap().clone();
        let mut words: Vec<String> = s.station.calls.clone();
        words.extend(s.station.name.split_whitespace().map(|w| w.to_string()));
        words.extend(s.station.qth.split_whitespace().map(|w| w.to_string()));
        let cfg = EngineConfig { tone_hz: s.audio.safe_tone() as f64, words, min_separation: s.audio.safe_squelch(), ..Default::default() };
        let mut st = self.audio.lock().unwrap();
        match Engine::start(source, cfg) {
            Ok(e) => {
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

    pub fn stop_recording(&self) {
        let st = self.audio.lock().unwrap();
        if let Some(e) = &st.engine {
            e.stop_recording();
        }
    }

    /// Remembers that the app itself made the radio transmit, for about as long as it takes.
    pub(crate) fn note_app_transmission(&self, text: &str, wpm: u8) {
        let secs = cwld_types::morse::duration_seconds(text, wpm.max(1)).unwrap_or(text.len() as f64);
        let until = Instant::now() + Duration::from_secs_f64(secs) + TX_TAIL;
        let mut t = self.tx_track.lock().unwrap();
        t.app_until = Some(t.app_until.map_or(until, |u| u.max(until)));
    }

    /// Called about every 100 ms by the shell: everything the audio thread reported.
    pub fn audio_tick(&self) -> Vec<CoreEvent> {
        let transmitting = self.rig.lock().unwrap().status().transmitting;
        let radio_text = self.rig.lock().unwrap().capabilities().radio_decoder_text;
        let now = Instant::now();
        {
            let mut t = self.tx_track.lock().unwrap();
            if transmitting {
                t.last_tx = Some(now);
            }
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
                EngineEvent::Decode { text, conf, tentative, wpm } => {
                    if let Some(source) = self.route_decode(now, radio_text) {
                        out.push(CoreEvent::Decode { source, text, conf, tentative, wpm });
                    }
                }
                EngineEvent::Waterfall { bins, lo_hz, hz_per_bin } => out.push(CoreEvent::Waterfall { bins, lo_hz, hz_per_bin }),
                EngineEvent::Level { dbfs } => out.push(CoreEvent::AudioLevel { dbfs }),
                EngineEvent::Stats { confidence, snr_db } => {
                    if let Some(c) = confidence {
                        out.push(CoreEvent::Decoder { stats: cwld_types::DecoderStats { confidence: c, snr_db, accuracy: None } });
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
        out
    }

    /// Who keyed the decoded text? `None`: drop it (our own typed text, already shown).
    fn route_decode(&self, now: Instant, radio_gives_text: bool) -> Option<DecodeSource> {
        let t = self.tx_track.lock().unwrap();
        let tx_recent = t.last_tx.is_some_and(|l| now.duration_since(l) < TX_TAIL);
        let app_busy = t.app_until.is_some_and(|u| now < u);
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
