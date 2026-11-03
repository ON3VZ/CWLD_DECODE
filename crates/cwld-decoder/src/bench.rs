//! The test bench: how many characters does the decoder get wrong, per SNR and speed?
//!
//! The measure is the character error rate: the edit distance between the keyed text and the
//! decoded text, divided by the length of the keyed text. Spaces count as characters.

use crate::pipeline::{Pipeline, PipelineConfig};
use cwld_dsp::{generate, CwSpec, Rng};

/// Typical amateur QSO phrases, with callsigns, numbers and common abbreviations.
pub const TEXTS: &[&str] = &[
    "CQ CQ CQ DE ON3VZ ON3VZ K",
    "ON3VZ DE DL1ABC UR 579 579 BK",
    "R TNX FER CALL OM NAME IS HANS QTH BERLIN",
    "RIG IC7300 PWR 5W ANT DIPOLE",
    "WX COLD AND RAIN TEMP 4 C",
    "73 ES GL DX DE ON3VZ SK",
    "QRZ DE F5XYZ K",
    "UR RST 599 IN HOBOKEN HW CPY",
    "TU 5NN 14 ON6WL",
    "AGN PSE QRS QRS",
];

pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let sub = prev[j - 1] + usize::from(a[i - 1] != b[j - 1]);
            cur[j] = sub.min(prev[j] + 1).min(cur[j - 1] + 1);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Decodes a recording from start to end and returns the text.
pub fn decode_samples(samples: &[f32], cfg: PipelineConfig) -> String {
    let mut p = Pipeline::new(cfg);
    let mut text = String::new();
    // Feed in chunks of 25 ms, like an audio callback would.
    let chunk = (p.frame_rate_samples()).max(1);
    for c in samples.chunks(chunk) {
        p.push(c);
        text.push_str(&p.poll().committed);
    }
    p.flush();
    text.push_str(&p.poll().committed);
    text.trim().to_string()
}

impl Pipeline {
    fn frame_rate_samples(&self) -> usize {
        // 25 ms of audio.
        (self.sample_rate() as f64 * 0.025) as usize
    }
}

#[derive(Clone, Debug)]
pub struct Cell {
    pub cer: f64,
    pub chars: usize,
    pub errors: usize,
}

/// Runs `trials` messages through the decoder with the signal settings in `base`.
pub fn run_cell(base: &CwSpec, trials: usize, seed: u64, pcfg: impl Fn(&CwSpec) -> PipelineConfig) -> Cell {
    let mut rng = Rng::new(seed);
    let (mut chars, mut errors) = (0usize, 0usize);
    for t in 0..trials {
        let text = TEXTS[rng.below(TEXTS.len())];
        let spec = CwSpec { text: text.to_string(), seed: seed.wrapping_mul(1000).wrapping_add(t as u64), ..base.clone() };
        let g = generate(&spec);
        let decoded = decode_samples(&g.samples, pcfg(&spec));
        chars += g.truth.chars().count();
        errors += levenshtein(&g.truth, &decoded);
    }
    Cell { cer: errors as f64 / chars.max(1) as f64, chars, errors }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_distance() {
        assert_eq!(levenshtein("CQ DE", "CQ DE"), 0);
        assert_eq!(levenshtein("CQ DE", "CQ D"), 1);
        assert_eq!(levenshtein("ABC", "AXC"), 1);
        assert_eq!(levenshtein("", "AB"), 2);
    }
}
