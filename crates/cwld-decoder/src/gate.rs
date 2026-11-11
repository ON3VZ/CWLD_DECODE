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

    #[test]
    fn a_fixed_speed_and_the_wide_filter_still_decode() {
        let g = generate(&CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K".into(), wpm: 20.0, snr_db: Some(15.0), lead_s: 1.0, trail_s: 2.0, ..Default::default() });
        let mut cfg = PipelineConfig::new(g.sample_rate, 700.0);
        cfg.decoder.fixed_wpm = Some(20.0);
        let text = decode_samples(&g.samples, cfg);
        assert!(text.contains("ON3VZ"), "fixed 20: {text:?}");
        let mut cfg = PipelineConfig::new(g.sample_rate, 700.0);
        cfg.bandwidth_factor = 2.4;
        let text = decode_samples(&g.samples, cfg);
        assert!(text.contains("ON3VZ"), "wide: {text:?}");
    }

    #[test]
    fn fast_fading_is_read_better_with_the_fading_switch_on() {
        // The level swings between full and a third (10 dB) about once a second, in little noise:
        // every mark is far above the noise, but many are far below the level of their neighbours.
        let text = "CQ CQ DE ON3VZ ON3VZ K";
        let (mut on, mut off) = (0, 0);
        for seed in 1..=6 {
            let g = generate(&CwSpec { text: text.into(), snr_db: Some(25.0), qsb_depth: 0.67, qsb_hz: 1.1, lead_s: 1.0, trail_s: 2.0, seed, ..Default::default() });
            let read = |fading: bool| {
                let mut cfg = PipelineConfig::new(g.sample_rate, 700.0);
                cfg.fading = fading;
                crate::bench::levenshtein(text, &decode_samples(&g.samples, cfg))
            };
            on += read(true);
            off += read(false);
        }
        println!("errors in {} characters: on {on}, off {off}", 6 * text.len());
        assert!(on < off, "on {on} off {off}");
    }

    #[test]
    fn a_speed_range_keeps_the_decoder_inside_it() {
        // Fast keying at 40 WPM, decoder told to look only between 5 and 25 WPM: the speed it
        // works with never leaves that range. (Since the decoder follows the sender's style it
        // may still read such text, as very light keying at 25 WPM.) With the full range it
        // finds 40 WPM.
        let g = generate(&CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K".into(), wpm: 40.0, snr_db: Some(20.0), lead_s: 1.0, trail_s: 2.0, ..Default::default() });
        let run = |max: f64| {
            let mut cfg = PipelineConfig::new(g.sample_rate, 700.0);
            cfg.decoder.wpm_max = max;
            let mut p = crate::Pipeline::new(cfg);
            let (mut text, mut top) = (String::new(), 0.0f64);
            for c in g.samples.chunks(200) {
                p.push(c);
                text.push_str(&p.poll().committed);
                top = top.max(p.wpm().unwrap_or(0.0));
            }
            (text, top)
        };
        let (_, top) = run(25.0);
        assert!(top <= 25.5, "narrow: {top}");
        let (full, top) = run(50.0);
        assert!(full.contains("ON3VZ"), "full: {full:?}");
        assert!((34.0..=46.0).contains(&top), "full: {top}");
    }
}

#[cfg(test)]
mod measure {
    use crate::bench::decode_samples;
    use crate::PipelineConfig;
    use cwld_dsp::{generate, generate_voice, CwSpec, VoiceSpec};

    fn cfg(fs: u32) -> PipelineConfig {
        let mut c = PipelineConfig::new(fs, 700.0);
        c.auto_tone = true;
        c.min_separation = 3.5;
        c
    }

    /// Prints how much text comes out of speech, noise and very weak CW. Run with --ignored --nocapture.
    #[test]
    #[ignore]
    fn print_false_text() {
        for snr in [30.0, 20.0, 10.0, 3.0] {
            for shift in [0.0, 150.0] {
                let mut tot = 0;
                for seed in 1..=5u64 {
                    let v = generate_voice(&VoiceSpec { seconds: 60.0, snr_db: Some(snr), shift_hz: shift, seed, ..Default::default() });
                    let t = decode_samples(&v, cfg(8000));
                    tot += t.replace(' ', "").len();
                    if seed == 1 { println!("voice snr {snr} shift {shift}: {t:?}"); }
                }
                println!("VOICE snr {snr:>4} shift {shift:>4}: {:.1} chars/min", tot as f64 / 5.0);
            }
        }
        for snr in [-6.0, -10.0, -14.0, -18.0] {
            let mut cer = 0.0;
            let mut out = 0;
            for seed in 1..=5u64 {
                let txt = "CQ CQ DE DL1ABC DL1ABC K UR 599 TNX 73";
                let g = generate(&CwSpec { text: txt.into(), wpm: 20.0, snr_db: Some(snr), lead_s: 2.0, trail_s: 2.0, seed, ..Default::default() });
                let t = decode_samples(&g.samples, cfg(g.sample_rate));
                cer += crate::bench::levenshtein(txt, &t) as f64 / txt.len() as f64;
                out += t.replace(' ', "").len();
                if seed == 1 { println!("cw {snr}: {t:?}"); }
            }
            println!("CW snr {snr:>5}: CER {:.2}, chars out {:.1}", cer / 5.0, out as f64 / 5.0);
        }
    }
}

#[cfg(test)]
mod voice_gate {
    use crate::bench::decode_samples;
    use crate::PipelineConfig;
    use cwld_dsp::{generate_voice, VoiceSpec};

    #[test]
    fn speech_gives_almost_no_text_and_says_so() {
        let v = generate_voice(&VoiceSpec { seconds: 30.0, snr_db: Some(20.0), seed: 3, ..Default::default() });
        let mut c = PipelineConfig::new(8000, 700.0);
        c.auto_tone = true;
        let text = decode_samples(&v, c.clone());
        assert!(text.replace(' ', "").len() <= 3, "{text:?}");
        c.voice_gate = false;
        let without = decode_samples(&v, c);
        assert!(without.replace(' ', "").len() > 10, "the test signal must fool the decoder without the gate: {without:?}");
        let mut p = crate::Pipeline::new(PipelineConfig::new(8000, 700.0));
        p.push(&v);
        assert!(p.voice_heard());
    }
}
