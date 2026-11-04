//! Finds the strongest keyed tone in the audio, so the decoder can move to a signal by itself
//! (the operator does not have to click it first).
//!
//! Every quarter second the last 1024 samples are measured at fixed pitches (Goertzel, one bin
//! every 7.8 Hz at 8 kHz). Each bin keeps a running mean of its power (about 1.7 s), which
//! smooths the noise a lot while a keyed tone keeps about half its power. The score of a bin is
//! its mean over the median of its neighbourhood (30 to 160 Hz away on both sides), in dB. The
//! neighbourhood follows the shape of the radio's filter, so coloured noise does not look like
//! a signal. White or coloured noise scores up to about 8 dB, a keyed tone at 0 dB SNR about 12.

use std::f64::consts::PI;

const SIZE: usize = 1024;
/// Weight of a new measurement in the running mean (one every 0.25 s): about 1.7 s.
const ALPHA: f64 = 0.15;
/// Measurements before the first candidate is reported (1 s).
const WARMUP_UPDATES: u32 = 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    pub hz: f64,
    /// Peak over median, in dB.
    pub score_db: f64,
    /// How many updates in a row the best bin stayed at (about) this pitch.
    pub steady: u32,
}

pub struct ToneSeeker {
    sample_rate: f64,
    buf: Vec<f32>,
    pos: usize,
    filled: usize,
    since: usize,
    hop: usize,
    freqs: Vec<f64>,
    coeffs: Vec<f64>,
    window: Vec<f64>,
    peak: Vec<f64>,
    score: Vec<f64>,
    /// The local noise level per bin (median of the neighbourhood of the running mean).
    floor: Vec<f64>,
    /// The last few raw measurements, newest last, for a quick look at what happens now.
    recent: std::collections::VecDeque<Vec<f64>>,
    noise: f64,
    updates: u32,
    best: Option<Candidate>,
}

impl ToneSeeker {
    pub fn new(sample_rate: u32, lo_hz: f64, hi_hz: f64) -> Self {
        let fs = sample_rate as f64;
        let res = fs / SIZE as f64;
        let mut freqs = Vec::new();
        let mut f = (lo_hz / res).ceil() * res;
        while f <= hi_hz {
            freqs.push(f);
            f += res;
        }
        let coeffs = freqs.iter().map(|f| 2.0 * (2.0 * PI * f / fs).cos()).collect();
        let window = (0..SIZE).map(|i| 0.5 - 0.5 * (2.0 * PI * i as f64 / (SIZE - 1) as f64).cos()).collect();
        let n = freqs.len();
        ToneSeeker {
            sample_rate: fs,
            buf: vec![0.0; SIZE],
            pos: 0,
            filled: 0,
            since: 0,
            hop: (fs / 4.0) as usize,
            freqs,
            coeffs,
            window,
            peak: vec![0.0; n],
            score: vec![0.0; n],
            floor: vec![0.0; n],
            recent: std::collections::VecDeque::new(),
            noise: 0.0,
            updates: 0,
            best: None,
        }
    }

    /// Feeds audio. Returns true when a new measurement gave a candidate.
    pub fn push(&mut self, samples: &[f32]) -> bool {
        let mut updated = false;
        for &x in samples {
            self.buf[self.pos] = x;
            self.pos = (self.pos + 1) % SIZE;
            self.filled = (self.filled + 1).min(SIZE);
            self.since += 1;
            if self.since >= self.hop && self.filled == SIZE {
                self.since = 0;
                self.update();
                updated = self.best.is_some();
            }
        }
        updated
    }

