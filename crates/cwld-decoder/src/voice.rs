//! Is this speech (or another wide signal) rather than CW? A check apart from the envelope gate.
//!
//! A CW signal is one narrow tone. When it is keyed, only its own pitch gets louder. Speech is
//! many harmonics of a moving pitch: when a syllable starts, the pitch of the decoder gets
//! louder together with large parts of the passband around it. So the detector compares two
//! levels every 32 ms: the power right at the decoder pitch, and the power in the rest of the
//! passband (300 to 2700 Hz, without 200 Hz on both sides of the pitch). Over the last four
//! seconds it computes the correlation of the two (in dB). Keyed CW in noise gives about 0, or
//! a negative value when the AGC of the radio pumps. Speech gives a clearly positive value.
//!
//! Costs: 28 Goertzel bins over 256 samples, 31 times a second.

use std::collections::VecDeque;
use std::f64::consts::PI;

const BLOCKS: usize = 125; // about 4 s
const MIN_BLOCKS: usize = 40;
/// Bins near the pitch (in Hz offset) and the number of bins in the rest of the passband.
const NEAR: [f64; 3] = [-12.0, 0.0, 12.0];
const WIDE_BINS: usize = 25;
const GUARD_HZ: f64 = 200.0;
/// A sample this many times the median magnitude of its block is a click.
const BLANK_OVER_MEDIAN: f32 = 10.0;
const BLANK_SPREAD: usize = 12;

#[derive(Clone, Copy, Debug, Default)]
pub struct VoiceStats {
    /// Correlation of pitch level and passband level, -1 to 1.
    pub corr: f32,
    /// Spread of the pitch level in dB: too little means nothing is keyed.
    pub spread_db: f32,
    /// Passband level minus pitch level while the pitch is loud (median, dB). Speech: about
    /// -15 to 0. A clean CW tone: far below, because only its own pitch is loud.
    pub wide_vs_near_db: f32,
}

impl VoiceStats {
    /// Looks like speech: the passband moves with the pitch and is about as loud.
    pub fn is_voice(&self) -> bool {
        self.corr >= VOICE_CORR && self.wide_vs_near_db >= WIDE_VS_NEAR_DB
    }
    pub fn is_clear(&self) -> bool {
        self.corr < CLEAR_CORR || self.wide_vs_near_db < WIDE_VS_NEAR_DB - 3.0
    }
}

pub struct VoiceDetector {
    fs: f64,
    n: usize,
    buf: Vec<f32>,
    window: Vec<f64>,
    hist: VecDeque<(f64, f64)>,
    pub stats: VoiceStats,
    pub evals: u64,
    since: usize,
}

fn goertzel(x: &[f64], f: f64, fs: f64) -> f64 {
    let c = 2.0 * (2.0 * PI * f / fs).cos();
    let (mut s1, mut s2) = (0.0, 0.0);
    for &v in x {
        let s = v + c * s1 - s2;
        s2 = s1;
        s1 = s;
    }
    s1 * s1 + s2 * s2 - c * s1 * s2
}

impl VoiceDetector {
    pub fn new(sample_rate: u32) -> Self {
        let fs = sample_rate as f64;
        let n = ((fs * 0.032) as usize).max(64);
        // Blackman window: low leakage, so a strong clean tone does not show up in the passband bins.
        let window = (0..n)
            .map(|i| {
                let a = 2.0 * PI * i as f64 / (n - 1) as f64;
                0.42 - 0.5 * a.cos() + 0.08 * (2.0 * a).cos()
            })
            .collect();
        VoiceDetector { fs, n, buf: Vec::with_capacity(n), window, hist: VecDeque::new(), stats: VoiceStats::default(), evals: 0, since: 0 }
    }

    pub fn push(&mut self, samples: &[f32], tone_hz: f64) {
        for &s in samples {
            self.buf.push(s);
            if self.buf.len() == self.n {
                self.block(tone_hz);
                self.buf.clear();
            }
        }
    }

