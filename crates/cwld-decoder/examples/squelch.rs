use cwld_decoder::bench::{decode_samples, run_cell};
use cwld_decoder::PipelineConfig;
use cwld_dsp::{generate, CwSpec};
fn wav(p: &str) -> Vec<f32> {
    cwld_dsp::wav::read(std::fs::File::open(p).unwrap()).unwrap().1
}
fn main() {
    let noise = wav("/tmp/rec/cwld-20261104-134601-7047.2kHz_8k.wav");
    let f2 = wav("/tmp/rec/cwld-20261104-134804-7035.4kHz_8k.wav");
    let f3 = wav("/tmp/rec/cwld-20261104-134852-7012.0kHz_8k.wav");
    let white = generate(&CwSpec { text: "".into(), snr_db: Some(0.0), lead_s: 30.0, trail_s: 0.0, ..Default::default() }).samples;
    let snrs = [6.0, 3.0, 0.0, -3.0];
    for ms in [2.0f32, 3.5, 4.0, 4.5, 5.0, 6.0] {
        let cfg = |tone: f64| { let mut p = PipelineConfig::new(8000, tone); p.min_separation = ms; p };
        let mut row = String::new();
        for snr in snrs {
            let base = CwSpec { wpm: 20.0, snr_db: Some(snr), jitter: 0.05, ..Default::default() };
            let c = run_cell(&base, 8, 777, |s| cfg(s.tone_hz));
            row += &format!(" {:5.1}", c.cer * 100.0);
        }
        let n700 = decode_samples(&noise, cfg(700.0)).replace(' ', "").len();
        let n410 = decode_samples(&noise, cfg(410.0)).replace(' ', "").len();
        let w = decode_samples(&white, cfg(700.0)).replace(' ', "").len();
        let t2 = decode_samples(&f2, cfg(796.0));
        let t3 = decode_samples(&f3, cfg(613.0));
        println!("sep {ms:3.1} | CER {row} | noise700 {n700} noise410 {n410} white {w}\n   f2: {t2}\n   f3: {t3}");
    }
}
