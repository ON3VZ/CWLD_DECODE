use cwld_dsp::frontend::{Frontend, FrontendConfig};
use cwld_dsp::{generate, CwSpec};
fn main() {
    let base = CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K UR RST 599 BK PSE K CQ CQ".into(), wpm: 20.0, seed: 3, lead_s: 4.0, qsb_depth: 0.8, qsb_hz: 0.3, ..Default::default() };
    let flat = generate(&CwSpec { qsb_depth: 0.0, ..base.clone() });
    let faded = generate(&base);
    let noisy = generate(&CwSpec { snr_db: Some(10.0), ..base.clone() });
    let run = |s: &[f32]| { let mut c = FrontendConfig::new(8000, 700.0); c.noise_blanker = false; c.auto_notch=false; let mut f = Frontend::new(c); f.set_bandwidth(20.0); f.push(s) };
    let (ff, fd, fz) = (run(&flat.samples), run(&faded.samples), run(&noisy.samples));
    let peak = ff.iter().map(|f| f.env).fold(0.0, f32::max);
    let mut bins = vec![(0usize, 0usize); 6];
    let (mut off_err, mut off_tot) = (0, 0);
    for i in 1000..ff.len().min(fz.len()) {
        let on = ff[i].env > peak * 0.5;
        if on {
            let amp = fd[i].env / peak; // 0.2 .. 1
            let b = ((amp * 5.0) as usize).min(5);
            bins[b].1 += 1;
            if fz[i].llr <= 0.0 { bins[b].0 += 1; }
        } else { off_tot += 1; if fz[i].llr > 0.0 { off_err += 1; } }
    }
    for (b, (e, t)) in bins.iter().enumerate() { if *t > 0 { println!("amp {:.1}-{:.1}: on-frames read as off {:.1}% of {}", b as f32 / 5.0, (b + 1) as f32 / 5.0, 100.0 * *e as f64 / *t as f64, t); } }
    println!("off-frames read as on {:.1}%", 100.0 * off_err as f64 / off_tot as f64);
}