    fn block(&mut self, tone: f64) {
        // Clicks (ignition, switching supplies) are short and loud in the whole passband, like
        // the start of a syllable. Samples far above the median of this block are blanked, with
        // a few around them, so a click does not count as speech.
        let mut mags: Vec<f32> = self.buf.iter().map(|s| s.abs()).collect();
        mags.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let limit = mags[mags.len() / 2] * BLANK_OVER_MEDIAN;
        let mut keep = vec![true; self.buf.len()];
        for (i, s) in self.buf.iter().enumerate() {
            if s.abs() > limit {
                let a = i.saturating_sub(BLANK_SPREAD);
                let b = (i + BLANK_SPREAD + 1).min(keep.len());
                keep[a..b].iter_mut().for_each(|k| *k = false);
            }
        }
        let x: Vec<f64> = self.buf.iter().zip(&self.window).zip(&keep).map(|((s, w), k)| if *k { *s as f64 * w } else { 0.0 }).collect();
        let near: f64 = NEAR.iter().map(|d| goertzel(&x, tone + d, self.fs)).sum::<f64>() / NEAR.len() as f64;
        let (lo, hi) = (300.0, 2700.0f64.min(self.fs * 0.45));
        let mut wide = 0.0;
        let mut count = 0;
        for k in 0..WIDE_BINS {
            let f = lo + (hi - lo) * (k as f64 + 0.5) / WIDE_BINS as f64;
            if (f - tone).abs() < GUARD_HZ {
                continue;
            }
            wide += goertzel(&x, f, self.fs);
            count += 1;
        }
        let wide = wide / count.max(1) as f64;
        self.hist.push_back((10.0 * (near + 1e-20).log10(), 10.0 * (wide + 1e-20).log10()));
        if self.hist.len() > BLOCKS {
            self.hist.pop_front();
        }
        self.since += 1;
        if self.since >= 8 && self.hist.len() >= MIN_BLOCKS {
            self.since = 0;
            self.stats = measure(&self.hist);
            self.evals += 1;
        }
    }
}

fn measure(h: &VecDeque<(f64, f64)>) -> VoiceStats {
    let n = h.len() as f64;
    let (ma, mb) = h.iter().fold((0.0, 0.0), |(a, b), (x, y)| (a + x, b + y));
    let (ma, mb) = (ma / n, mb / n);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in h {
        sab += (x - ma) * (y - mb);
        saa += (x - ma).powi(2);
        sbb += (y - mb).powi(2);
    }
    let corr = if saa > 0.0 && sbb > 0.0 { sab / (saa * sbb).sqrt() } else { 0.0 };
    // The loudest third of the blocks at the pitch: what does the passband do then?
    let mut near: Vec<f64> = h.iter().map(|p| p.0).collect();
    near.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // Never below 30 dB under the loudest block, so silence (no noise at all) does not count.
    let cut = near[(near.len() as f64 * 0.67) as usize].max(near[near.len() - 1] - 30.0);
    let mut diff: Vec<f64> = h.iter().filter(|p| p.0 >= cut).map(|p| p.1 - p.0).collect();
    diff.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let wide_vs_near_db = diff.get(diff.len() / 2).cloned().unwrap_or(-99.0);
    VoiceStats { corr: corr as f32, spread_db: (saa / n).sqrt() as f32, wide_vs_near_db: wide_vs_near_db as f32 }
}

/// Above this correlation the audio counts as speech. Measured on the test bench (see the
/// handbook of the decoder): keyed CW stays below about 0.4 (with a strong second station),
/// synthetic speech at 10 dB SNR and more has a median of 0.6 and up.
pub const VOICE_CORR: f32 = 0.45;
/// Back below this value the gate may open again.
pub const CLEAR_CORR: f32 = 0.35;
/// Measurements (0.26 s each) in a row that must look clear before decoding starts again.
pub const REOPEN_AFTER: u32 = 6;
/// Speech needs the passband within this many dB of the pitch level while the pitch is loud.
pub const WIDE_VS_NEAR_DB: f32 = -22.0;

#[cfg(test)]
mod tests {
    use super::*;
    use cwld_dsp::gen::Interferer;
    use cwld_dsp::{generate, generate_voice, CwSpec, VoiceSpec};

    /// Every measurement over a recording, in order.
    fn corrs(samples: &[f32], fs: u32, tone: f64) -> Vec<f32> {
        stats(samples, fs, tone).iter().map(|s| s.corr).collect()
    }

    fn stats(samples: &[f32], fs: u32, tone: f64) -> Vec<VoiceStats> {
        let mut d = VoiceDetector::new(fs);
        let mut out = Vec::new();
        for c in samples.chunks(200) {
            let e = d.evals;
            d.push(c, tone);
            if d.evals != e {
                out.push(d.stats);
            }
        }
        out
    }

    fn pct(v: &[f32], p: f64) -> f32 {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        s[((s.len() - 1) as f64 * p) as usize]
    }

