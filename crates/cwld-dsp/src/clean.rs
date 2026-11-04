//! Removing interference before the decoder: a noise blanker against short clicks and
//! automatic notches against steady carriers.
//!
//! Both are classic signal processing, written for CW: a keyed tone is never shorter than a
//! dot (24 ms at 50 WPM) and never steady for long, so short bursts and steady tones can be
//! told apart from it.

use std::collections::VecDeque;
use std::f64::consts::PI;

/// Blanks short, strong bursts (ignition, switching supplies, lightning crashes).
///
/// The input is delayed by a few milliseconds. A burst is a stretch where the short-term peak
/// level is far above the usual level. If it ends within `max_ms`, it is replaced by silence.
/// Anything longer is a signal (a keyed tone), and is left alone.
pub struct NoiseBlanker {
    delay: VecDeque<f32>,
    delay_len: usize,
    window: usize,
    peaks: VecDeque<f32>,
    level: f64,
    level_alpha: f64,
    ratio: f64,
    max_len: usize,
    /// Start of the current excursion, counted in input samples.
    excursion: Option<u64>,
    long_excursion: bool,
    /// Ranges (in input sample numbers) to silence.
    blank: VecDeque<(u64, u64)>,
    n: u64,
    pub blanked: u64,
}

impl NoiseBlanker {
    pub fn new(sample_rate: u32) -> Self {
        let fs = sample_rate as f64;
        let window = (0.002 * fs) as usize + 1;
        let max_len = (0.006 * fs) as usize;
        NoiseBlanker {
            delay: VecDeque::new(),
            delay_len: max_len + 2 * window + 4,
            window,
            peaks: VecDeque::new(),
            level: 0.0,
            level_alpha: 1.0 / (0.1 * fs),
            ratio: 5.0,
            max_len,
            excursion: None,
            long_excursion: false,
            blank: VecDeque::new(),
            n: 0,
            blanked: 0,
        }
    }

    /// Processes one sample and returns the delayed, cleaned sample (or `None` while the delay
    /// line fills).
    pub fn process(&mut self, x: f32) -> Option<f32> {
        let idx = self.n;
        self.n += 1;
        self.peaks.push_back(x.abs());
        if self.peaks.len() > self.window {
            self.peaks.pop_front();
        }
        let peak = self.peaks.iter().cloned().fold(0.0f32, f32::max) as f64;
        if self.level == 0.0 {
            self.level = peak.max(1e-9);
        }
        let thr = self.level * self.ratio;
        if peak > thr {
            if self.excursion.is_none() {
                self.excursion = Some(idx);
                self.long_excursion = false;
            }
            let start = self.excursion.unwrap();
            if idx - start > self.max_len as u64 {
                // Too long for a click: this is a signal. Let the level follow it quickly.
                self.long_excursion = true;
                self.level += 0.02 * (peak - self.level);
            }
        } else {
            if let Some(start) = self.excursion.take() {
                if !self.long_excursion {
                    let from = start.saturating_sub(self.window as u64);
                    self.blank.push_back((from, idx));
                }
            }
            self.level += self.level_alpha * (peak - self.level);
        }

        self.delay.push_back(x);
        if self.delay.len() <= self.delay_len {
            return None;
        }
        let y = self.delay.pop_front().unwrap();
        let out_idx = idx - self.delay_len as u64;
        while let Some(&(_, b)) = self.blank.front() {
            if b < out_idx {
                self.blank.pop_front();
            } else {
                break;
            }
        }
        if let Some(&(a, b)) = self.blank.front() {
            if out_idx >= a && out_idx <= b {
                self.blanked += 1;
                return Some(0.0);
            }
        }
        Some(y)
    }

    pub fn delay_samples(&self) -> usize {
        self.delay_len
    }
}

/// A second order notch filter (biquad).
#[derive(Clone)]
struct Notch {
    hz: f64,
    b: [f64; 3],
    a: [f64; 2],
    x: [f64; 2],
    y: [f64; 2],
}

