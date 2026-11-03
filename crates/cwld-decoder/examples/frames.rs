use cwld_dsp::frontend::{Frontend, FrontendConfig};
use cwld_dsp::{generate, CwSpec};
fn main() {
    for snr in [10.0, 3.0, 0.0, -3.0, -6.0] {
        for (bwn, bw) in [(10.0, "10Hz"), (20.0, "20Hz"), (40.0, "40Hz")] {
            for afc in [false, true] {
                let base = CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K UR RST 599 BK PSE K".into(), wpm: 20.0, jitter: 0.0, seed: 3, lead_s: 4.0, ..Default::default() };
                let clean = generate(&base);
                let noisy = generate(&CwSpec { snr_db: Some(snr), ..base.clone() });
                let run = |s: &[f32], afc: bool| {
                    let mut c = FrontendConfig::new(8000, 700.0);
                    c.afc = afc;
                    let mut f = Frontend::new(c);
                    f.set_bandwidth(bwn);
                    let fr = f.push(s);
                    (fr, f.tone_hz())
                };
                let (fc, _) = run(&clean.samples, false);
                let (fnz, tone) = run(&noisy.samples, afc);
                let thr = fc.iter().map(|f| f.env).fold(0.0, f32::max) * 0.5;
                let mut err = 0; let mut tot = 0;
                for (a, b) in fc.iter().zip(&fnz).skip(1000) {
                    tot += 1;
                    if (a.env > thr) != (b.llr > 0.0) { err += 1; }
                }
                println!("snr {snr:+} bw {bw} afc {afc}: frame sign errors {:.1}%  tone {:.1}", 100.0 * err as f64 / tot as f64, tone);
            }
        }
    }
}
