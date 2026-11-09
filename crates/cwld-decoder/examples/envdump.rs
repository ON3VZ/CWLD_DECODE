use cwld_dsp::frontend::{Frontend, FrontendConfig};
use cwld_dsp::{generate, CwSpec};
use std::io::Write;
fn dump(name: &str, sr: u32, s: &[f32], tone: f64) {
    let mut f = Frontend::new(FrontendConfig::new(sr, tone));
    let mut out = Vec::new();
    for c in s.chunks(sr as usize / 20) {
        for fr in f.push(c) { out.extend_from_slice(&fr.env.to_le_bytes()); }
    }
    std::fs::File::create(format!("/tmp/rec/env_{name}.f32")).unwrap().write_all(&out).unwrap();
}
fn wav(p: &str) -> (u32, Vec<f32>) { cwld_dsp::wav::read(std::fs::File::open(p).unwrap()).unwrap() }
fn main() {
    let (sr, s) = wav("/tmp/rec/noise0.wav");
    dump("n956", sr, &s, 956.0); dump("n700", sr, &s, 700.0);
    let (sr, s) = wav("/tmp/rec/cwld-20261104-134601-7047.2kHz_8k.wav"); dump("n7047", sr, &s, 700.0); dump("n7047b", sr, &s, 410.0);
    let (sr, s) = wav("/tmp/rec/cwld-20261104-134804-7035.4kHz_8k.wav"); dump("r2", sr, &s, 796.0);
    let (sr, s) = wav("/tmp/rec/cwld-20261104-134852-7012.0kHz_8k.wav"); dump("r3", sr, &s, 613.0);
    for snr in [10.0, 6.0, 3.0, 0.0, -3.0] {
        let g = generate(&CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K UR RST 599 BK PSE K CQ CQ DE ON3VZ K".into(), wpm: 20.0, snr_db: Some(snr), jitter: 0.05, seed: 5, ..Default::default() });
        dump(&format!("s{}", snr as i32), g.sample_rate, &g.samples, 700.0);
    }
    let w = generate(&CwSpec { text: "".into(), snr_db: Some(0.0), lead_s: 30.0, trail_s: 0.0, ..Default::default() });
    dump("white", w.sample_rate, &w.samples, 700.0);
}