impl Notch {
    fn new(hz: f64, fs: f64, width_hz: f64) -> Self {
        let w = 2.0 * PI * hz / fs;
        let r = 1.0 - PI * width_hz / fs;
        let c = w.cos();
        // Zeros on the unit circle at the carrier, poles just inside: unity gain elsewhere.
        let b = [1.0, -2.0 * c, 1.0];
        let a = [-2.0 * r * c, r * r];
        // Normalise the gain far from the notch (at 0 Hz and Nyquist the average is close to 1).
        let g0 = (1.0 + a[0] + a[1]) / (b[0] + b[1] + b[2]);
        Notch { hz, b: [b[0] * g0, b[1] * g0, b[2] * g0], a, x: [0.0; 2], y: [0.0; 2] }
    }

    fn run(&mut self, x: f64) -> f64 {
        let y = self.b[0] * x + self.b[1] * self.x[0] + self.b[2] * self.x[1] - self.a[0] * self.y[0] - self.a[1] * self.y[1];
        self.x = [x, self.x[0]];
        self.y = [y, self.y[0]];
        y
    }
}

/// Finds steady carriers near the wanted tone and notches them out.
///
/// Every quarter second the power is measured in 4 Hz steps around the tone (Hann window,
/// Goertzel). A carrier is a strong peak whose power hardly changes between blocks: a keyed
/// signal has key-up moments, so its weakest block is far below its average. The wanted tone
/// itself (within `guard_hz`) is never notched.
pub struct AutoNotch {
    fs: f64,
    block: Vec<f32>,
    block_len: usize,
    window: Vec<f64>,
    span_hz: f64,
    step_hz: f64,
    guard_hz: f64,
    history: VecDeque<Vec<f64>>,
    notches: Vec<Notch>,
    pub max_notches: usize,
    changed: bool,
    history_lo: Option<f64>,
}

const HISTORY_BLOCKS: usize = 6;

impl AutoNotch {
    pub fn new(sample_rate: u32) -> Self {
        let fs = sample_rate as f64;
        let block_len = (0.25 * fs) as usize;
        let window = (0..block_len).map(|i| 0.5 - 0.5 * (2.0 * PI * i as f64 / (block_len - 1) as f64).cos()).collect();
        AutoNotch { fs, block: Vec::with_capacity(block_len), block_len, window, span_hz: 400.0, step_hz: 4.0, guard_hz: 18.0, history: VecDeque::new(), notches: Vec::new(), max_notches: 3, changed: false, history_lo: None }
    }

    /// True once the detector has seen enough blocks to judge what is steady.
    pub fn ready(&self) -> bool {
        self.history.len() >= HISTORY_BLOCKS
    }

    pub fn active_notches(&self) -> Vec<f64> {
        self.notches.iter().map(|n| n.hz).collect()
    }

    /// Filters one sample. `x_raw` feeds the detector, which must see the input before the
    /// notches, otherwise a notched carrier would look gone and the notch would be removed.
    pub fn process(&mut self, x: f32, tone_hz: f64) -> f32 {
        self.block.push(x);
        if self.block.len() >= self.block_len {
            self.analyse(tone_hz);
            self.block.clear();
        }
        let mut y = x as f64;
        for n in self.notches.iter_mut() {
            y = n.run(y);
        }
        y as f32
    }

