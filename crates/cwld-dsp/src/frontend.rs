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

use crate::clean::{AutoNotch, NoiseBlanker};
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
    /// Silence short clicks before anything else (see `clean::NoiseBlanker`).
    pub noise_blanker: bool,
    /// Notch out steady carriers near the tone (see `clean::AutoNotch`).
    pub auto_notch: bool,
    /// Use the exact envelope statistics of a tone in noise (Rice against Rayleigh) with a
    /// signal level that follows fading. Off: the older Gaussian two-level model.
    pub rice: bool,
    /// Follow the key-down level quickly, so fading (QSB) does not look like key-up.
    pub track_fading: bool,
    /// Key-down evidence for marks weaker than the followed level (a fade inside a word):
    /// how many noise spreads above the noise a mark must stand to count (0 = off, the old
    /// behaviour: only marks near the followed level count), and what every halving of the
    /// level costs in nats.
    pub weak_floor_sd: f64,
    pub weak_cost: f64,
    /// The deepest fade taken along, as a ratio of the followed level (4 = 12 dB).
    pub weak_depth: f64,
    /// A weak mark only counts within this many Hz of the tone.
    pub weak_pitch_hz: f64,
    /// A weak mark holds its level: nothing more than this many times stronger within half a unit.
    pub weak_plateau: f64,
    /// Step of that level follower per key-down frame (0.03 is about 150 ms of key-down).
    pub fade_rate: f64,
    /// Shape of the low-pass filter: 0 is the soft filter (three one-pole sections, the
    /// standard), an even number 2 to 8 a Butterworth filter of that order with the same
    /// -3 dB point and a much steeper slope beyond it (see `STEEP_ORDER`).
    pub steep_order: usize,
}

/// Order of the steep filter the app offers ("Steep" next to "Narrow" and "Wide"). Chosen on
/// the test bench, see docs/meetresultaten_decoder.md (0.20.0).
pub const STEEP_ORDER: usize = 6;

impl FrontendConfig {
    pub fn new(sample_rate: u32, tone_hz: f64) -> Self {
        FrontendConfig { sample_rate, tone_hz, bandwidth_hz: 40.0, afc: true, afc_range_hz: 60.0, noise_blanker: true, auto_notch: true, rice: false, track_fading: true, weak_floor_sd: 4.0, weak_cost: 0.5, weak_depth: 4.0, weak_pitch_hz: 15.0, weak_plateau: 2.0, fade_rate: 0.05, steep_order: 0 }
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
    /// State of the steep filter: per section the two integrator states of I and of Q.
    svf: [[f64; 4]; 4],
}

/// Coefficients of one low-pass section of the steep filter (a state-variable filter in the
/// trapezoidal form, which stays well behaved when its cutoff moves with the speed).
#[derive(Clone, Copy, Debug, Default)]
struct Svf {
    a1: f64,
    a2: f64,
    a3: f64,
}

impl Svf {
    fn new(fc: f64, q: f64, fs: f64) -> Self {
        let g = (PI * fc / fs).tan();
        let k = 1.0 / q;
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        Svf { a1, a2, a3: g * a2 }
    }

    /// One sample through the section; `st` holds (ic1, ic2).
    #[inline]
    fn run(&self, x: f64, st: &mut [f64]) -> f64 {
        let v3 = x - st[1];
        let v1 = self.a1 * st[0] + self.a2 * v3;
        let v2 = st[1] + self.a2 * st[0] + self.a3 * v3;
        st[0] = 2.0 * v1 - st[0];
        st[1] = 2.0 * v2 - st[1];
        v2
    }
}

/// Section Q values of a Butterworth low-pass of even order `n`.
fn butterworth_q(n: usize) -> Vec<f64> {
    (0..n / 2).map(|k| 1.0 / (2.0 * ((2 * k + 1) as f64 * PI / (2 * n) as f64).sin())).collect()
}

impl Chain {
    fn run(&mut self, x: f32, f: f64, fs: f64, alpha: f64, steep: &[Svf]) -> (f64, f64) {
        self.phase += 2.0 * PI * f / fs;
        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
        let (s, c) = self.phase.sin_cos();
        let mut i = x as f64 * c;
        let mut q = -(x as f64) * s;
        if !steep.is_empty() {
            for (sec, st) in steep.iter().zip(self.svf.iter_mut()) {
                let (a, b) = st.split_at_mut(2);
                i = sec.run(i, a);
                q = sec.run(q, b);
            }
            return (i, q);
        }
        for sec in self.lp.iter_mut() {
            sec[0] += alpha * (i - sec[0]);
            sec[1] += alpha * (q - sec[1]);
            i = sec[0];
            q = sec[1];
        }
        (i, q)
    }

