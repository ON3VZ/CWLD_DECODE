//! Waterfall rows: 20 per second, 1024-point FFT at 8 kHz (7.8 Hz per bin), only the part of
//! the spectrum where CW tones live. Values 0..255 relative to the noise floor, so the picture
//! looks the same whatever the audio level.

use crate::fft::fft;
use std::f64::consts::PI;

pub const SAMPLE_RATE: f64 = 8000.0;
pub const SIZE: usize = 1024;
pub const HOP: usize = 400;

pub struct Row {
    pub bins: Vec<u8>,
    pub lo_hz: f64,
    pub hz_per_bin: f64,
}

pub struct Waterfall {
    window: Vec<f64>,
    /// Circular buffer of the last `SIZE` samples, `pos` is the oldest.
    buf: Vec<f32>,
    pos: usize,
    filled: usize,
    since_row: usize,
    lo: usize,
    hi: usize,
    /// Per bin: a running median of the level (dB), so the radio's filter shape is not mistaken
    /// for signal. Steps of at most 1 dB a second, large steps at the start.
    floors: Vec<f64>,
    rows: u32,
}

impl Waterfall {
    pub fn new(lo_hz: f64, hi_hz: f64) -> Self {
        let res = SAMPLE_RATE / SIZE as f64;
        let window = (0..SIZE).map(|i| 0.5 - 0.5 * (2.0 * PI * i as f64 / (SIZE - 1) as f64).cos()).collect();
        let lo = (lo_hz / res).floor().max(1.0) as usize;
        let hi = ((hi_hz / res).ceil() as usize).min(SIZE / 2 - 1).max(lo + 1);
        Waterfall { window, buf: vec![0.0; SIZE], pos: 0, filled: 0, since_row: 0, lo, hi, floors: Vec::new(), rows: 0 }
    }

    pub fn hz_per_bin() -> f64 {
        SAMPLE_RATE / SIZE as f64
    }

    pub fn push(&mut self, samples: &[f32]) -> Vec<Row> {
        let mut rows = Vec::new();
        for &x in samples {
            self.buf[self.pos] = x;
            self.pos = (self.pos + 1) % SIZE;
            self.filled = (self.filled + 1).min(SIZE);
            self.since_row += 1;
            if self.since_row >= HOP && self.filled == SIZE {
                self.since_row = 0;
                rows.push(self.row());
            }
        }
        rows
    }

    fn row(&mut self) -> Row {
        let mut re: Vec<f64> = (0..SIZE).map(|i| self.buf[(self.pos + i) % SIZE] as f64 * self.window[i]).collect();
        let mut im = vec![0.0; SIZE];
        fft(&mut re, &mut im);
        let db: Vec<f64> = (self.lo..=self.hi).map(|k| 10.0 * (re[k] * re[k] + im[k] * im[k] + 1e-20).log10()).collect();
        if self.floors.len() != db.len() {
            self.floors = db.clone();
            self.rows = 0;
        }
        // Stochastic median per bin: one small step towards the new value, big steps at the start.
        let step = (2.0 * 0.97f64.powi(self.rows as i32)).max(0.05);
        self.rows = self.rows.saturating_add(1);
        for (f, d) in self.floors.iter_mut().zip(&db) {
            *f += step * (d - *f).signum();
        }
        // A narrow signal only lifts a few bins, so the median of the neighbours (about 80 Hz
        // either side) is the noise level of the radio's filter at that pitch.
        let n = self.floors.len();
        let local: Vec<f64> = (0..n)
            .map(|k| {
                let mut v: Vec<f64> = self.floors[k.saturating_sub(10)..(k + 11).min(n)].to_vec();
                v.sort_by(|a, b| a.partial_cmp(b).unwrap());
                // The median of the level of noise is about 3.8 dB above its 25th percentile.
                v[v.len() / 2] - 3.8
            })
            .collect();
        // From 3 dB below the floor (dark) to 40 dB above it (white).
        let bins = db.iter().zip(&local).map(|(d, f)| (((d - f + 3.0) / 43.0) * 255.0).clamp(0.0, 255.0) as u8).collect();
        Row { bins, lo_hz: self.lo as f64 * Self::hz_per_bin(), hz_per_bin: Self::hz_per_bin() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twenty_rows_a_second_and_the_tone_is_bright() {
        let mut wf = Waterfall::new(200.0, 1400.0);
        let mut rng = cwld_dsp::Rng::new(4);
        let x: Vec<f32> = (0..8000).map(|i| ((2.0 * PI * 700.0 * i as f64 / 8000.0).sin() * 0.3 + 0.01 * rng.gauss()) as f32).collect();
        let rows = wf.push(&x);
        assert!((rows.len() as i64 - 18).abs() <= 2, "{}", rows.len());
        let r = rows.last().unwrap();
        let k = ((700.0 - r.lo_hz) / r.hz_per_bin).round() as usize;
        let brightest = r.bins.iter().enumerate().max_by_key(|(_, v)| **v).unwrap().0;
        assert!((brightest as i64 - k as i64).abs() <= 1, "{brightest} vs {k}");
        assert!(r.bins[k] > 200);
        assert!(r.bins[r.bins.len() - 5] < 80);
    }

    #[test]
    fn the_radios_filter_shape_does_not_light_up_the_picture() {
        // Noise only in 200 to 900 Hz, nothing above: the quiet part must not make the noise bright.
        let mut wf = Waterfall::new(200.0, 1400.0);
        let mut rng = cwld_dsp::Rng::new(9);
        let mut lp = 0.0f64;
        let x: Vec<f32> = (0..8000 * 12)
            .map(|_| {
                let w = rng.gauss();
                // crude low-pass at about 900 Hz, strong roll-off through repeated filtering
                lp += 0.5 * (w - lp);
                lp as f32 * 0.2
            })
            .collect();
        let rows = wf.push(&x);
        let r = rows.last().unwrap();
        let k = ((500.0 - r.lo_hz) / r.hz_per_bin).round() as usize;
        let inband: f64 = r.bins[k - 20..k + 20].iter().map(|v| *v as f64).sum::<f64>() / 40.0;
        assert!(inband < 70.0, "in-band noise too bright: {inband}");
    }
}
