//! A small neural network as an extra source of key-down evidence (experimental).
//!
//! Per 5 ms frame it looks at the envelope and the classic log-likelihood ratio of 120 ms on each
//! side, scaled by the signal level of the last two seconds, and says how likely the key is down.
//! It has about 6000 weights, was trained on the CPU on synthetic CW of CWLD's own generator
//! (tools/nn/train.py) and runs in plain Rust. Its answer comes 240 ms late, because it looks ahead.

use crate::neural_weights as nw;
use cwld_dsp::frontend::Frame;

/// Linear-interpolated percentile of a sorted slice (the same rule the trainer uses).
fn percentile(sorted: &[f32], q: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let pos = q * (sorted.len() - 1) as f32;
    let i = pos.floor() as usize;
    let f = pos - i as f32;
    let a = sorted[i];
    let b = sorted[(i + 1).min(sorted.len() - 1)];
    a + (b - a) * f
}

pub struct NeuralLlr {
    /// Strength of the network's answer towards the decoder. Neighbouring answers share most of
    /// their evidence, so the raw log-odds would count it many times.
    pub alpha: f32,
    /// Add the network's answer to the classic evidence of the same frame instead of replacing it.
    pub mix: bool,
    env: Vec<f32>,
    llr: Vec<f32>,
    /// Envelope divided by the level, and the noise-to-signal ratio, per frame (once known).
    scaled: Vec<f32>,
    ratio: Vec<f32>,
    /// Index of `env[0]` in the whole stream, and the next frame to answer for.
    base: usize,
    next: usize,
    input: Vec<f32>,
    h1: Vec<f32>,
    h2: Vec<f32>,
}

impl NeuralLlr {
    pub fn new(alpha: f32) -> Self {
        NeuralLlr { alpha, mix: false, env: Vec::new(), llr: Vec::new(), scaled: Vec::new(), ratio: Vec::new(), base: 0, next: 0, input: vec![0.0; nw::INPUTS], h1: vec![0.0; nw::H1], h2: vec![0.0; nw::H2] }
    }

    /// Frames the answer lags behind the input.
    pub const DELAY: usize = 2 * nw::W;

    fn total(&self) -> usize {
        self.base + self.env.len()
    }

    /// Fills `scaled` and `ratio` for every frame whose level can be measured. `end`: the stream
    /// is over, measure with what there is.
    fn measure(&mut self, end: bool) {
        let n = self.total();
        loop {
            let k = self.base + self.scaled.len();
            if k >= n {
                break;
            }
            let block = k - k % nw::STEP;
            if !end && block + nw::W + 1 > n {
                break;
            }
            let a = block.saturating_sub(nw::BACK).max(self.base);
            let b = (block + nw::W + 1).min(n);
            let mut seg: Vec<f32> = self.env[a - self.base..b - self.base].to_vec();
            seg.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
            let hi = percentile(&seg, 0.9).max(1e-6);
            let lo = percentile(&seg, 0.2);
            let until = (block + nw::STEP).min(n);
            for j in k..until {
                self.scaled.push((self.env[j - self.base] / hi).clamp(0.0, 2.5));
                self.ratio.push(lo / hi);
            }
        }
    }

    fn answer(&mut self, i: usize) -> f32 {
        let w = nw::W as isize;
        let known = self.base + self.scaled.len();
        let n = self.total();
        for (slot, d) in (-w..=w).enumerate() {
            let k = i as isize + d;
            let ok = k >= self.base as isize && (k as usize) < n;
            self.input[slot] = if ok && (k as usize) < known { self.scaled[k as usize - self.base] } else { 0.0 };
            self.input[2 * nw::W + 1 + slot] = if ok { (self.llr[k as usize - self.base] / 6.0).clamp(-1.0, 1.0) } else { 0.0 };
        }
        self.input[nw::INPUTS - 1] = self.ratio[i - self.base];
        for j in 0..nw::H1 {
            let mut s = nw::B1[j];
            for (x, row) in self.input.iter().zip(nw::W1.chunks_exact(nw::H1)) {
                s += x * row[j];
            }
            self.h1[j] = s.max(0.0);
        }
        for j in 0..nw::H2 {
            let mut s = nw::B2[j];
            for (x, row) in self.h1.iter().zip(nw::W2.chunks_exact(nw::H2)) {
                s += x * row[j];
            }
            self.h2[j] = s.max(0.0);
        }
        let z: f32 = nw::B3[0] + self.h2.iter().zip(nw::W3.iter()).map(|(a, b)| a * b).sum::<f32>();
        let own = self.alpha * (z - nw::PRIOR_LOGIT);
        if self.mix {
            self.llr[i - self.base] + own
        } else {
            own
        }
    }