    fn update(&mut self) {
        let mut power = Vec::with_capacity(self.freqs.len());
        for &c in &self.coeffs {
            let (mut s1, mut s2) = (0.0f64, 0.0f64);
            for i in 0..SIZE {
                let x = self.buf[(self.pos + i) % SIZE] as f64 * self.window[i];
                let s0 = x + c * s1 - s2;
                s2 = s1;
                s1 = s0;
            }
            power.push((s1 * s1 + s2 * s2 - c * s1 * s2).max(0.0));
        }
        let first = self.noise == 0.0;
        for (p, &now) in self.peak.iter_mut().zip(&power) {
            *p = if first { now } else { *p + ALPHA * (now - *p) };
        }
        let n = self.peak.len();
        self.floor = (0..n).map(|k| local_median(&self.peak, k).max(1e-20)).collect();
        self.score = (0..n).map(|k| self.peak[k] / self.floor[k]).collect();
        self.recent.push_back(power);
        if self.recent.len() > RECENT {
            self.recent.pop_front();
        }
        self.noise = 1.0;
        let (k, _) = self.score.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap();
        let pk = self.score[k];
        // Parabolic interpolation between the neighbouring bins for a finer pitch.
        let hz = if k > 0 && k + 1 < self.peak.len() {
            let (a, b, c) = (self.peak[k - 1].max(1e-30).ln(), self.peak[k].max(1e-30).ln(), self.peak[k + 1].max(1e-30).ln());
            let d = a - 2.0 * b + c;
            let off = if d.abs() > 1e-12 { (0.5 * (a - c) / d).clamp(-0.5, 0.5) } else { 0.0 };
            self.freqs[k] + off * (self.sample_rate / SIZE as f64)
        } else {
            self.freqs[k]
        };
        let score_db = 10.0 * pk.max(1e-20).log10();
        self.updates += 1;
        if self.updates < WARMUP_UPDATES {
            // The running means need a moment before they say anything.
            return;
        }
        let steady = match self.best {
            Some(b) if (b.hz - hz).abs() <= 20.0 => b.steady + 1,
            _ => 1,
        };
        self.best = Some(Candidate { hz, score_db, steady });
    }

    /// The strongest tone right now, if any measurement was made yet.
    pub fn best(&self) -> Option<Candidate> {
        self.best
    }

    /// Score in dB of the bin closest to `hz` (peak over median).
    pub fn score_at(&self, hz: f64) -> Option<f64> {
        if self.noise == 0.0 {
            return None;
        }
        let k = self.freqs.iter().enumerate().min_by(|a, b| (a.1 - hz).abs().partial_cmp(&(b.1 - hz).abs()).unwrap())?.0;
        let lo = k.saturating_sub(1);
        let hi = (k + 1).min(self.peak.len() - 1);
        let pk = self.score[lo..=hi].iter().cloned().fold(0.0, f64::max);
        Some(10.0 * pk.max(1e-20).log10())
    }

    /// The loudest moment of the last second at `hz` over the local noise, in dB. Unlike the
    /// running mean this drops quickly when a station stops.
    pub fn recent_score_at(&self, hz: f64) -> Option<f64> {
        if self.recent.is_empty() || self.noise == 0.0 {
            return None;
        }
        let k = self.freqs.iter().enumerate().min_by(|a, b| (a.1 - hz).abs().partial_cmp(&(b.1 - hz).abs()).unwrap())?.0;
        let lo = k.saturating_sub(1);
        let hi = (k + 1).min(self.peak.len() - 1);
        let mut best: f64 = 0.0;
        for p in &self.recent {
            for j in lo..=hi {
                best = best.max(p[j] / self.floor[j]);
            }
        }
        Some(10.0 * best.max(1e-20).log10())
    }
}

/// How many measurements `recent_score_at` looks back over (1 s).
const RECENT: usize = 4;

/// Median of the bins 4 to 20 places away from `k` (about 30 to 160 Hz), on both sides.
fn local_median(p: &[f64], k: usize) -> f64 {
    let mut v: Vec<f64> = Vec::with_capacity(34);
    for d in 4..=20usize {
        if k >= d {
            v.push(p[k - d]);
        }
        if k + d < p.len() {
            v.push(p[k + d]);
        }
    }
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gen::{generate, CwSpec};

    #[test]
    fn finds_a_keyed_signal_away_from_the_default_pitch() {
        let g = generate(&CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K".into(), tone_hz: 615.0, snr_db: Some(14.0), ..Default::default() });
        let mut s = ToneSeeker::new(8000, 300.0, 1200.0);
        let mut top: Option<Candidate> = None;
        for c in g.samples.chunks(2000) {
            if s.push(c) {
                let b = s.best().unwrap();
                if top.map_or(true, |t| b.score_db > t.score_db) {
                    top = Some(b);
                }
            }
        }
        let b = top.unwrap();
        assert!((b.hz - 615.0).abs() < 5.0, "{b:?}");
        assert!(b.score_db > 14.0, "{b:?}");
        assert!(b.steady >= 4, "{b:?}");
    }

    #[test]
    fn noise_alone_scores_low() {
        let g = generate(&CwSpec { text: "".into(), snr_db: Some(0.0), lead_s: 10.0, trail_s: 0.0, ..Default::default() });
        let mut s = ToneSeeker::new(8000, 300.0, 1200.0);
        let mut worst: f64 = 0.0;
        for c in g.samples.chunks(2000) {
            if s.push(c) {
                worst = worst.max(s.best().unwrap().score_db);
            }
        }
        assert!(worst < 10.0, "{worst}");
    }
}