    /// Clears the filter state when the shape changes: the old state means nothing to the new shape.
    fn switch_shape(&mut self) {
        self.lp = [[0.0; 2]; 3];
        self.svf = [[0.0; 4]; 4];
    }
}

pub struct Frontend {
    cfg: FrontendConfig,
    blanker: NoiseBlanker,
    notch: AutoNotch,
    main: Chain,
    refs: [Chain; 2],
    /// Slow averages of the power in the wanted band and in two reference bands beside it.
    pow_main: f64,
    pow_ref: [f64; 2],
    present: bool,
    notch_changed: bool,
    /// Tracked squared amplitude of the keyed tone (0 = not known yet).
    amp2: f64,
    /// Tracked key-down envelope level for the Gaussian model (0 = not known yet).
    sig_track: f64,
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
    pitch_acc: (f64, f64),
    weak_on: bool,
    /// The last frames, held back half a unit (see `frame`): envelope, evidence without and with weak marks.
    tail: std::collections::VecDeque<(f32, f32, f32)>,
    afc_frames: usize,
    /// Sections of the steep filter, empty for the soft filter.
    steep: Vec<Svf>,
}

impl Frontend {
    pub fn new(cfg: FrontendConfig) -> Self {
        let mut f = Frontend {
            f_now: cfg.tone_hz,
            alpha: 0.0,
            blanker: NoiseBlanker::new(cfg.sample_rate),
            notch: AutoNotch::new(cfg.sample_rate),
            cfg,
            main: Chain::default(),
            refs: [Chain::default(), Chain::default()],
            pow_main: 0.0,
            pow_ref: [0.0; 2],
            present: false,
            notch_changed: false,
            amp2: 0.0,
            sig_track: 0.0,
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
            pitch_acc: (0.0, 0.0),
            weak_on: true,
            tail: std::collections::VecDeque::new(),
            afc_frames: 0,
            steep: Vec::new(),
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
        // The steep filter: -3 dB at the bandwidth, like the soft one (whose three sections
        // together are 3 dB down at 0.97 times the bandwidth).
        let order = self.cfg.steep_order.min(8) & !1;
        let fs = self.cfg.sample_rate as f64;
        self.steep = butterworth_q(order).into_iter().map(|q| Svf::new(self.cfg.bandwidth_hz, q, fs)).collect();
    }

    /// Switches between the soft filter (0) and a steep one of this even order (2 to 8).
    pub fn set_steep(&mut self, order: usize) {
        let order = order.min(8) & !1;
        if order == self.cfg.steep_order {
            return;
        }
        self.cfg.steep_order = order;
        self.main.switch_shape();
        for r in self.refs.iter_mut() {
            r.switch_shape();
        }
        self.set_bandwidth(self.cfg.bandwidth_hz);
    }

    pub fn steep_order(&self) -> usize {
        self.cfg.steep_order
    }

    /// AFC on or off.
    /// Weak marks in fading on or off (see `FrontendConfig::weak_floor_sd`). Off is the
    /// behaviour before 0.16.0: only marks near the followed level count as key-down.
    pub fn set_weak_marks(&mut self, on: bool) {
        self.weak_on = on;
        if !on {
            self.tail.clear();
        }
    }

    pub fn set_afc(&mut self, on: bool) {
        self.cfg.afc = on;
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
        self.tail.clear();
        self.pitch_acc = (0.0, 0.0);
        // The presence test starts again for the new pitch.
        self.frames_seen = 0;
        self.present = false;
    }

    /// Drops the frames still held back (see `frame`). For when the audio that follows is not
    /// the continuation of what came before (the same seconds are heard again).
    pub fn drop_held(&mut self) {
        self.tail.clear();
        self.pitch_acc = (0.0, 0.0);
        self.prev_z = None;
    }

    /// True once after a notch was added or removed: evidence from before is suspect.
    pub fn take_notch_changed(&mut self) -> bool {
        std::mem::take(&mut self.notch_changed)
    }

    /// Frequencies of the carriers that are notched out now.
    pub fn notches(&self) -> Vec<f64> {
        self.notch.active_notches()
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
        for &raw in samples {
            let x = if self.cfg.noise_blanker {
                match self.blanker.process(raw) {
                    Some(v) => v,
                    None => continue,
                }
            } else {
                raw
            };
            let x = if self.cfg.auto_notch { self.notch.process(x, self.f_now) } else { x };
            if self.cfg.auto_notch && self.notch.take_changed() {
                // The envelope statistics so far include the carrier: start them again.
                self.reset_stats();
                self.sig_track = 0.0;
                self.amp2 = 0.0;
                self.notch_changed = true;
            }
            let (i, q) = self.main.run(x, self.f_now, fs, self.alpha, &self.steep);
            let (ia, qa) = self.refs[0].run(x, self.f_now - ref_off, fs, self.alpha, &self.steep);
            let (ib, qb) = self.refs[1].run(x, self.f_now + ref_off, fs, self.alpha, &self.steep);
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
                if self.sample_count < (WARMUP_SECONDS * self.cfg.sample_rate as f64) as u64 || self.frames_seen < 40 {
                    self.present = false;
                } else if self.present && r < 1.15 {
                    self.present = false;
                } else if !self.present && r > 1.3 {
                    self.present = true;
                }
                out.extend(self.frame(i, q));
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

    fn frame(&mut self, i: f64, q: f64) -> Option<Frame> {
        let env = (i * i + q * q).sqrt() as f32;
        self.hist[self.hist_pos] = env;
        self.hist_pos = (self.hist_pos + 1) % STATS_FRAMES;
        self.hist_len = (self.hist_len + 1).min(STATS_FRAMES);
        self.since_stats += 1;
        if self.hist_len >= MIN_STATS_FRAMES && (self.since_stats >= STATS_EVERY || !self.have_levels) {
            self.since_stats = 0;
            self.update_levels();
        }
        // The pitch of this frame against the tuned tone, from the phase step over the last
        // frames: a mark of this station sits on the tone, one of a station next door does not.
        let on_pitch = match self.prev_z {
            Some((pi, pq)) => {
                let z = (i * pi + q * pq, q * pi - i * pq);
                self.pitch_acc = (0.5 * self.pitch_acc.0 + z.0, 0.5 * self.pitch_acc.1 + z.1);
                (self.pitch_acc.1.atan2(self.pitch_acc.0) * FRAME_RATE / (2.0 * PI)).abs() <= self.cfg.weak_pitch_hz
            }
            None => false,
        };
        let llr = if self.have_levels { self.llr_at(env, false) } else { 0.0 };
        // Not in the first moments after tuning: the filter settling looks like a weak dot.
        let llr_weak = if self.have_levels && on_pitch && self.weak_on && self.cfg.weak_floor_sd > 0.0 && self.frames_seen > 80 { self.llr_at(env, true) } else { llr };
        if !self.cfg.rice && self.have_levels && self.cfg.track_fading {
            // Follow the key-down level through fading. Only frames that are surely key-down
            // count, the floor keeps it above the noise.
            let l = self.levels;
            if self.sig_track <= 0.0 {
                self.sig_track = l.signal as f64;
            } else if env as f64 > l.noise as f64 + 2.0 * l.noise_sd as f64 {
                // Gate on "clearly above the noise", not on "looks like the old level":
                // otherwise a fading signal drops out of the gate and the level never follows.
                self.sig_track += self.cfg.fade_rate * (env as f64 - self.sig_track);
            }
            let floor = l.noise as f64 + 2.0 * l.noise_sd as f64;
            // Never far above the long-term level either: a strong click must not drag it up.
            self.sig_track = self.sig_track.clamp(floor, 2.0 * l.signal as f64);
        }
        if self.cfg.rice && self.have_levels {
            // Follow the signal level on frames that are surely key-down, quickly enough to
            // ride through fading (about 150 ms of key-down time).
            let s2 = self.noise_sigma2();
            if self.amp2 <= 0.0 {
                let hi = self.levels.signal as f64;
                self.amp2 = (hi * hi - 2.0 * s2).max(s2);
            } else if llr > 2.0 {
                let est = (env as f64).powi(2) - 2.0 * s2;
                self.amp2 += 0.033 * (est - self.amp2);
                self.amp2 = self.amp2.max(s2);
            }
        }

        // The AFC waits until the notch detector has had its first look: otherwise a steady
        // carrier inside the passband pulls the tone away before it is notched out.
        let notch_ready = !self.cfg.auto_notch || self.notch.ready();
        if self.cfg.afc && notch_ready && self.have_levels && self.levels.separation > 3.0 {
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
        if self.cfg.weak_floor_sd <= 0.0 || !self.weak_on {
            return Some(Frame { env, llr });
        }
        // A weak mark is a level that holds, not the slope of a strong mark: between two full
        // dots the filter never falls back to the noise, and those in-between frames must stay
        // key-up or short gaps close. So every frame waits half a unit and counts as a weak
        // mark only when nothing clearly stronger stands within half a unit on either side.
        let half = ((120.0 / self.cfg.bandwidth_hz).round() as usize).clamp(1, 12);
        self.tail.push_back((env, llr, llr_weak));
        while self.tail.len() > 2 * half + 1 {
            self.tail.pop_front();
        }
        if self.tail.len() < 2 * half + 1 {
            return None;
        }
        let (e, strong, weak) = self.tail[half];
        let top = self.tail.iter().map(|t| t.0).fold(0.0f32, f32::max);
        Some(Frame { env: e, llr: if top <= self.cfg.weak_plateau as f32 * e { weak } else { strong } })
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

    /// Noise power per I/Q component in the filter, from the reference bands beside the tone
    /// (pure noise, the keyed tone never reaches them): E[e^2] = 2 sigma^2.
    fn noise_sigma2(&self) -> f64 {
        let r = self.pow_ref[0].min(self.pow_ref[1]);
        if r > 0.0 && self.frames_seen > 100 {
            r / 2.0
        } else {
            // Before the references settle: from the noise cluster (Rayleigh mean sigma*sqrt(pi/2)).
            let m = self.levels.noise as f64 / (PI / 2.0).sqrt();
            m * m
        }
    }

    fn llr(&self, env: f32) -> f32 {
        // Without the pitch of the frame a weak mark is not taken along.
        self.llr_at(env, false)
    }

    fn llr_at(&self, env: f32, on_pitch: bool) -> f32 {
        if self.cfg.rice && self.amp2 > 0.0 {
            if self.levels.separation < 1.5 {
                return 0.0;
            }
            let s2 = self.noise_sigma2().max(1e-24);
            let a = self.amp2.sqrt();
            let e = env as f64;
            // ln Rice(e; A, s) - ln Rayleigh(e; s) = -A^2 / 2s^2 + ln I0(e A / s^2)
            let v = -self.amp2 / (2.0 * s2) + ln_i0(e * a / s2);
            return v.clamp(-12.0, 12.0) as f32;
        }
        let l = &self.levels;
        if l.separation < 1.5 {
            return 0.0;
        }
        let e = env as f64;
        let n = l.noise as f64;
        let (s, ss) = if self.cfg.track_fading && self.sig_track > 0.0 {
            // Scale the spread with the level, but never below the noise spread.
            let k = self.sig_track / (l.signal as f64).max(1e-12);
            (self.sig_track, ((l.signal_sd as f64) * k).max(l.noise_sd as f64 * 0.7))
        } else {
            (l.signal as f64, l.signal_sd as f64)
        };
        let (sn, ss) = ((l.noise_sd as f64).max(1e-9), ss.max(1e-9));
        let mut on = -0.5 * ((e - s) / ss).powi(2) - ss.ln();
        if self.cfg.weak_floor_sd > 0.0 && on_pitch && e < s {
            // In fast fading the level of one dot says little about the next: a mark may be
            // anywhere between the followed level and just above the noise. Such a mark is
            // key-down too, at a price that grows with how far it has dropped. Without this a
            // dot at a third of the level counted as silence, and the character lost it.
            // The noise is measured beside the tone, where the keyed signal never comes: the
            // lower cluster of the envelope holds the faded marks themselves in deep fading
            // and would put the floor too high.
            let sigma = self.noise_sigma2().sqrt();
            let (rn, rsd) = (sigma * (PI / 2.0).sqrt(), sigma * 0.655);
            // Not deeper than `weak_depth` below the level either: what is that much weaker is
            // more likely another station close by than this one in a fade.
            let floor = (rn + self.cfg.weak_floor_sd * rsd).max(s / self.cfg.weak_depth).min(s);
            let at = e.max(floor);
            let spread = (ss * at / s).max(rsd);
            let mut weak = -spread.ln() - self.cfg.weak_cost * (s / at).ln() / std::f64::consts::LN_2;
            if e < floor {
                weak -= 0.5 * ((e - floor) / rsd).powi(2);
            }
            on = on.max(weak);
        }
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

/// Natural log of the modified Bessel function I0, accurate to about 1e-7 relative.
/// Polynomial approximations from Abramowitz and Stegun, 9.8.1 and 9.8.2.
pub fn ln_i0(x: f64) -> f64 {
    let ax = x.abs();
    if ax < 3.75 {
        let t = (ax / 3.75).powi(2);
        (1.0 + t * (3.5156229 + t * (3.0899424 + t * (1.2067492 + t * (0.2659732 + t * (0.0360768 + t * 0.0045813)))))).ln()
    } else {
        let t = 3.75 / ax;
        let p = 0.39894228 + t * (0.01328592 + t * (0.00225319 + t * (-0.00157565 + t * (0.00916281 + t * (-0.02057706 + t * (0.02635537 + t * (-0.01647633 + t * 0.00392377)))))));
        ax - 0.5 * ax.ln() + p.ln()
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

    /// Envelope level of a steady tone `off_hz` away from the tuned pitch, after the filter settled.
    fn response(off_hz: f64, steep: usize) -> f64 {
        let mut cfg = FrontendConfig::new(8000, 700.0);
        cfg.afc = false;
        cfg.noise_blanker = false;
        cfg.auto_notch = false;
        cfg.steep_order = steep;
        cfg.bandwidth_hz = 20.0;
        let mut f = Frontend::new(cfg);
        let x: Vec<f32> = (0..16000).map(|i| (2.0 * PI * (700.0 + off_hz) * i as f64 / 8000.0).sin() as f32).collect();
        let fr = f.push(&x);
        let tail = &fr[fr.len() - 100..];
        tail.iter().map(|f| f.env as f64).sum::<f64>() / tail.len() as f64
    }

    #[test]
    fn steep_filter_has_the_same_passband_and_much_less_beside_it() {
        for order in [0, STEEP_ORDER] {
            let pass = response(0.0, order);
            let edge = 20.0 * (response(20.0, order) / pass).log10();
            assert!((-4.5..=-2.0).contains(&edge), "order {order}: {edge:.1} dB at the bandwidth");
        }
        let db = |off: f64, o: usize| 20.0 * (response(off, o) / response(0.0, o)).log10();
        if std::env::var("CWLD_SHOW").is_ok() {
            for o in [0, 4, 6, 8] {
                println!("order {o}: {}", [20.0, 40.0, 60.0, 80.0, 120.0, 200.0].map(|f| format!("{f} Hz {:.1} dB", db(f, o))).join(", "));
            }
        }
        // 60 and 120 Hz away: a neighbour station.
        assert!(db(60.0, STEEP_ORDER) < db(60.0, 0) - 10.0, "{} vs {}", db(60.0, STEEP_ORDER), db(60.0, 0));
        assert!(db(120.0, STEEP_ORDER) < -55.0, "{}", db(120.0, STEEP_ORDER));
    }

    #[test]
    fn frame_rate_is_right() {
        let spec = CwSpec { text: "TEST".into(), ..Default::default() };
        let g = generate(&spec);
        let (frames, _) = run(&spec, 700.0);
        let expect = g.samples.len() as f64 / 8000.0 * FRAME_RATE;
        // Frames are held back half a unit (6 frames at the default bandwidth) to judge weak marks.
        assert!((frames.len() as f64 - (expect - 6.0)).abs() < 3.0, "{} frames, {expect} expected", frames.len());
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
    fn ln_i0_matches_known_values() {
        // I0(0) = 1, I0(1) = 1.2660659, I0(5) = 27.239872, I0(20) = 4.355828e7
        assert!(ln_i0(0.0).abs() < 1e-9);
        assert!((ln_i0(1.0) - 1.2660659f64.ln()).abs() < 1e-6);
        assert!((ln_i0(5.0) - 27.239872f64.ln()).abs() < 1e-5);
        assert!((ln_i0(20.0) - 4.355828e7f64.ln()).abs() < 1e-5);
    }

    #[test]
    fn finds_the_tone() {
        let g = generate(&CwSpec { text: "CQ CQ CQ".into(), tone_hz: 623.0, snr_db: Some(5.0), ..Default::default() });
        let (f, ratio) = find_tone(&g.samples, 8000, 300.0, 1000.0).unwrap();
        assert!((f - 623.0).abs() <= 5.0, "{f}");
        assert!(ratio > 10.0, "{ratio}");
    }
}