    fn analyse(&mut self, tone_hz: f64) {
        let lo = (tone_hz - self.span_hz).max(150.0);
        let hi = (tone_hz + self.span_hz).min(self.fs / 2.0 - 100.0);
        let mut powers = Vec::new();
        let nb = ((hi - lo) / self.step_hz).floor() as usize + 1;
        for i in 0..nb {
            let f = lo + i as f64 * self.step_hz;
            let w = 2.0 * PI * f / self.fs;
            let coeff = 2.0 * w.cos();
            let (mut s1, mut s2) = (0.0f64, 0.0f64);
            for (x, win) in self.block.iter().zip(&self.window) {
                let s0 = *x as f64 * win + coeff * s1 - s2;
                s2 = s1;
                s1 = s0;
            }
            powers.push(s1 * s1 + s2 * s2 - coeff * s1 * s2);
        }
        // A retune moves the analysed band: the old blocks no longer line up with the new ones.
        let same_shape = self.history.back().map_or(true, |h| h.len() == nb);
        if !same_shape || self.history_lo.map_or(false, |l| (l - lo).abs() > 1e-9) {
            self.history.clear();
        }
        self.history_lo = Some(lo);
        self.history.push_back(powers);
        if self.history.len() > HISTORY_BLOCKS {
            self.history.pop_front();
        }
        if self.history.len() < HISTORY_BLOCKS {
            return;
        }
        let nb = self.history[0].len();
        let mean: Vec<f64> = (0..nb).map(|i| self.history.iter().map(|h| h[i]).sum::<f64>() / HISTORY_BLOCKS as f64).collect();
        let min: Vec<f64> = (0..nb).map(|i| self.history.iter().map(|h| h[i]).fold(f64::MAX, f64::min)).collect();
        let max: Vec<f64> = (0..nb).map(|i| self.history.iter().map(|h| h[i]).fold(0.0, f64::max)).collect();
        let mut sorted = mean.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let floor = sorted[nb / 2].max(1e-30);
        let mut found: Vec<(f64, f64)> = Vec::new();
        for i in 1..nb.saturating_sub(1) {
            let freq = lo + i as f64 * self.step_hz;
            if (freq - tone_hz).abs() < self.guard_hz {
                continue;
            }
            let is_peak = mean[i] >= mean[i - 1] && mean[i] >= mean[i + 1];
            let strong = mean[i] > 30.0 * floor;
            // A carrier hardly moves between blocks. A keyed station, even a fast one that
            // keys something in every block, swings much more.
            let steady = min[i] > 0.75 * mean[i] && max[i] < 1.33 * mean[i];
            if is_peak && strong && steady {
                // Parabolic interpolation of the peak position (on log power).
                let (a, b, c) = (mean[i - 1].ln(), mean[i].ln(), mean[i + 1].ln());
                let den = a - 2.0 * b + c;
                let delta = if den.abs() > 1e-12 { 0.5 * (a - c) / den } else { 0.0 };
                found.push((freq + delta.clamp(-0.5, 0.5) * self.step_hz, mean[i]));
            }
        }
        found.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        found.truncate(self.max_notches);
        let before: Vec<f64> = self.notches.iter().map(|n| n.hz).collect();
        // Keep notches that still match a carrier (their filter state stays), add new ones.
        let mut next: Vec<Notch> = Vec::new();
        for (hz, _) in &found {
            if let Some(pos) = self.notches.iter().position(|n| (n.hz - hz).abs() < 3.0) {
                next.push(self.notches.remove(pos));
            } else {
                next.push(Notch::new(*hz, self.fs, 6.0));
            }
        }
        if next.len() != before.len() || next.iter().any(|n| !before.iter().any(|b| (b - n.hz).abs() < 0.01)) {
            self.changed = true;
        }
        self.notches = next;
    }

    /// True once after the set of notches changed (a carrier came or went).
    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gen::{generate, CwSpec};

    fn power_near(samples: &[f32], fs: f64, hz: f64) -> f64 {
        let w = 2.0 * PI * hz / fs;
        let coeff = 2.0 * w.cos();
        let (mut s1, mut s2) = (0.0f64, 0.0f64);
        for x in samples {
            let s0 = *x as f64 + coeff * s1 - s2;
            s2 = s1;
            s1 = s0;
        }
        (s1 * s1 + s2 * s2 - coeff * s1 * s2) / samples.len() as f64
    }

