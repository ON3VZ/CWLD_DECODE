//! Finds the strongest keyed tone in the audio, so the decoder can move to a signal by itself
//! (the operator does not have to click it first).
//!
//! Every quarter second the last 1024 samples are measured at fixed pitches (Goertzel, one bin
//! every 7.8 Hz at 8 kHz). Each bin keeps a running mean of its power (about 1.7 s), which
//! smooths the noise a lot while a keyed tone keeps about half its power. The score of a bin is
//! its mean over the noise next to it, in dB: the median of the bins 30 to 160 Hz away on both
//! sides. On a slope (the edge of the radio's filter or of a notch) the louder side decides: a
//! tone stands above the noise on both sides, an edge only above one, so an edge does not look
//! like a signal. White or coloured noise scores up to about 8 dB, a keyed tone
//! at 0 dB SNR about 12.
//!
//! The seeker also notices a hole in the audio (see `hole`): a stretch in the middle of the
//! passband that is far quieter than both sides, as a notch filter on the radio makes.

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
    /// Bins `first..=last` are inside the range asked for; the ones outside only serve as
    /// neighbourhood, so the edges of the range are judged against noise on both sides too.
    first: usize,
    last: usize,
    /// Updates in a row with a hole in the audio, and where it was.
    hole_updates: u32,
    hole_hz: f64,
}

/// Bins measured outside the range on each side, as neighbourhood (about 160 Hz at 8 kHz).
const MARGIN_BINS: f64 = 21.0;
/// A hole: the quietest stretch is this much below the noise on both sides (20 dB).
const HOLE_RATIO: f64 = 100.0;
/// Updates in a row before a hole is reported (3 s) .
const HOLE_UPDATES: u32 = 12;

