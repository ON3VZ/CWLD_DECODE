//! Synthetic SSB speech for the test bench, so the decoder can be checked on phone signals.
//!
//! This is not real speech. It has what matters for a CW decoder: a voiced source with a pitch
//! that glides between about 90 and 220 Hz, harmonics shaped by two moving formants, syllables
//! of 80 to 300 ms at about four per second, short pauses, some unvoiced hiss, and the
//! 300 to 2700 Hz passband of an SSB receiver. Noise is added with the same "SNR in 500 Hz"
//! convention as the CW generator, using the mean speech power while talking.

use crate::rng::Rng;
use std::f64::consts::PI;

#[derive(Clone, Debug)]
pub struct VoiceSpec {
    pub seconds: f64,
    pub sample_rate: u32,
    /// SNR in 500 Hz. `None`: no noise.
    pub snr_db: Option<f64>,
    /// Shifts every frequency (an SSB signal tuned off by this many Hz).
    pub shift_hz: f64,
    pub seed: u64,
}

impl Default for VoiceSpec {
    fn default() -> Self {
        VoiceSpec { seconds: 30.0, sample_rate: 8000, snr_db: Some(10.0), shift_hz: 0.0, seed: 1 }
    }
}

/// One syllable: start, length, start and end pitch, formants, voiced or hiss.
struct Syl {
    start: f64,
    len: f64,
    f0a: f64,
    f0b: f64,
    f1: f64,
    f2: f64,
    voiced: bool,
}

fn formant_gain(f: f64, centre: f64, width: f64) -> f64 {
    let x = (f - centre) / width;
    1.0 / (1.0 + x * x)
}

pub fn generate_voice(spec: &VoiceSpec) -> Vec<f32> {
    let fs = spec.sample_rate as f64;
    let n = (spec.seconds * fs) as usize;
    let mut rng = Rng::new(spec.seed);
    // Syllables in phrases of 1 to 4 s with pauses between them.
    let mut syls = Vec::new();
    let mut t = 0.2;
    while t < spec.seconds {
        let phrase_end = t + 1.0 + rng.uniform() * 3.0;
        let mut f0 = 100.0 + rng.uniform() * 80.0;
        while t < phrase_end.min(spec.seconds) {
            let len = 0.08 + rng.uniform() * 0.22;
            let f0b = (f0 + (rng.uniform() - 0.5) * 60.0).clamp(90.0, 220.0);
            syls.push(Syl { start: t, len, f0a: f0, f0b, f1: 350.0 + rng.uniform() * 450.0, f2: 900.0 + rng.uniform() * 1300.0, voiced: rng.uniform() > 0.15 });
            f0 = f0b;
            t += len + 0.02 + rng.uniform() * 0.08;
        }
        t += 0.15 + rng.uniform() * 0.6;
    }
    let mut out = vec![0f32; n];
    let mut phase = 0.0f64;
    let mut si = 0usize;
    for (i, o) in out.iter_mut().enumerate() {
        let t = i as f64 / fs;
        while si < syls.len() && syls[si].start + syls[si].len < t {
            si += 1;
        }
        let Some(s) = syls.get(si).filter(|s| s.start <= t) else { continue };
        let p = (t - s.start) / s.len;
        // Raised cosine syllable envelope with 20 to 40 ms edges.
        let env = (PI * p).sin().powf(0.6);
        if s.voiced {
            let f0 = s.f0a + (s.f0b - s.f0a) * p;
            phase += 2.0 * PI * f0 / fs;
            let mut v = 0.0;
            let mut k = 1.0;
            while k * f0 + spec.shift_hz < 3000.0 {
                let f = k * f0 + spec.shift_hz;
                if f > 300.0 && f < 2700.0 {
                    let g = formant_gain(f, s.f1, 120.0) + 0.6 * formant_gain(f, s.f2, 180.0) + 0.02;
                    // A shifted harmonic keeps its own phase track: k times the base phase plus the shift.
                    v += g * (k * phase + 2.0 * PI * spec.shift_hz * t).sin();
                }
                k += 1.0;
            }
            *o = (env * v) as f32;
        } else {
            *o = (env * 0.4 * rng.gauss()) as f32;
        }
    }
    // Scale talk to a mean power of 1 (over the samples where something is said).
    let talk: Vec<f32> = out.iter().cloned().filter(|x| *x != 0.0).collect();
    let pow = talk.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / talk.len().max(1) as f64;
    let g = 1.0 / pow.sqrt().max(1e-9);
    for x in out.iter_mut() {
        *x = (*x as f64 * g) as f32;
    }
    if let Some(snr) = spec.snr_db {
        // Speech power spread over about 2400 Hz: the part in 500 Hz sets the SNR.
        let sig_in_500 = 500.0 / 2400.0;
        let noise_in_500 = sig_in_500 / 10f64.powf(snr / 10.0);
        let sd = (noise_in_500 * (fs / 2.0) / 500.0).sqrt();
        for x in out.iter_mut() {
            *x += (rng.gauss() * sd) as f32;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_has_talk_and_pauses() {
        let v = generate_voice(&VoiceSpec { snr_db: None, seconds: 10.0, ..Default::default() });
        let silent = v.iter().filter(|x| **x == 0.0).count() as f64 / v.len() as f64;
        assert!(silent > 0.1 && silent < 0.7, "{silent}");
    }
}
