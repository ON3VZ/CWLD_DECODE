use cwld_dsp::frontend::{Frontend, FrontendConfig};
use cwld_dsp::{generate, CwSpec};
fn main() {
    for snr in [10.0, 3.0] {
        for rice in [false, true] {
            let base = CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K UR RST 599 BK PSE K".into(), wpm: 20.0, seed: 3, lead_s: 4.0, qsb_depth: 0.8, qsb_hz: 0.3, ..Default::default() };
            let clean = generate(&base);
            let noisy = generate(&CwSpec { snr_db: Some(snr), ..base.clone() });
            let run = |s: &[f32], rice: bool| {
                let mut c = FrontendConfig::new(8000, 700.0);
                c.track_fading = rice;
                c.noise_blanker = false;
                let mut f = Frontend::new(c);
                f.set_bandwidth(20.0);
                let fr = f.push(s);
                (fr, f)
            };
            let clean_flat = generate(&CwSpec { qsb_depth: 0.0, ..base.clone() });
            let (fc, _) = run(&clean_flat.samples, false);
            let (fnz, f) = run(&noisy.samples, rice);
            let thr = fc.iter().map(|f| f.env).fold(0.0, f32::max) * 0.5;
            let (mut err, mut tot, mut llr_on, mut llr_off, mut non, mut noff) = (0, 0, 0.0f64, 0.0f64, 0, 0);
            for (a, b) in fc.iter().zip(&fnz).skip(1000) {
                tot += 1;
                let on = a.env > thr;
                if on != (b.llr > 0.0) { err += 1; }
                if on { llr_on += b.llr as f64; non += 1 } else { llr_off += b.llr as f64; noff += 1 }
            }
            println!("snr {snr:+} track {rice}: sign errors {:.1}%  mean llr on {:.2} off {:.2}  levels {:?}", 100.0 * err as f64 / tot as f64, llr_on / non as f64, llr_off / noff as f64, f.levels().map(|l| (l.noise, l.signal, l.separation)));
        }
    }
}
