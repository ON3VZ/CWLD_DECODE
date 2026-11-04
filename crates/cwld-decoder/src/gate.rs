//! Is there keyed CW at all? A check on the shape of the envelope, apart from the level.
//!
//! Noise through a narrow filter makes an envelope that wanders up and down. Its crossings of
//! a middle level are many and irregular. Keyed CW holds one of two levels for whole elements
//! of a few set lengths. The gate looks at the thresholded envelope over the last seconds.

use cwld_dsp::FRAME_RATE;
use std::collections::VecDeque;

const WINDOW: usize = 1000; // 5 s of frames
const MIN_WINDOW: usize = 300; // 1.5 s before a first judgement
const EVERY: usize = 50;

#[derive(Clone, Copy, Debug, Default)]
pub struct GateStats {
    /// Marks per second divided by the filter bandwidth in Hz.
    pub marks_per_bw: f32,
    /// Share of marks shorter than about one noise correlation time.
    pub short_frac: f32,
    /// Share of time the envelope is above the middle level.
    pub duty: f32,
    pub marks: usize,
}

pub struct EnvelopeGate {
    hist: VecDeque<f32>,
    since: usize,
    pub stats: GateStats,
    /// Number of measurements made so far.
    pub evals: u64,
}

impl EnvelopeGate {
    pub fn new() -> Self {
        EnvelopeGate { hist: VecDeque::new(), since: 0, stats: GateStats::default(), evals: 0 }
    }

    pub fn push(&mut self, env: &[f32], bandwidth_hz: f64) {
        for &e in env {
            self.hist.push_back(e);
            if self.hist.len() > WINDOW {
                self.hist.pop_front();
            }
        }
        self.since += env.len();
        if self.since >= EVERY && self.hist.len() >= MIN_WINDOW {
            self.since = 0;
            self.stats = measure(self.hist.make_contiguous(), bandwidth_hz);
            self.evals += 1;
        }
    }
}

fn measure(w: &[f32], bw: f64) -> GateStats {
    let mut v: Vec<f32> = w.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let lo = v[(v.len() as f64 * 0.25) as usize];
    let hi = v[(v.len() as f64 * 0.97) as usize];
    let thr = 0.5 * (lo + hi);
    let tau = FRAME_RATE / bw.max(1.0);
    let mut marks = 0usize;
    let mut short = 0usize;
    let mut on = 0usize;
    let mut run = 0usize;
    for &e in w {
        if e > thr {
            on += 1;
            run += 1;
        } else if run > 0 {
            marks += 1;
            if (run as f64) < tau {
                short += 1;
            }
            run = 0;
        }
    }
    let secs = w.len() as f64 / FRAME_RATE;
    GateStats { marks_per_bw: (marks as f64 / secs / bw.max(1.0)) as f32, short_frac: if marks > 0 { short as f32 / marks as f32 } else { 0.0 }, duty: on as f32 / w.len() as f32, marks }
}

/// Noise: nearly every mark is a blip shorter than the filter's correlation time. Measured on
/// noise recordings and white noise: 0.96 to 1.00. Keyed CW at +3 dB and better: 0.0 to 0.4.
pub const NOISE_SHORT_FRAC: f32 = 0.95;
/// Back above this share the gate opens again (hysteresis).
pub const OPEN_SHORT_FRAC: f32 = 0.90;
/// Measurements in a row that must look like noise before the gate closes (0.25 s each).
pub const CLOSE_AFTER: u32 = 3;

#[cfg(test)]
mod tests {
    use crate::bench::decode_samples;
    use crate::{Pipeline, PipelineConfig};
    use cwld_dsp::{generate, CwSpec};

    #[test]
    fn noise_gives_no_text_and_no_signal() {
        for seed in [1u64, 2, 3] {
            let g = generate(&CwSpec { text: "".into(), snr_db: Some(0.0), lead_s: 40.0, trail_s: 0.0, seed, ..Default::default() });
            let text = decode_samples(&g.samples, PipelineConfig::new(g.sample_rate, 700.0));
            assert!(text.replace(' ', "").len() <= 3, "seed {seed}: {text:?}");
            let mut p = Pipeline::new(PipelineConfig::new(g.sample_rate, 700.0));
            p.push(&g.samples);
            p.poll();
            assert!(!p.cw_present());
        }
    }

    #[test]
    fn keyed_cw_opens_the_gate_even_after_noise() {
        let g = generate(&CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K".into(), wpm: 20.0, snr_db: Some(6.0), lead_s: 20.0, trail_s: 2.0, ..Default::default() });
        let text = decode_samples(&g.samples, PipelineConfig::new(g.sample_rate, 700.0));
        assert!(text.contains("ON3VZ"), "{text:?}");
    }

    #[test]
    fn the_gate_can_be_switched_off() {
        let mut c = PipelineConfig::new(8000, 700.0);
        c.cw_gate = false;
        assert!(Pipeline::new(c).cw_present());
    }
}
