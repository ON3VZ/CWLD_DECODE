//! Synthetic CW for the test bench: a keyed tone with white noise, fading, drift,
//! sloppy timing and an optional second station.
//!
//! **SNR convention:** the ratio between the power of the tone while the key is down and the
//! noise power in a 500 Hz band. This is the usual amateur definition, and it makes results
//! comparable with other programs that quote "SNR in 500 Hz".

use crate::rng::Rng;
use cwld_types::morse;
use std::f64::consts::PI;

#[derive(Clone, Debug)]
pub struct Interferer {
    /// Distance from the wanted tone in Hz (can be negative).
    pub offset_hz: f64,
    /// Level relative to the wanted signal in dB (0 = equally strong).
    pub rel_db: f64,
    pub wpm: f64,
}

#[derive(Clone, Debug)]
pub struct CwSpec {
    pub text: String,
    pub wpm: f64,
    pub tone_hz: f64,
    pub sample_rate: u32,
    /// SNR in 500 Hz. `None` means no noise at all.
    pub snr_db: Option<f64>,
    /// Depth of slow fading (QSB), 0 = none, 0.8 = deep. Fading rate in `qsb_hz`.
    pub qsb_depth: f64,
    pub qsb_hz: f64,
    /// Tone drift in Hz over the whole message (positive = rising).
    pub drift_hz: f64,
    /// Dash length in units (3.0 is the standard).
    pub dash_ratio: f64,
    /// Multiplies the gaps between characters and words (1.0 standard, 1.5 = sloppy spacing).
    pub gap_scale: f64,
    /// Relative standard deviation of every element and gap (0.0 = machine, 0.1 = hand keying).
    pub jitter: f64,
    /// Key rise and fall time in milliseconds.
    pub rise_ms: f64,
    pub lead_s: f64,
    pub trail_s: f64,
    pub interferer: Option<Interferer>,
    pub seed: u64,
}

impl Default for CwSpec {
    fn default() -> Self {
        CwSpec {
            text: "CQ CQ DE ON3VZ K".to_string(),
            wpm: 20.0,
            tone_hz: 700.0,
            sample_rate: 8000,
            snr_db: None,
            qsb_depth: 0.0,
            qsb_hz: 0.2,
            drift_hz: 0.0,
            dash_ratio: 3.0,
            gap_scale: 1.0,
            jitter: 0.0,
            rise_ms: 5.0,
            lead_s: 0.5,
            trail_s: 1.5,
            interferer: None,
            seed: 1,
        }
    }
}

pub struct Generated {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    /// The text that was actually keyed (unsupported characters removed, upper case).
    pub truth: String,
}

/// Key-down timeline as (down?, seconds) pairs.
fn timeline(spec: &CwSpec, text: &str, rng: &mut Rng) -> Vec<(bool, f64)> {
    let unit = 1.2 / spec.wpm;
    let mut jit = |x: f64, rng: &mut Rng| (x * (1.0 + spec.jitter * rng.gauss())).max(x * 0.4);
    let mut out: Vec<(bool, f64)> = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();
    for (wi, word) in words.iter().enumerate() {
        if wi > 0 {
            out.push((false, jit(7.0 * unit * spec.gap_scale, rng)));
        }
        let chars: Vec<char> = word.chars().collect();
        for (ci, c) in chars.iter().enumerate() {
            if ci > 0 {
                out.push((false, jit(3.0 * unit * spec.gap_scale, rng)));
            }
            let pat = morse::pattern(*c).unwrap_or("");
            for (ei, e) in pat.chars().enumerate() {
                if ei > 0 {
                    out.push((false, jit(unit, rng)));
                }
                let len = if e == '-' { spec.dash_ratio } else { 1.0 };
                out.push((true, jit(len * unit, rng)));
            }
        }
    }
    out
}

fn clean_text(text: &str) -> String {
    let mut out = String::new();
    for w in text.to_ascii_uppercase().split_whitespace() {
        let word: String = w.chars().filter(|c| morse::pattern(*c).is_some()).collect();
        if !word.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&word);
        }
    }
    out
}

/// The key signal (0..1, with soft edges) for a timeline, `n` samples long.
fn key_signal(tl: &[(bool, f64)], fs: f64, lead: f64, n: usize, rise_ms: f64) -> Vec<f32> {
    let mut key = vec![0f32; n];
    let mut t = lead;
    for (down, d) in tl {
        if *down {
            let a = (t * fs) as usize;
            let b = (((t + d) * fs) as usize).min(n);
            for k in key.iter_mut().take(b).skip(a.min(n)) {
                *k = 1.0;
            }
        }
        t += d;
    }
    // Box smoothing gives the edges a finite rise time (no key clicks).
    let w = ((rise_ms / 1000.0 * fs) as usize).max(1);
    if w > 1 {
        let mut out = vec![0f32; n];
        let mut acc = 0f32;
        for i in 0..n + w / 2 {
            if i < n {
                acc += key[i];
            }
            if i >= w {
                acc -= key[i - w];
            }
            let o = i as isize - (w / 2) as isize;
            if o >= 0 && (o as usize) < n {
                out[o as usize] = acc / w as f32;
            }
        }
        return out;
    }
    key
}

