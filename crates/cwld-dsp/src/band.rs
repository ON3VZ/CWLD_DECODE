//! A live practice band: band noise plus any number of simulated stations, rendered block by
//! block. The practice simulator adds a station's transmission with `add`, the audio thread
//! asks for samples with `render` and reports finished transmissions back.
//!
//! Levels use the same SNR convention as `gen`: tone power while keyed against the noise power
//! in 500 Hz. The noise has a fixed reference level, so a station keeps its strength when the
//! noise is switched off.

use crate::gen::{clean_text, key_signal, timeline, CwSpec};
use crate::rng::Rng;
use std::f64::consts::PI;

/// Standard deviation of the band noise (about -30 dBFS over the full band).
pub const NOISE_SIGMA: f64 = 0.03;

/// One transmission of one simulated station.
#[derive(Clone, Debug, PartialEq)]
pub struct BandSignal {
    pub text: String,
    pub wpm: f64,
    /// Pitch in Hz (the audio tone you hear).
    pub tone_hz: f64,
    /// SNR in 500 Hz.
    pub snr_db: f64,
    /// Depth of slow fading, 0 none to 0.9 deep.
    pub qsb_depth: f64,
    /// Relative timing spread: 0 machine, 0.08 a good hand key.
    pub jitter: f64,
    /// Seconds from now before the first element.
    pub delay_s: f64,
    pub seed: u64,
}

impl Default for BandSignal {
    fn default() -> Self {
        BandSignal { text: String::new(), wpm: 22.0, tone_hz: 700.0, snr_db: 15.0, qsb_depth: 0.0, jitter: 0.0, delay_s: 0.0, seed: 1 }
    }
}

struct Active {
    id: u64,
    /// Sample index of the band clock where the transmission starts.
    start: u64,
    key: Vec<f32>,
    tone: f64,
    amp: f64,
    phase: f64,
    qsb_depth: f64,
    qsb_hz: f64,
    qsb_phase: f64,
}

pub struct PracticeBand {
    fs: f64,
    rng: Rng,
    noise: bool,
    /// Samples rendered so far: the band clock.
    pos: u64,
    signals: Vec<Active>,
    next_id: u64,
    finished: Vec<u64>,
}

impl std::fmt::Debug for PracticeBand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PracticeBand").field("pos", &self.pos).field("active", &self.signals.len()).finish()
    }
}

/// Tone amplitude for a given SNR in 500 Hz against the reference noise.
pub fn amplitude_for(snr_db: f64, fs: f64) -> f64 {
    let snr = 10f64.powf(snr_db / 10.0);
    (2.0 * snr * NOISE_SIGMA * NOISE_SIGMA * 1000.0 / fs).sqrt()
}

impl PracticeBand {
    pub fn new(sample_rate: u32, seed: u64, noise: bool) -> Self {
        PracticeBand { fs: sample_rate as f64, rng: Rng::new(seed), noise, pos: 0, signals: Vec::new(), next_id: 1, finished: Vec::new() }
    }

    pub fn sample_rate(&self) -> u32 {
        self.fs as u32
    }

    pub fn set_noise(&mut self, on: bool) {
        self.noise = on;
    }

