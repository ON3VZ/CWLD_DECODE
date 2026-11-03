//! From audio samples to "is the key down?" evidence.
//!
//! The chain: mix the wanted tone down to 0 Hz, low-pass it (narrow, so noise is removed),
//! take the envelope at `FRAME_RATE` frames per second, and turn every frame into a
//! **log-likelihood ratio** (positive = key down is more likely). The decoder does not
//! get a hard on/off decision, it gets this soft evidence.
//!
//! The tone is tracked (AFC) by watching the phase rotation of the baseband signal while the
//! key is down. The noise and signal levels come from a two-cluster fit over the last
//! seconds, so there is no fixed threshold to tune.

use std::f64::consts::PI;

/// Frames per second handed to the decoder.
pub const FRAME_RATE: f64 = 200.0;
/// The presence test needs this long to settle before it may say yes.
const WARMUP_SECONDS: f64 = 2.0;
const STATS_FRAMES: usize = 800;
const STATS_EVERY: usize = 20;
const MIN_STATS_FRAMES: usize = 100;

#[derive(Clone, Debug)]
pub struct FrontendConfig {
    pub sample_rate: u32,
    pub tone_hz: f64,
    /// Width of the low-pass filter in Hz (per side of the tone).
    pub bandwidth_hz: f64,
    pub afc: bool,
    /// The AFC never leaves `tone_hz` plus or minus this many Hz.
    pub afc_range_hz: f64,
}

