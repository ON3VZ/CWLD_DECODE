//! The practice band: simulated stations instead of the radio. Nothing is ever transmitted.
//! The interface runs the simulator (who calls, what they send) and hands every transmission
//! to `practice_key`. The band is decoded like the radio and played on the computer's output.

use crate::audio::AudioStatus;
use crate::{AppCore, CoreError};
use cwld_audio::Source;
use cwld_dsp::{BandSignal, PracticeBand};
use cwld_types::TxPolicy;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

/// One transmission of a simulated station, as the interface sends it.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(default)]
pub struct PracticeSignal {
    pub text: String,
    pub wpm: f64,
    pub tone_hz: f64,
    pub snr_db: f64,
    pub qsb_depth: f64,
    pub jitter: f64,
    pub delay_s: f64,
    pub seed: u64,
}

impl Default for PracticeSignal {
    fn default() -> Self {
        let b = BandSignal::default();
        PracticeSignal { text: String::new(), wpm: b.wpm, tone_hz: b.tone_hz, snr_db: b.snr_db, qsb_depth: 0.0, jitter: 0.0, delay_s: 0.0, seed: 1 }
    }
}

/// Longest text one transmission may hold: a CQ or an exchange is much shorter.
pub const MAX_PRACTICE_TEXT: usize = 200;

impl AppCore {
    pub fn practice_running(&self) -> bool {
        self.practice.lock().unwrap().is_some()
    }

    /// Starts the practice band in place of the radio audio. Only in practice mode.
    /// `play`: heard on the default output. `noise`: band noise on.
    pub fn practice_start(&self, noise: bool, volume: f32, seed: u64, play: bool) -> Result<AudioStatus, CoreError> {
        if self.policy() != TxPolicy::Practice {
            return Err(CoreError::Invalid("practice_mode_needed"));
        }
        let band = Arc::new(Mutex::new(PracticeBand::new(cwld_audio::engine::DECODE_RATE, seed.max(1), noise)));
        let st = self.start_audio_source(Source::Practice { band: band.clone(), play, volume })?;
        *self.practice.lock().unwrap() = Some(band);
        Ok(st)
    }

    /// Ends the practice band and the audio. The interface starts the radio audio again.
    pub fn practice_stop(&self) {
        let was = self.practice.lock().unwrap().take();
        if was.is_some() {
            self.stop_audio();
        }
    }

    /// Forgets the band without touching the audio (another source takes over).
    pub(crate) fn forget_practice(&self) {
        self.practice.lock().unwrap().take();
    }

    /// Puts one transmission on the practice band. Returns its id for `PracticeDone`.
    pub fn practice_key(&self, s: &PracticeSignal) -> Result<u64, CoreError> {
        if self.policy() != TxPolicy::Practice {
            return Err(CoreError::Invalid("practice_mode_needed"));
        }
        if s.text.len() > MAX_PRACTICE_TEXT {
            return Err(CoreError::Invalid("message_too_long"));
        }
        let guard = self.practice.lock().unwrap();
        let band = guard.as_ref().ok_or(CoreError::Invalid("practice_not_running"))?;
        let sig = BandSignal { text: s.text.clone(), wpm: s.wpm, tone_hz: s.tone_hz, snr_db: s.snr_db, qsb_depth: s.qsb_depth, jitter: s.jitter, delay_s: s.delay_s.clamp(0.0, 30.0), seed: s.seed };
        let id = band.lock().unwrap().add(&sig);
        Ok(id)
    }

    /// Takes every simulated station off the band at once.
    pub fn practice_clear(&self) {
        if let Some(b) = self.practice.lock().unwrap().as_ref() {
            b.lock().unwrap().clear();
        }
    }

    pub fn practice_volume(&self, v: f32) {
        if let Some(e) = self.audio.lock().unwrap().engine_ref() {
            e.set_volume(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CoreEvent;
    use std::time::{Duration, Instant};

    fn practice_core() -> AppCore {
        let c = AppCore::new(None);
        c.set_policy(TxPolicy::Practice, false).unwrap();
        c
    }

    #[test]
    fn needs_practice_mode() {
        let c = AppCore::new(None);
        c.set_policy(TxPolicy::ListenOnly, false).unwrap();
        assert!(matches!(c.practice_start(true, 0.5, 1, false), Err(CoreError::Invalid("practice_mode_needed"))));
        assert!(matches!(c.practice_key(&PracticeSignal::default()), Err(CoreError::Invalid("practice_mode_needed"))));
    }

    #[test]
    fn key_needs_a_running_band_and_short_text() {
        let c = practice_core();
        assert!(matches!(c.practice_key(&PracticeSignal { text: "CQ".into(), ..Default::default() }), Err(CoreError::Invalid("practice_not_running"))));
        c.practice_start(false, 0.5, 1, false).unwrap();
        let long = PracticeSignal { text: "E ".repeat(150), ..Default::default() };
        assert!(matches!(c.practice_key(&long), Err(CoreError::Invalid("message_too_long"))));
        c.practice_stop();
    }

    #[test]
    fn a_station_is_heard_decoded_and_reported_done() {
        let c = practice_core();
        c.practice_start(true, 0.5, 3, false).unwrap();
        assert!(c.practice_running());
        let id = c.practice_key(&PracticeSignal { text: "PA3XYZ".into(), wpm: 28.0, snr_db: 20.0, delay_s: 0.2, ..Default::default() }).unwrap();
        let mut text = String::new();
        let mut done = false;
        let until = Instant::now() + Duration::from_secs(10);
        while Instant::now() < until && !(done && text.contains("PA3XYZ")) {
            for ev in c.audio_tick() {
                match ev {
                    CoreEvent::Decode { text: t, .. } => text.push_str(&t),
                    CoreEvent::PracticeDone { ids } => done |= ids.contains(&id),
                    _ => {}
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(done);
        assert!(text.contains("PA3XYZ"), "{text:?}");
        c.practice_stop();
        assert!(!c.practice_running());
        assert!(!c.audio_status().running);
    }

    #[test]
    fn leaving_practice_mode_stops_the_band() {
        let c = practice_core();
        c.practice_start(false, 0.5, 1, false).unwrap();
        c.set_policy(TxPolicy::ListenOnly, false).unwrap();
        assert!(!c.practice_running());
        assert!(!c.audio_status().running);
    }
}