    #[test]
    fn blanker_removes_clicks_but_keeps_the_tone() {
        let spec = CwSpec { text: "TTT".into(), wpm: 15.0, snr_db: Some(20.0), impulses_per_s: 20.0, impulse_db: 30.0, ..Default::default() };
        let g = generate(&spec);
        let clean = generate(&CwSpec { impulses_per_s: 0.0, ..spec.clone() });
        let mut nb = NoiseBlanker::new(8000);
        let out: Vec<f32> = g.samples.iter().filter_map(|x| nb.process(*x)).collect();
        let peak_in = g.samples.iter().fold(0.0f32, |a, x| a.max(x.abs()));
        let peak_out = out.iter().fold(0.0f32, |a, x| a.max(x.abs()));
        let peak_clean = clean.samples.iter().fold(0.0f32, |a, x| a.max(x.abs()));
        assert!(peak_in > 5.0 * peak_clean, "test needs strong clicks");
        assert!(peak_out < 1.5 * peak_clean, "{peak_out} vs {peak_clean}");
        // The keyed tone survives: its power at 700 Hz is about the same.
        let p_out = power_near(&out, 8000.0, 700.0);
        let p_clean = power_near(&clean.samples, 8000.0, 700.0);
        assert!(p_out > 0.8 * p_clean, "{p_out} vs {p_clean}");
        // And only a small part of the time is blanked.
        assert!((nb.blanked as f64) < 0.1 * g.samples.len() as f64);
    }

    #[test]
    fn blanker_leaves_a_strong_clean_signal_alone() {
        let g = generate(&CwSpec { text: "CQ CQ".into(), ..Default::default() });
        let mut nb = NoiseBlanker::new(8000);
        let out: Vec<f32> = g.samples.iter().filter_map(|x| nb.process(*x)).collect();
        let d = nb.delay_samples();
        let diff: f32 = out.iter().zip(&g.samples).map(|(a, b)| (a - b).abs()).sum();
        assert_eq!(out.len(), g.samples.len() - d);
        assert!(diff < 1e-3, "{diff}");
    }

    #[test]
    fn notch_finds_and_removes_a_carrier_but_not_the_keyed_tone() {
        let spec = CwSpec { text: "CQ CQ CQ DE ON3VZ ON3VZ K".into(), snr_db: Some(20.0), carrier: Some((50.0, 20.0)), ..Default::default() };
        let g = generate(&spec);
        let mut an = AutoNotch::new(8000);
        let out: Vec<f32> = g.samples.iter().map(|x| an.process(*x, 700.0)).collect();
        let notches = an.active_notches();
        assert_eq!(notches.len(), 1, "{notches:?}");
        assert!((notches[0] - 750.0).abs() < 2.0, "{notches:?}");
        let tail = out.len() / 2;
        let before = power_near(&g.samples[tail..], 8000.0, 750.0);
        let after = power_near(&out[tail..], 8000.0, 750.0);
        assert!(after < before / 100.0, "{after} vs {before}");
        let tone_before = power_near(&g.samples[tail..], 8000.0, 700.0);
        let tone_after = power_near(&out[tail..], 8000.0, 700.0);
        assert!(tone_after > 0.7 * tone_before);
    }

    #[test]
    fn retuning_while_analysing_is_safe() {
        let g = generate(&CwSpec { text: "CQ CQ CQ CQ".into(), snr_db: Some(20.0), carrier: Some((50.0, 20.0)), ..Default::default() });
        let mut an = AutoNotch::new(8000);
        for (i, x) in g.samples.iter().enumerate() {
            let tone = if i < g.samples.len() / 2 { 700.0 } else { 553.0 };
            an.process(*x, tone);
        }
    }

    #[test]
    fn notch_ignores_a_keyed_second_station() {
        let spec = CwSpec {
            text: "CQ CQ CQ".into(),
            snr_db: Some(20.0),
            interferer: Some(crate::gen::Interferer { offset_hz: 80.0, rel_db: 10.0, wpm: 20.0 }),
            lead_s: 0.5,
            trail_s: 3.0,
            ..Default::default()
        };
        let g = generate(&spec);
        let mut an = AutoNotch::new(8000);
        for x in &g.samples {
            an.process(*x, 700.0);
        }
        assert!(an.active_notches().is_empty(), "{:?}", an.active_notches());
    }
}