    /// Queues a transmission. Returns its id, reported by `take_finished` when it is over.
    /// Text without a single morse character is finished at once.
    pub fn add(&mut self, s: &BandSignal) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let text = clean_text(&s.text);
        let spec = CwSpec { text: text.clone(), wpm: s.wpm.clamp(5.0, 60.0), jitter: s.jitter.clamp(0.0, 0.3), sample_rate: self.fs as u32, ..Default::default() };
        let mut rng = Rng::new(s.seed);
        let tl = timeline(&spec, &text, &mut rng);
        let body: f64 = tl.iter().map(|(_, d)| d).sum();
        let n = (body * self.fs) as usize;
        if n == 0 {
            self.finished.push(id);
            return id;
        }
        let key = key_signal(&tl, self.fs, 0.0, n, spec.rise_ms);
        let start = self.pos + (s.delay_s.max(0.0) * self.fs) as u64;
        self.signals.push(Active {
            id,
            start,
            key,
            tone: s.tone_hz.clamp(100.0, 3000.0),
            amp: amplitude_for(s.snr_db.clamp(-20.0, 50.0), self.fs),
            phase: rng.uniform() * 2.0 * PI,
            qsb_depth: s.qsb_depth.clamp(0.0, 0.95),
            qsb_hz: 0.08 + 0.2 * rng.uniform(),
            qsb_phase: rng.uniform() * 2.0 * PI,
        });
        id
    }

    /// Stops every station at once (the exercise ends or starts again).
    pub fn clear(&mut self) {
        for s in self.signals.drain(..) {
            self.finished.push(s.id);
        }
    }

    /// Transmissions still waiting or under way.
    pub fn active(&self) -> usize {
        self.signals.len()
    }

    /// Ids of transmissions that ended since the last call.
    pub fn take_finished(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.finished)
    }

    /// The next `n` samples of the band.
    pub fn render(&mut self, n: usize) -> Vec<f32> {
        let mut out = vec![0f64; n];
        if self.noise {
            for o in out.iter_mut() {
                *o = NOISE_SIGMA * self.rng.gauss();
            }
        }
        let fs = self.fs;
        let pos = self.pos;
        for s in self.signals.iter_mut() {
            let w = 2.0 * PI * s.tone / fs;
            for (i, o) in out.iter_mut().enumerate() {
                let t = pos + i as u64;
                if t < s.start {
                    continue;
                }
                let k = (t - s.start) as usize;
                if k >= s.key.len() {
                    break;
                }
                s.phase += w;
                let key = s.key[k] as f64;
                if key > 0.0 {
                    let sec = t as f64 / fs;
                    let fade = 1.0 - s.qsb_depth * 0.5 * (1.0 + (2.0 * PI * s.qsb_hz * sec + s.qsb_phase).sin());
                    *o += s.amp * fade * key * s.phase.sin();
                }
            }
            if s.phase > 1e6 {
                s.phase %= 2.0 * PI;
            }
        }
        self.pos += n as u64;
        let end = self.pos;
        let finished = &mut self.finished;
        self.signals.retain(|s| {
            let done = s.start + s.key.len() as u64 <= end;
            if done {
                finished.push(s.id);
            }
            !done
        });
        // A soft limit keeps a pile-up of strong stations from clipping hard.
        out.into_iter().map(|x| if x.abs() < 0.8 { x as f32 } else { (x.signum() * (0.8 + 0.2 * ((x.abs() - 0.8) / 0.2).tanh())) as f32 }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn power(x: &[f32]) -> f64 {
        x.iter().map(|s| (*s as f64).powi(2)).sum::<f64>() / x.len() as f64
    }

    #[test]
    fn a_transmission_ends_and_is_reported_once() {
        let mut b = PracticeBand::new(8000, 1, false);
        let id = b.add(&BandSignal { text: "PARIS".into(), wpm: 20.0, ..Default::default() });
        // 43 units of 60 ms.
        let x = b.render((2.5 * 8000.0) as usize);
        assert!(b.take_finished().is_empty());
        assert!(power(&x) > 0.0);
        b.render(2000);
        assert_eq!(b.take_finished(), vec![id]);
        assert!(b.take_finished().is_empty());
        assert_eq!(b.active(), 0);
    }

    #[test]
    fn delay_keeps_silence_first() {
        let mut b = PracticeBand::new(8000, 1, false);
        b.add(&BandSignal { text: "E".into(), delay_s: 1.0, ..Default::default() });
        let x = b.render(7900);
        assert!(x.iter().all(|s| *s == 0.0));
        let y = b.render(2000);
        assert!(y.iter().any(|s| *s != 0.0));
    }

    #[test]
    fn level_follows_the_snr_convention() {
        let mut b = PracticeBand::new(8000, 3, true);
        let noise = b.render(16000);
        let pn = power(&noise);
        assert!((pn - NOISE_SIGMA * NOISE_SIGMA).abs() < 0.0002, "{pn}");
        // A long dash at 10 dB: tone power is 10x the noise in 500 Hz (= sigma^2 / 8 at 8 kHz).
        let mut q = PracticeBand::new(8000, 3, false);
        q.add(&BandSignal { text: "T".into(), wpm: 5.0, snr_db: 10.0, ..Default::default() });
        let x = q.render(4000);
        let pt = power(&x[200..3800]);
        let expect = 10.0 * NOISE_SIGMA * NOISE_SIGMA / 8.0;
        assert!((pt / expect - 1.0).abs() < 0.05, "{pt} vs {expect}");
    }

    #[test]
    fn clear_finishes_everything() {
        let mut b = PracticeBand::new(8000, 1, true);
        let a = b.add(&BandSignal { text: "CQ CQ".into(), ..Default::default() });
        let c = b.add(&BandSignal { text: "TEST".into(), delay_s: 3.0, ..Default::default() });
        b.render(100);
        b.clear();
        let mut f = b.take_finished();
        f.sort();
        assert_eq!(f, vec![a, c]);
    }

    #[test]
    fn empty_text_is_finished_at_once() {
        let mut b = PracticeBand::new(8000, 1, false);
        let id = b.add(&BandSignal { text: "#".into(), ..Default::default() });
        assert_eq!(b.take_finished(), vec![id]);
    }
}
