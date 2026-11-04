//! Any sound card rate down to the decoder rate (8 kHz).
//!
//! A windowed-sinc low-pass at the input rate removes everything above about 3.2 kHz, then the
//! output is read between input samples by linear interpolation. CW lives below 1.5 kHz, so
//! this is plenty, and it works for 48, 44.1 and 16 kHz alike.

use std::f64::consts::PI;

pub struct Resampler {
    step: f64,
    taps: Vec<f64>,
    hist: Vec<f64>,
    mask: usize,
    /// Input samples seen so far.
    count: u64,
    /// Next output position, in input samples.
    next: f64,
    passthrough: bool,
}

impl Resampler {
    pub fn new(in_rate: u32, out_rate: u32) -> Self {
        let passthrough = in_rate == out_rate;
        let n = ((in_rate as f64 / 375.0).round() as usize).max(16) | 1;
        let fc = 3200.0 / in_rate as f64;
        let mid = (n - 1) as f64 / 2.0;
        let mut taps: Vec<f64> = (0..n)
            .map(|i| {
                let x = i as f64 - mid;
                let sinc = if x == 0.0 { 2.0 * fc } else { (2.0 * PI * fc * x).sin() / (PI * x) };
                let w = 0.42 - 0.5 * (2.0 * PI * i as f64 / (n - 1) as f64).cos() + 0.08 * (4.0 * PI * i as f64 / (n - 1) as f64).cos();
                sinc * w
            })
            .collect();
        let sum: f64 = taps.iter().sum();
        for t in taps.iter_mut() {
            *t /= sum;
        }
        let size = (n + 4).next_power_of_two();
        Resampler { step: in_rate as f64 / out_rate as f64, taps, hist: vec![0.0; size], mask: size - 1, count: 0, next: 0.0, passthrough }
    }

    /// Filter output at input index `j` (needs samples j-N+1..=j in the history).
    fn fir(&self, j: u64) -> f64 {
        let n = self.taps.len();
        let mut acc = 0.0;
        for (k, t) in self.taps.iter().enumerate() {
            let idx = j as i64 - (n - 1 - k) as i64;
            if idx >= 0 {
                acc += t * self.hist[idx as usize & self.mask];
            }
        }
        acc
    }

    pub fn process(&mut self, input: &[f32], out: &mut Vec<f32>) {
        if self.passthrough {
            out.extend_from_slice(input);
            return;
        }
        for &x in input {
            self.hist[self.count as usize & self.mask] = x as f64;
            let k = self.count;
            self.count += 1;
            // Produce every output whose two neighbouring input samples are now known.
            while self.next + 1.0 <= k as f64 {
                let j = self.next.floor() as u64;
                let frac = self.next - j as f64;
                let y0 = self.fir(j);
                let y1 = self.fir(j + 1);
                out.push((y0 + frac * (y1 - y0)) as f32);
                self.next += self.step;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(rate: u32, hz: f64, secs: f64) -> Vec<f32> {
        (0..(rate as f64 * secs) as usize).map(|i| (2.0 * PI * hz * i as f64 / rate as f64).sin() as f32 * 0.5).collect()
    }

    fn rms(x: &[f32]) -> f64 {
        (x.iter().map(|v| (*v as f64).powi(2)).sum::<f64>() / x.len() as f64).sqrt()
    }

    #[test]
    fn length_and_level_are_right_for_common_rates() {
        for rate in [48_000u32, 44_100, 16_000] {
            let mut r = Resampler::new(rate, 8000);
            let mut out = Vec::new();
            r.process(&tone(rate, 700.0, 2.0), &mut out);
            assert!((out.len() as i64 - 16_000).abs() < 40, "{rate}: {}", out.len());
            let level = rms(&out[1000..]);
            assert!((level - 0.3535).abs() < 0.01, "{rate}: {level}");
        }
    }

    #[test]
    fn frequency_is_kept() {
        let mut r = Resampler::new(48_000, 8000);
        let mut out = Vec::new();
        r.process(&tone(48_000, 700.0, 1.0), &mut out);
        // Count zero crossings: 700 Hz gives 1400 per second.
        let z = out.windows(2).skip(500).filter(|w| (w[0] < 0.0) != (w[1] < 0.0)).count() as f64;
        let secs = (out.len() - 501) as f64 / 8000.0;
        assert!((z / secs - 1400.0).abs() < 10.0, "{}", z / secs);
    }

    #[test]
    fn high_tones_do_not_fold_back() {
        // 6 kHz at 48 kHz would alias to 2 kHz at 8 kHz without the filter.
        let mut r = Resampler::new(48_000, 8000);
        let mut out = Vec::new();
        r.process(&tone(48_000, 6000.0, 1.0), &mut out);
        assert!(rms(&out[500..]) < 0.01, "{}", rms(&out[500..]));
    }

    #[test]
    fn chunked_input_gives_the_same_output() {
        let x = tone(48_000, 700.0, 0.5);
        let mut a = Resampler::new(48_000, 8000);
        let mut b = Resampler::new(48_000, 8000);
        let (mut oa, mut ob) = (Vec::new(), Vec::new());
        a.process(&x, &mut oa);
        for c in x.chunks(37) {
            b.process(c, &mut ob);
        }
        assert_eq!(oa, ob);
    }
}