impl ToneSeeker {
    pub fn new(sample_rate: u32, lo_hz: f64, hi_hz: f64) -> Self {
        let fs = sample_rate as f64;
        let res = fs / SIZE as f64;
        let mut freqs = Vec::new();
        let start = (lo_hz / res).ceil() * res;
        let mut f = (start - MARGIN_BINS * res).max(res * 2.0);
        f = (f / res).round() * res;
        let (mut first, mut last) = (0usize, 0usize);
        while f <= hi_hz + MARGIN_BINS * res && f < fs / 2.0 - res {
            if f < lo_hz - 1e-6 {
                first = freqs.len() + 1;
            }
            if f <= hi_hz + 1e-6 {
                last = freqs.len();
            }
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
            first,
            last,
            hole_updates: 0,
            hole_hz: 0.0,
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
        self.look_for_hole();
        let (k, _) = self.score.iter().enumerate().filter(|(k, _)| *k >= self.first && *k <= self.last).max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap();
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

    /// Is there a stretch between 400 and 1000 Hz that is far quieter than the audio 200 to
    /// 350 Hz below it and above it? That is not what a receiver filter looks like: it is what a
    /// notch filter leaves behind.
    fn look_for_hole(&mut self) {
        let res = self.sample_rate / SIZE as f64;
        let n = self.peak.len();
        let med = |a: usize, b: usize, p: &[f64]| -> Option<f64> {
            if a > b || b >= p.len() {
                return None;
            }
            let mut v: Vec<f64> = p[a..=b].to_vec();
            v.sort_by(|x, y| x.partial_cmp(y).unwrap());
            Some(v[v.len() / 2])
        };
        let near = (40.0 / res).round() as usize;
        let (d1, d2) = ((200.0 / res).round() as usize, (350.0 / res).round() as usize);
        let mut found: Option<(f64, f64)> = None;
        for k in 0..n {
            let f = self.freqs[k];
            if !(400.0..=1000.0).contains(&f) || k < d2 || k + d2 >= n || k < near || k + near >= n {
                continue;
            }
            let (Some(mid), Some(left), Some(right)) = (med(k - near, k + near, &self.peak), med(k - d2, k - d1, &self.peak), med(k + d1, k + d2, &self.peak)) else {
                continue;
            };
            let depth = left.min(right) / mid.max(1e-30);
            if depth >= HOLE_RATIO && found.map_or(true, |(_, d)| depth > d) {
                found = Some((f, depth));
            }
        }
        match found {
            Some((f, _)) => {
                self.hole_updates += 1;
                self.hole_hz = f;
            }
            None => self.hole_updates = 0,
        }
    }

    /// The middle of a hole in the audio (Hz), once it has been there for a few seconds.
    pub fn hole(&self) -> Option<f64> {
        (self.hole_updates >= HOLE_UPDATES).then_some(self.hole_hz)
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

/// The noise next to bin `k`, from the bins 4 to 20 places below and above it (about 30 to
/// 160 Hz away). Normally the median of all of them. When the two sides differ by more than
/// 6 dB, the bin sits on a slope (the edge of the radio's filter or of a notch): then the louder
/// side decides, because a tone has to stand above the noise on both sides.
fn local_median(p: &[f64], k: usize) -> f64 {
    let (mut lo, mut hi): (Vec<f64>, Vec<f64>) = (Vec::with_capacity(17), Vec::with_capacity(17));
    for d in 4..=20usize {
        if k >= d {
            lo.push(p[k - d]);
        }
        if k + d < p.len() {
            hi.push(p[k + d]);
        }
    }
    let median = |v: &mut Vec<f64>| -> f64 {
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        v[v.len() / 2]
    };
    let mut all: Vec<f64> = lo.iter().chain(hi.iter()).cloned().collect();
    if all.is_empty() {
        return 0.0;
    }
    let both = median(&mut all);
    if lo.len() < 6 || hi.len() < 6 {
        return both;
    }
    let (a, b) = (median(&mut lo), median(&mut hi));
    if a.max(b) > 4.0 * a.min(b) {
        a.max(b)
    } else {
        both
    }
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

    /// Noise with the middle of the passband taken out, as a notch on the radio does: loud below
    /// 420 Hz and above 860 Hz, 40 dB down in between. `tone`: a keyed station at that pitch.
    fn notched(seconds: f64, tone: Option<f64>, seed: u64) -> Vec<f32> {
        let fs = 8000.0;
        let n = (seconds * fs) as usize;
        let mut rng = crate::Rng::new(seed);
        let mut out = vec![0f32; n];
        // Many sines with random phases make noise with exactly the wanted shape.
        let mut f = 150.0;
        while f < 1500.0 {
            let in_hole = f > 430.0 && f < 850.0;
            let amp = if in_hole { 0.00008 } else { 0.008 };
            let ph = rng.uniform() * 2.0 * PI;
            let df = f + rng.uniform() * 3.0;
            for (i, o) in out.iter_mut().enumerate() {
                *o += (amp * (2.0 * PI * df * i as f64 / fs + ph).sin()) as f32;
            }
            f += 3.7;
        }
        if let Some(t) = tone {
            let g = generate(&CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K CQ CQ DE ON3VZ ON3VZ K CQ CQ DE ON3VZ K".into(), tone_hz: t, snr_db: None, lead_s: 0.5, ..Default::default() });
            for (o, s) in out.iter_mut().zip(g.samples.iter()) {
                *o += s * 0.5;
            }
        }
        out
    }

    #[test]
    fn the_edge_of_a_notch_is_not_a_signal() {
        let mut s = ToneSeeker::new(8000, 300.0, 1200.0);
        let mut worst: Option<Candidate> = None;
        for c in notched(12.0, None, 5).chunks(2000) {
            if s.push(c) {
                let b = s.best().unwrap();
                if worst.map_or(true, |w| b.score_db > w.score_db) {
                    worst = Some(b);
                }
            }
        }
        assert!(worst.unwrap().score_db < 12.0, "an edge scored like a station: {worst:?}");
    }

    #[test]
    fn a_station_next_to_the_notch_wins_from_the_edges() {
        let mut s = ToneSeeker::new(8000, 300.0, 1200.0);
        let mut last = None;
        for c in notched(12.0, Some(891.0), 6).chunks(2000) {
            if s.push(c) {
                last = s.best();
            }
        }
        let b = last.unwrap();
        assert!((b.hz - 891.0).abs() < 8.0, "{b:?}");
        assert!(b.score_db > 14.0, "{b:?}");
    }

    #[test]
    fn a_hole_in_the_audio_is_noticed_and_normal_audio_is_not() {
        let mut s = ToneSeeker::new(8000, 300.0, 1200.0);
        for c in notched(8.0, Some(891.0), 7).chunks(2000) {
            s.push(c);
        }
        let h = s.hole().expect("the hole must be reported");
        assert!((500.0..=800.0).contains(&h), "{h}");
        // Plain noise, and a normal CW signal in noise: no hole.
        for spec in [CwSpec { text: "".into(), snr_db: Some(0.0), lead_s: 10.0, trail_s: 0.0, ..Default::default() }, CwSpec { text: "CQ CQ CQ DE ON3VZ ON3VZ ON3VZ K".into(), snr_db: Some(10.0), ..Default::default() }] {
            let mut s = ToneSeeker::new(8000, 300.0, 1200.0);
            for c in generate(&spec).samples.chunks(2000) {
                s.push(c);
                assert!(s.hole().is_none());
            }
        }
    }
}