    #[test]
    #[ignore]
    fn print_correlations() {
        for snr in [30.0, 10.0, 3.0, 0.0] {
            for shift in [0.0, 150.0, -80.0] {
                let mut all = Vec::new();
                let mut ratio = Vec::new();
                let mut voice = 0usize;
                for seed in 1..=4 {
                    let st = stats(&generate_voice(&VoiceSpec { seconds: 40.0, snr_db: Some(snr), shift_hz: shift, seed, ..Default::default() }), 8000, 700.0);
                    ratio.extend(st.iter().map(|s| s.wide_vs_near_db));
                    voice += st.iter().filter(|s| s.is_voice()).count();
                    all.extend(corrs(&generate_voice(&VoiceSpec { seconds: 40.0, snr_db: Some(snr), shift_hz: shift, seed, ..Default::default() }), 8000, 700.0));
                }
                println!("voice snr {snr:>4} shift {shift:>5}: p5 {:.2} p50 {:.2} ratio p5 {:.1} p50 {:.1} voice {:.0}%", pct(&all, 0.05), pct(&all, 0.5), pct(&ratio, 0.05), pct(&ratio, 0.5), 100.0 * voice as f64 / all.len() as f64);
            }
        }
        let cases: Vec<(&str, CwSpec)> = vec![
            ("cw +20", CwSpec { snr_db: Some(20.0), ..Default::default() }),
            ("cw +3", CwSpec { snr_db: Some(3.0), ..Default::default() }),
            ("cw -3", CwSpec { snr_db: Some(-3.0), ..Default::default() }),
            ("cw qsb", CwSpec { snr_db: Some(6.0), qsb_depth: 0.8, qsb_hz: 0.3, ..Default::default() }),
            ("cw qrm", CwSpec { snr_db: Some(6.0), interferer: Some(Interferer { offset_hz: 300.0, rel_db: 6.0, wpm: 25.0 }), ..Default::default() }),
            ("cw qrm near", CwSpec { snr_db: Some(6.0), interferer: Some(Interferer { offset_hz: 120.0, rel_db: 0.0, wpm: 18.0 }), ..Default::default() }),
            ("cw carrier", CwSpec { snr_db: Some(6.0), carrier: Some((400.0, 10.0)), ..Default::default() }),
            ("cw clicks", CwSpec { snr_db: Some(6.0), impulses_per_s: 5.0, impulse_db: 10.0, ..Default::default() }),
            ("clicks 15/s +46", CwSpec { snr_db: Some(10.0), impulses_per_s: 15.0, impulse_db: 46.0, ..Default::default() }),
            ("clicks 40/s +40", CwSpec { snr_db: Some(6.0), impulses_per_s: 40.0, impulse_db: 40.0, ..Default::default() }),
            ("cw 35wpm", CwSpec { snr_db: Some(10.0), wpm: 35.0, ..Default::default() }),
            ("cw 12wpm hand", CwSpec { snr_db: Some(10.0), wpm: 12.0, jitter: 0.15, ..Default::default() }),
            ("cw nonoise", CwSpec { snr_db: None, ..Default::default() }),
            ("cw +40 fast rise", CwSpec { snr_db: Some(40.0), rise_ms: 1.0, ..Default::default() }),
            ("noise", CwSpec { text: "".into(), snr_db: Some(0.0), lead_s: 40.0, ..Default::default() }),
        ];
        for (name, base) in cases {
            let mut all = Vec::new();
            let mut ratio = Vec::new();
            let mut voice = 0usize;
            for seed in 1..=4 {
                let g = generate(&CwSpec { text: if base.text.is_empty() { String::new() } else { "CQ CQ DE DL1ABC DL1ABC K UR 599 TNX FER QSO 73 ES GL".into() }, seed, lead_s: base.lead_s.max(1.0), trail_s: 1.0, ..base.clone() });
                let st = stats(&g.samples, g.sample_rate, 700.0);
                ratio.extend(st.iter().map(|s| s.wide_vs_near_db));
                voice += st.iter().filter(|s| s.is_voice()).count();
                all.extend(st.iter().map(|s| s.corr));
            }
            println!("{name:>14}: p50 {:.2} p95 {:.2} max {:.2} ratio p50 {:.1} p95 {:.1} voice {voice}", pct(&all, 0.5), pct(&all, 0.95), pct(&all, 1.0), pct(&ratio, 0.5), pct(&ratio, 0.95));
        }
    }
}

#[cfg(test)]
mod checks {
    use super::*;
    use cwld_dsp::{generate, CwSpec};

    fn any_voice(spec: CwSpec) -> bool {
        let g = generate(&spec);
        let mut d = VoiceDetector::new(g.sample_rate);
        let mut hit = false;
        for c in g.samples.chunks(200) {
            let e = d.evals;
            d.push(c, 700.0);
            hit |= d.evals != e && d.stats.is_voice();
        }
        hit
    }

    #[test]
    fn keyed_cw_clicks_and_a_clean_sidetone_are_not_speech() {
        let text = "CQ CQ DE DL1ABC DL1ABC K UR 599 TNX 73".to_string();
        assert!(!any_voice(CwSpec { text: text.clone(), snr_db: Some(10.0), ..Default::default() }));
        assert!(!any_voice(CwSpec { text: text.clone(), snr_db: None, ..Default::default() }), "a clean sidetone");
        assert!(!any_voice(CwSpec { text: text.clone(), snr_db: Some(10.0), impulses_per_s: 15.0, impulse_db: 46.0, ..Default::default() }), "clicks");
        assert!(!any_voice(CwSpec { text, snr_db: Some(40.0), rise_ms: 1.0, ..Default::default() }), "strong hard keying");
    }
}