    /// Like `push`, but as frames: the envelope of the same (older) frame with the new evidence,
    /// so everything after the network sees one consistent, delayed stream.
    pub fn push_frames(&mut self, frames: &[Frame]) -> Vec<Frame> {
        let first = self.next;
        let out = self.push(frames);
        self.as_frames(first, out)
    }

    pub fn flush_frames(&mut self) -> Vec<Frame> {
        let first = self.next;
        let out = self.flush();
        self.as_frames(first, out)
    }

    fn as_frames(&self, first: usize, llr: Vec<f32>) -> Vec<Frame> {
        llr.into_iter().enumerate().map(|(k, l)| Frame { env: self.env.get(first + k - self.base).copied().unwrap_or(0.0), llr: l }).collect()
    }

    fn drain(&mut self, end: bool) -> Vec<f32> {
        self.measure(end);
        let n = self.total();
        let known = self.base + self.scaled.len();
        let mut out = Vec::new();
        while self.next < n && (end || (self.next + Self::DELAY < n && self.next + nw::W < known)) {
            if self.next >= known {
                break;
            }
            let i = self.next;
            out.push(self.answer(i));
            self.next += 1;
        }
        // Forget what no answer needs any more.
        let keep_from = self.next.saturating_sub(nw::BACK + nw::W + nw::STEP + 4000);
        if keep_from > self.base + 2000 {
            let cut = keep_from - self.base;
            self.env.drain(..cut);
            self.llr.drain(..cut);
            self.scaled.drain(..cut);
            self.ratio.drain(..cut);
            self.base = keep_from;
        }
        out
    }

    /// Takes new frames and returns the evidence for the frames that are now old enough.
    pub fn push(&mut self, frames: &[Frame]) -> Vec<f32> {
        for f in frames {
            self.env.push(f.env);
            self.llr.push(f.llr);
        }
        self.drain(false)
    }

    /// The stream ends: the evidence for the frames still held.
    pub fn flush(&mut self) -> Vec<f32> {
        self.drain(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keyed(n: usize, period: usize) -> Vec<Frame> {
        (0..n).map(|i| if (i / period) % 2 == 0 { Frame { env: 1.0, llr: 4.0 } } else { Frame { env: 0.05, llr: -4.0 } }).collect()
    }

    #[test]
    fn every_frame_gets_one_answer_in_order() {
        let mut nn = NeuralLlr::new(1.0);
        let frames = keyed(1000, 12);
        let mut out = Vec::new();
        for c in frames.chunks(7) {
            out.extend(nn.push(c));
        }
        assert!(out.len() <= 1000 - NeuralLlr::DELAY);
        out.extend(nn.flush());
        assert_eq!(out.len(), 1000);
    }

    #[test]
    fn a_clean_keyed_tone_is_followed() {
        let mut nn = NeuralLlr::new(1.0);
        let frames = keyed(1200, 12);
        let mut out = nn.push(&frames);
        out.extend(nn.flush());
        // Away from the edges of the marks the sign must match the key.
        let (mut ok, mut all) = (0, 0);
        for (i, z) in out.iter().enumerate().skip(100) {
            if i % 12 >= 3 && i % 12 <= 8 {
                all += 1;
                if (*z > 0.0) == ((i / 12) % 2 == 0) {
                    ok += 1;
                }
            }
        }
        assert!(ok as f32 / all as f32 > 0.97, "{ok} of {all}");
    }

    #[test]
    fn chunking_does_not_change_the_answer() {
        let frames = keyed(900, 9);
        let mut a = NeuralLlr::new(1.0);
        let mut one = a.push(&frames);
        one.extend(a.flush());
        let mut b = NeuralLlr::new(1.0);
        let mut two = Vec::new();
        for c in frames.chunks(5) {
            two.extend(b.push(c));
        }
        two.extend(b.flush());
        assert_eq!(one.len(), two.len());
        // The last frames see the end of the stream differently only when flushed, which is the same here.
        for (x, y) in one.iter().zip(two.iter()).take(800) {
            assert!((x - y).abs() < 1e-4);
        }
    }
}