pub fn generate(spec: &CwSpec) -> Generated {
    let mut rng = Rng::new(spec.seed);
    let fs = spec.sample_rate as f64;
    let truth = clean_text(&spec.text);
    let tl = timeline(spec, &truth, &mut rng);
    let body: f64 = tl.iter().map(|(_, d)| d).sum();
    let total = spec.lead_s + body + spec.trail_s;
    let n = (total * fs) as usize;
    let key = key_signal(&tl, fs, spec.lead_s, n, spec.rise_ms);

    let amp = 0.5f64;
    let mut samples = vec![0f32; n];
    let mut phase = 0.0f64;
    let qsb_phase = rng.uniform() * 2.0 * PI;
    for i in 0..n {
        let t = i as f64 / fs;
        let f = spec.tone_hz + spec.drift_hz * (t / total);
        phase += 2.0 * PI * f / fs;
        let fade = 1.0 - spec.qsb_depth * 0.5 * (1.0 + (2.0 * PI * spec.qsb_hz * t + qsb_phase).sin());
        samples[i] = (amp * fade * key[i] as f64 * phase.sin()) as f32;
    }

    if let Some(intf) = &spec.interferer {
        let mut irng = Rng::new(spec.seed ^ 0xABCD);
        let alphabet: Vec<char> = "ETAOINSHRDLUCMFWYPVBGKQJXZ0123456789".chars().collect();
        let mut text = String::new();
        for _ in 0..40 {
            let len = 1 + irng.below(5);
            for _ in 0..len {
                text.push(alphabet[irng.below(alphabet.len())]);
            }
            text.push(' ');
        }
        let ispec = CwSpec { wpm: intf.wpm, ..spec.clone() };
        let itl = timeline(&ispec, &clean_text(&text), &mut irng);
        let ikey = key_signal(&itl, fs, 0.0, n, spec.rise_ms);
        let iamp = amp * 10f64.powf(intf.rel_db / 20.0);
        let mut ph = 0.0f64;
        for i in 0..n {
            ph += 2.0 * PI * (spec.tone_hz + intf.offset_hz) / fs;
            samples[i] += (iamp * ikey[i] as f64 * ph.sin()) as f32;
        }
    }

    if let Some(snr) = spec.snr_db {
        // Noise in 500 Hz = sigma^2 * 500 / (fs / 2). Tone power while keyed = amp^2 / 2.
        let snr_lin = 10f64.powf(snr / 10.0);
        let sigma = ((amp * amp / 2.0) * fs / 1000.0 / snr_lin).sqrt();
        for s in samples.iter_mut() {
            *s += (sigma * rng.gauss()) as f32;
        }
    }
    Generated { samples, sample_rate: spec.sample_rate, truth }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_matches_paris_timing() {
        let spec = CwSpec { text: "PARIS".into(), wpm: 20.0, lead_s: 0.0, trail_s: 0.0, ..Default::default() };
        let g = generate(&spec);
        // PARIS is 50 units minus the final word gap of 7: 43 units at 60 ms.
        let expect = 43.0 * 0.06;
        let got = g.samples.len() as f64 / 8000.0;
        assert!((got - expect).abs() < 0.01, "{got} vs {expect}");
    }

    #[test]
    fn unsupported_characters_are_dropped_from_truth() {
        let g = generate(&CwSpec { text: "cq # de on3vz".into(), ..Default::default() });
        assert_eq!(g.truth, "CQ DE ON3VZ");
    }

    #[test]
    fn same_seed_same_signal_and_snr_is_close() {
        let spec = CwSpec { snr_db: Some(10.0), jitter: 0.1, seed: 5, ..Default::default() };
        let a = generate(&spec);
        let b = generate(&spec);
        assert_eq!(a.samples, b.samples);
        // Noise only part (the lead-in) has power sigma^2 = 0.125 * 8 / 10.
        let lead = &a.samples[..(0.4 * 8000.0) as usize];
        let p: f64 = lead.iter().map(|s| (*s as f64).powi(2)).sum::<f64>() / lead.len() as f64;
        assert!((p - 0.1).abs() < 0.02, "{p}");
    }

    #[test]
    fn noise_free_has_silent_gaps() {
        let g = generate(&CwSpec::default());
        let lead = &g.samples[..(0.4 * 8000.0) as usize];
        assert!(lead.iter().all(|s| s.abs() < 1e-6));
    }
}
