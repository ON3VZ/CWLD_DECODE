use cwld_decoder::{Pipeline, PipelineConfig};
use cwld_dsp::{generate, CwSpec};
fn run(name: &str, sr: u32, s: &[f32], tone: f64) {
    let mut p = Pipeline::new(PipelineConfig::new(sr, tone));
    let mut line = String::new();
    let chunk = sr as usize / 4;
    for (i, c) in s.chunks(chunk).enumerate() {
        p.push(c);
        p.poll();
        if i % 20 == 19 && i > 40 {
            let g = p.gate_stats();
            line += &format!(" [{:.2} {:.2} {:.2} n{}]", g.marks_per_bw, g.short_frac, g.duty, g.marks);
        }
    }
    println!("{name:8}{line}");
}
fn wav(p: &str) -> (u32, Vec<f32>) { cwld_dsp::wav::read(std::fs::File::open(p).unwrap()).unwrap() }
fn main() {
    let (sr, s) = wav("/tmp/rec/noise0.wav");
    run("n956", sr, &s, 956.0); run("n700", sr, &s, 700.0);
    let (sr, s) = wav("/tmp/rec/cwld-20261104-134601-7047.2kHz_8k.wav"); run("n7047", sr, &s, 700.0); run("n7047b", sr, &s, 410.0);
    let (sr, s) = wav("/tmp/rec/cwld-20261104-134804-7035.4kHz_8k.wav"); run("r2", sr, &s, 796.0);
    let (sr, s) = wav("/tmp/rec/cwld-20261104-134852-7012.0kHz_8k.wav"); run("r3", sr, &s, 613.0);
    let w = generate(&CwSpec { text: "".into(), snr_db: Some(0.0), lead_s: 40.0, trail_s: 0.0, ..Default::default() });
    run("white", w.sample_rate, &w.samples, 700.0);
    for wpm in [12.0, 20.0, 30.0] {
        for snr in [10.0, 3.0, 0.0, -3.0] {
            let g = generate(&CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K UR RST 599 BK PSE K CQ CQ DE ON3VZ K TNX FER CALL UR RST 579 NAME KRIS QTH HOBOKEN HW? K".into(), wpm, snr_db: Some(snr), jitter: 0.05, seed: 5, ..Default::default() });
            run(&format!("w{wpm}s{snr}"), g.sample_rate, &g.samples, 700.0);
        }
    }
}
