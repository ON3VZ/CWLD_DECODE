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
    /// The sender's own spacing, in units: inside a character (1), between characters (3) and
    /// between words (7). `gap_scale` multiplies the last two on top of this.
    pub intra_gap: f64,
    pub char_gap: f64,
    pub word_gap: f64,
    /// Extra relative spread of the gaps only (a keyer makes even elements, the operator the gaps).
    pub gap_jitter: f64,
    /// Share of the dots and dashes that a short deep fade takes away completely (0.03 = three
    /// in a hundred). The text that was keyed stays the truth.
    pub lost_elements: f64,
    /// The sender changes his ways part of the way through: after this share of the words
    /// (0 = never) the gaps between characters and words and the speed become the second set.
    /// A value of 0 in the second set keeps the first.
    pub change_after: f64,
    pub char_gap2: f64,
    pub word_gap2: f64,
    pub wpm2: f64,
    /// The speed swings slowly by this share around `wpm` (0.15 = 15 percent up and down).
    pub speed_wobble: f64,
    /// Key rise and fall time in milliseconds.
    pub rise_ms: f64,
    pub lead_s: f64,
    pub trail_s: f64,
    pub interferer: Option<Interferer>,
    /// A steady unkeyed carrier: (offset from the wanted tone in Hz, level relative to the
    /// wanted signal in dB). A tuner-upper or a birdie.
    pub carrier: Option<(f64, f64)>,
    /// Impulse noise (ignition, switching supplies): clicks per second and their peak level
    /// relative to the wanted signal in dB.
    pub impulses_per_s: f64,
    pub impulse_db: f64,
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
            intra_gap: 1.0,
            char_gap: 3.0,
            word_gap: 7.0,
            gap_jitter: 0.0,
            speed_wobble: 0.0,
            change_after: 0.0,
            char_gap2: 0.0,
            word_gap2: 0.0,
            wpm2: 0.0,
            lost_elements: 0.0,
            rise_ms: 5.0,
            lead_s: 0.5,
            trail_s: 1.5,
            interferer: None,
            carrier: None,
            impulses_per_s: 0.0,
            impulse_db: 20.0,
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
pub(crate) fn timeline(spec: &CwSpec, text: &str, rng: &mut Rng) -> Vec<(bool, f64)> {
    let base = 1.2 / spec.wpm;
    let jit = |x: f64, rng: &mut Rng| (x * (1.0 + spec.jitter * rng.gauss())).max(x * 0.4);
    let gsd = (spec.jitter * spec.jitter + spec.gap_jitter * spec.gap_jitter).sqrt();
    let gjit = |x: f64, rng: &mut Rng| (x * (1.0 + gsd * rng.gauss())).max(x * 0.4);
    let mut out: Vec<(bool, f64)> = Vec::new();
    let mut n = 0usize;
    // One swing of the speed takes about 40 elements and gaps (a few words).
    let words: Vec<&str> = text.split_whitespace().collect();
    let change_at = if spec.change_after > 0.0 { (words.len() as f64 * spec.change_after).round() as usize } else { usize::MAX };
    let late = std::cell::Cell::new(false);
    let pick = |first: f64, second: f64| if late.get() && second > 0.0 { second } else { first };
    let unit = |n: &mut usize| {
        *n += 1;
        pick(base, if spec.wpm2 > 0.0 { 1.2 / spec.wpm2 } else { 0.0 }) * (1.0 + spec.speed_wobble * (2.0 * std::f64::consts::PI * *n as f64 / 40.0).sin())
    };
    for (wi, word) in words.iter().enumerate() {
        late.set(wi >= change_at);
        if wi > 0 {
            out.push((false, gjit(pick(spec.word_gap, spec.word_gap2) * unit(&mut n) * spec.gap_scale, rng)));
        }
        let chars: Vec<char> = word.chars().collect();
        for (ci, c) in chars.iter().enumerate() {
            if ci > 0 {
                out.push((false, gjit(pick(spec.char_gap, spec.char_gap2) * unit(&mut n) * spec.gap_scale, rng)));
            }
            let pat = morse::pattern(*c).unwrap_or("");
            for (ei, e) in pat.chars().enumerate() {
                if ei > 0 {
                    out.push((false, gjit(spec.intra_gap * unit(&mut n), rng)));
                }
                let len = if e == '-' { spec.dash_ratio } else { 1.0 };
                let d = jit(len * unit(&mut n), rng);
                // Lost in a fade: the time passes, the key is not heard.
                let heard = spec.lost_elements <= 0.0 || rng.uniform() >= spec.lost_elements;
                out.push((heard, d));
            }
        }
    }
    out
}

pub(crate) fn clean_text(text: &str) -> String {
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
pub(crate) fn key_signal(tl: &[(bool, f64)], fs: f64, lead: f64, n: usize, rise_ms: f64) -> Vec<f32> {
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

    if let Some((off, db)) = spec.carrier {
        let a = amp * 10f64.powf(db / 20.0);
        let mut ph = rng.uniform() * 2.0 * PI;
        for s in samples.iter_mut() {
            ph += 2.0 * PI * (spec.tone_hz + off) / fs;
            *s += (a * ph.sin()) as f32;
        }
    }

    if spec.impulses_per_s > 0.0 {
        // Poisson arrivals. Each click is a short decaying burst of random sign, about 1 ms.
        let peak = amp * 10f64.powf(spec.impulse_db / 20.0);
        let mut t = 0.0;
        loop {
            t += -(1.0 - rng.uniform()).ln() / spec.impulses_per_s;
            let start = (t * fs) as usize;
            if start >= n {
                break;
            }
            let len = (0.0015 * fs) as usize + 1;
            let sign = if rng.uniform() < 0.5 { -1.0 } else { 1.0 };
            for k in 0..len.min(n - start) {
                let decay = (-(k as f64) / (0.0003 * fs)).exp();
                let ring = if k % 2 == 0 { 1.0 } else { -0.6 };
                samples[start + k] += (sign * peak * decay * ring) as f32;
            }
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
