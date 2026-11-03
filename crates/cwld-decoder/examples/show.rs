use cwld_decoder::bench::{decode_samples, levenshtein, TEXTS};
use cwld_decoder::PipelineConfig;
use cwld_dsp::{generate, CwSpec};
fn main() {
    let a: Vec<f64> = std::env::args().skip(1).map(|x| x.parse().unwrap()).collect();
    let (snr, wpm) = (a.get(0).copied().unwrap_or(30.0), a.get(1).copied().unwrap_or(20.0));
    let bw = a.get(2).copied();
    let es = a.get(3).copied();
    for (i, t) in TEXTS.iter().enumerate() {
        let spec = CwSpec { text: t.to_string(), wpm, snr_db: Some(snr), jitter: 0.05, seed: 100 + i as u64, ..Default::default() };
        let g = generate(&spec);
        let mut cfg = PipelineConfig::new(8000, 700.0);
        if let Some(b) = bw { cfg.bandwidth_factor = b; }
        if let Some(e) = es { cfg.decoder.evidence_scale = e; }
        let got = decode_samples(&g.samples, cfg);
        let d = levenshtein(&g.truth, &got);
        println!("{} {:?}\n   {:?}", if d == 0 { "ok " } else { "ERR" }, g.truth, got);
    }
}
