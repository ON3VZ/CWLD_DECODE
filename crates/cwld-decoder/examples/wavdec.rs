use cwld_decoder::bench::decode_samples;
use cwld_decoder::PipelineConfig;
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (sr, s) = cwld_dsp::wav::read(std::fs::File::open(&a[0]).unwrap()).unwrap();
    let tone: f64 = a[1].parse().unwrap();
    let mut cfg = PipelineConfig::new(sr, tone);
    cfg.auto_tone = a.get(2).is_some_and(|x| x == "auto");
    if let Some(m) = a.get(3) { cfg.min_separation = m.parse().unwrap(); }
    println!("{}", decode_samples(&s, cfg));
}