impl FrontendConfig {
    pub fn new(sample_rate: u32, tone_hz: f64) -> Self {
        FrontendConfig { sample_rate, tone_hz, bandwidth_hz: 40.0, afc: true, afc_range_hz: 60.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub env: f32,
    /// Log-likelihood ratio key down versus key up.
    pub llr: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Levels {
    pub noise: f32,
    pub signal: f32,
    pub noise_sd: f32,
    pub signal_sd: f32,
    /// Separation of the two levels in noise standard deviations. Below about 2 there is no
    /// usable signal.
    pub separation: f32,
}

/// One mixer and low-pass: shifts a frequency band down to 0 Hz and filters it.
#[derive(Clone, Default)]
struct Chain {
    phase: f64,
    lp: [[f64; 2]; 3],
}

impl Chain {
    fn run(&mut self, x: f32, f: f64, fs: f64, alpha: f64) -> (f64, f64) {
        self.phase += 2.0 * PI * f / fs;
        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
        let (s, c) = self.phase.sin_cos();
        let mut i = x as f64 * c;
        let mut q = -(x as f64) * s;
        for sec in self.lp.iter_mut() {
            sec[0] += alpha * (i - sec[0]);
            sec[1] += alpha * (q - sec[1]);
            i = sec[0];
            q = sec[1];
        }
        (i, q)
    }
}

pub struct Frontend {
    cfg: FrontendConfig,
    main: Chain,
    refs: [Chain; 2],
    /// Slow averages of the power in the wanted band and in two reference bands beside it.
    pow_main: f64,
    pow_ref: [f64; 2],
    present: bool,
    frames_seen: u64,
    f_now: f64,
    alpha: f64,
    sample_count: u64,
    next_frame_at: f64,
    hist: Vec<f32>,
    hist_pos: usize,
    hist_len: usize,
    since_stats: usize,
    levels: Levels,
    have_levels: bool,
    prev_z: Option<(f64, f64)>,
    afc_acc: (f64, f64),
    afc_frames: usize,
}

impl Frontend {
    pub fn new(cfg: FrontendConfig) -> Self {
        let mut f = Frontend {
            f_now: cfg.tone_hz,
            alpha: 0.0,
            cfg,
            main: Chain::default(),
            refs: [Chain::default(), Chain::default()],
            pow_main: 0.0,
            pow_ref: [0.0; 2],
            present: false,
            frames_seen: 0,
            sample_count: 0,
            next_frame_at: 0.0,
            hist: vec![0.0; STATS_FRAMES],
            hist_pos: 0,
            hist_len: 0,
            since_stats: 0,
            levels: Levels::default(),
            have_levels: false,
            prev_z: None,
            afc_acc: (0.0, 0.0),
            afc_frames: 0,
        };
        f.set_bandwidth(f.cfg.bandwidth_hz);
        f
    }

    pub fn set_bandwidth(&mut self, hz: f64) {
        self.cfg.bandwidth_hz = hz.clamp(8.0, 90.0);
        // Three one-pole sections in a row: the cutoff of the whole chain is lower than the
        // cutoff of one section, so scale it up.
        let fc = self.cfg.bandwidth_hz * 1.9;
        self.alpha = 1.0 - (-2.0 * PI * fc / self.cfg.sample_rate as f64).exp();
    }

    pub fn bandwidth_hz(&self) -> f64 {
        self.cfg.bandwidth_hz
    }

    /// The tone the front-end is listening to now (after AFC).
    pub fn tone_hz(&self) -> f64 {
        self.f_now
    }

    /// Moves the listening tone (for example after the operator clicked on the waterfall).
    pub fn retune(&mut self, hz: f64) {
        self.cfg.tone_hz = hz;
        self.f_now = hz;
        self.reset_stats();
    }

    pub fn levels(&self) -> Option<Levels> {
        self.have_levels.then_some(self.levels)
    }

    fn reset_stats(&mut self) {
        self.hist_len = 0;
        self.hist_pos = 0;
        self.since_stats = 0;
        self.have_levels = false;
        self.prev_z = None;
    }

    pub fn push(&mut self, samples: &[f32]) -> Vec<Frame> {
        let fs = self.cfg.sample_rate as f64;
        let step = fs / FRAME_RATE;
        let mut out = Vec::with_capacity(samples.len() / step as usize + 1);
        let ref_off = (self.cfg.bandwidth_hz * 4.0).max(110.0);
        for &x in samples {
            let (i, q) = self.main.run(x, self.f_now, fs, self.alpha);
            let (ia, qa) = self.refs[0].run(x, self.f_now - ref_off, fs, self.alpha);
            let (ib, qb) = self.refs[1].run(x, self.f_now + ref_off, fs, self.alpha);
            self.sample_count += 1;
            if self.sample_count as f64 >= self.next_frame_at {
                self.next_frame_at += step;
                // Slow power averages (time constant about 4 s) for the presence test.
                self.frames_seen += 1;
                // The first seconds use a plain running mean, later the 4 s time constant.
                let a = (1.0 / self.frames_seen as f64).max(1.0 / (4.0 * FRAME_RATE));
                self.pow_main += a * (i * i + q * q - self.pow_main);
                self.pow_ref[0] += a * (ia * ia + qa * qa - self.pow_ref[0]);
                self.pow_ref[1] += a * (ib * ib + qb * qb - self.pow_ref[1]);
                let r = self.presence_ratio();
                if self.sample_count < (WARMUP_SECONDS * self.cfg.sample_rate as f64) as u64 {
                    self.present = false;
                } else if self.present && r < 1.15 {
                    self.present = false;
                } else if !self.present && r > 1.3 {
                    self.present = true;
                }
                out.push(self.frame(i, q));
            }
        }
        out
    }

    /// Power in the wanted band divided by the quieter of two neighbouring bands. Pure noise
    /// gives about 1, a keyed tone gives clearly more.
    pub fn presence_ratio(&self) -> f64 {
        let r = self.pow_ref[0].min(self.pow_ref[1]);
        if r <= 1e-18 {
            0.0
        } else {
            self.pow_main / r
        }
    }

    /// True while a keyed tone is judged to be present in the wanted band.
    pub fn signal_present(&self) -> bool {
        self.present
    }

    fn frame(&mut self, i: f64, q: f64) -> Frame {
        let env = (i * i + q * q).sqrt() as f32;
        self.hist[self.hist_pos] = env;
        self.hist_pos = (self.hist_pos + 1) % STATS_FRAMES;
        self.hist_len = (self.hist_len + 1).min(STATS_FRAMES);
        self.since_stats += 1;
        if self.hist_len >= MIN_STATS_FRAMES && (self.since_stats >= STATS_EVERY || !self.have_levels) {
            self.since_stats = 0;
            self.update_levels();
        }
        let llr = if self.have_levels { self.llr(env) } else { 0.0 };

        if self.cfg.afc && self.have_levels && self.levels.separation > 3.0 {
            let on = env > (self.levels.noise + self.levels.signal) * 0.5;
            if let (true, Some((pi, pq))) = (on, self.prev_z) {
                // z * conj(prev): its angle is the phase step per frame.
                self.afc_acc.0 += i * pi + q * pq;
                self.afc_acc.1 += q * pi - i * pq;
                self.afc_frames += 1;
            }
            if self.afc_frames >= 40 {
                let err_hz = self.afc_acc.1.atan2(self.afc_acc.0) * FRAME_RATE / (2.0 * PI);
                let lo = self.cfg.tone_hz - self.cfg.afc_range_hz;
                let hi = self.cfg.tone_hz + self.cfg.afc_range_hz;
                self.f_now = (self.f_now + 0.6 * err_hz).clamp(lo, hi);
                self.afc_acc = (0.0, 0.0);
                self.afc_frames = 0;
            }
        }
        self.prev_z = Some((i, q));
        Frame { env, llr }
    }

    /// Evidence for an envelope value, with the levels known now. Used to re-score frames
    /// that were kept back before the levels were known.
    pub fn llr_for(&self, env: f32) -> f32 {
        if self.have_levels {
            self.llr(env)
        } else {
            0.0
        }
    }

    fn llr(&self, env: f32) -> f32 {
        let l = &self.levels;
        if l.separation < 1.5 {
            return 0.0;
        }
        let e = env as f64;
        let (n, s) = (l.noise as f64, l.signal as f64);
        let (sn, ss) = ((l.noise_sd as f64).max(1e-9), (l.signal_sd as f64).max(1e-9));
        let on = -0.5 * ((e - s) / ss).powi(2) - ss.ln();
        let off = -0.5 * ((e - n) / sn).powi(2) - sn.ln();
        (on - off).clamp(-12.0, 12.0) as f32
    }

    /// Two-cluster fit (k-means with k = 2) of the recent envelope values.
    fn update_levels(&mut self) {
        let mut v: Vec<f32> = self.hist[..self.hist_len].to_vec();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let pick = |p: f64| v[((v.len() - 1) as f64 * p) as usize] as f64;
        let (mut lo, mut hi) = (pick(0.25), pick(0.97));
        if hi - lo < 1e-9 {
            self.have_levels = false;
            return;
        }
        let (mut sd_lo, mut sd_hi) = (0.0, 0.0);
        for _ in 0..8 {
            let mid = (lo + hi) * 0.5;
            let (mut a, mut na, mut b, mut nb) = (0.0, 0usize, 0.0, 0usize);
            for &x in &v {
                if (x as f64) < mid {
                    a += x as f64;
                    na += 1;
                } else {
                    b += x as f64;
                    nb += 1;
                }
            }
            if na == 0 || nb == 0 {
                break;
            }
            lo = a / na as f64;
            hi = b / nb as f64;
            let (mut va, mut vb) = (0.0, 0.0);
            for &x in &v {
                if (x as f64) < mid {
                    va += (x as f64 - lo).powi(2);
                } else {
                    vb += (x as f64 - hi).powi(2);
                }
            }
            sd_lo = (va / na as f64).sqrt();
            sd_hi = (vb / nb as f64).sqrt();
        }
        // The upper cluster of a key-down signal cannot be tighter than the noise.
        let sd_hi = sd_hi.max(sd_lo * 0.7);
        // A cluster fit splits pure noise too, so the separation is judged against the
        // spread of the lower cluster: pure noise gives about 2.
        let sep = (hi - lo) / sd_lo.max(1e-12);
        self.levels = Levels { noise: lo as f32, signal: hi as f32, noise_sd: sd_lo as f32, signal_sd: sd_hi as f32, separation: sep as f32 };
        self.have_levels = true;
    }
}

/// Looks for the strongest steady tone between `lo_hz` and `hi_hz` (Goertzel, 5 Hz steps).
/// Returns the frequency and how many times stronger it is than the median bin.
pub fn find_tone(samples: &[f32], sample_rate: u32, lo_hz: f64, hi_hz: f64) -> Option<(f64, f64)> {
    if samples.len() < sample_rate as usize / 4 {
        return None;
    }
    let fs = sample_rate as f64;
    let mut bins: Vec<(f64, f64)> = Vec::new();
    let mut f = lo_hz;
    while f <= hi_hz {
        let w = 2.0 * PI * f / fs;
        let coeff = 2.0 * w.cos();
        let (mut s1, mut s2) = (0.0f64, 0.0f64);
        for &x in samples {
            let s0 = x as f64 + coeff * s1 - s2;
            s2 = s1;
            s1 = s0;
        }
        bins.push((f, s1 * s1 + s2 * s2 - coeff * s1 * s2));
        f += 5.0;
    }
    let mut p: Vec<f64> = bins.iter().map(|b| b.1).collect();
    p.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = p[p.len() / 2].max(1e-18);
    let best = bins.iter().cloned().fold((0.0, 0.0), |a, b| if b.1 > a.1 { b } else { a });
    Some((best.0, best.1 / median))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gen::{generate, CwSpec};

    fn run(spec: &CwSpec, tone: f64) -> (Vec<Frame>, Frontend) {
        let g = generate(spec);
        let mut f = Frontend::new(FrontendConfig::new(g.sample_rate, tone));
        let frames = f.push(&g.samples);
        (frames, f)
    }

    #[test]
    fn frame_rate_is_right() {
        let spec = CwSpec { text: "TEST".into(), ..Default::default() };
        let g = generate(&spec);
        let (frames, _) = run(&spec, 700.0);
        let expect = g.samples.len() as f64 / 8000.0 * FRAME_RATE;
        assert!((frames.len() as f64 - expect).abs() < 3.0);
    }

    #[test]
    fn clean_signal_gives_confident_evidence() {
        let spec = CwSpec { text: "TTTT".into(), wpm: 15.0, ..Default::default() };
        let (frames, f) = run(&spec, 700.0);
        // Middle of the first dash: key down, evidence strongly positive.
        let first_mark = (0.5 * FRAME_RATE) as usize + 20;
        assert!(frames[first_mark].llr > 3.0, "{:?}", frames[first_mark]);
        // Lead-in: nothing but silence, evidence negative once levels are known.
        let late_gap = frames.len() - 100;
        assert!(frames[late_gap].llr < -3.0, "{:?}", frames[late_gap]);
        assert!(f.levels().unwrap().separation > 5.0);
    }

    #[test]
    fn pure_noise_gives_no_evidence() {
        let g = generate(&CwSpec { text: "".into(), snr_db: Some(0.0), lead_s: 6.0, trail_s: 0.0, ..Default::default() });
        let mut f = Frontend::new(FrontendConfig::new(8000, 700.0));
        f.push(&g.samples);
        // Noise only: whatever the cluster fit says, the separation is the noise-only value.
        let sep = f.levels().map(|l| l.separation).unwrap_or(0.0);
        assert!(sep < 4.0, "{sep}");
    }

    #[test]
    fn afc_follows_a_mistuned_tone() {
        let spec = CwSpec { text: "CQ CQ CQ DE ON3VZ ON3VZ K".into(), wpm: 18.0, snr_db: Some(15.0), tone_hz: 715.0, ..Default::default() };
        let (_, f) = run(&spec, 700.0);
        assert!((f.tone_hz() - 715.0).abs() < 4.0, "{}", f.tone_hz());
    }

    #[test]
    fn afc_stays_inside_its_range() {
        let spec = CwSpec { text: "CQ CQ CQ CQ CQ CQ".into(), snr_db: Some(15.0), tone_hz: 900.0, ..Default::default() };
        let (_, f) = run(&spec, 700.0);
        assert!(f.tone_hz() <= 760.0 + 1e-9);
    }

    #[test]
    fn finds_the_tone() {
        let g = generate(&CwSpec { text: "CQ CQ CQ".into(), tone_hz: 623.0, snr_db: Some(5.0), ..Default::default() });
        let (f, ratio) = find_tone(&g.samples, 8000, 300.0, 1000.0).unwrap();
        assert!((f - 623.0).abs() <= 5.0, "{f}");
        assert!(ratio > 10.0, "{ratio}");
    }
}
