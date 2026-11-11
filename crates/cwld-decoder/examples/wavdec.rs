use cwld_decoder::bench::decode_samples;
use cwld_decoder::PipelineConfig;
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (sr, s) = cwld_dsp::wav::read(std::fs::File::open(&a[0]).unwrap()).unwrap();
    let tone: f64 = a[1].parse().unwrap();
    let mut cfg = PipelineConfig::new(sr, tone);
    cfg.auto_tone = a.get(2).is_some_and(|x| x == "auto");
    if let Some(m) = a.get(3) { cfg.min_separation = m.parse().unwrap(); }
    if std::env::var("NOSPACE").is_ok() { cfg.spacing = false; }
        if let Ok(w) = std::env::var("WEAK") { let v: Vec<f64> = w.split(',').map(|x| x.parse().unwrap()).collect(); cfg.weak_marks = Some((v[0], v[1], v[2])); }
    if std::env::var("NOSTYLE").is_ok() { cfg.decoder.learn_style = false; }
    if std::env::var("NOGATE").is_ok() { cfg.cw_gate = false; }
    if std::env::var("NOVOICE").is_ok() { cfg.voice_gate = false; }
    if std::env::var("NOLM").is_ok() { cfg.language_model = false; }
    println!("{}", decode_samples(&s, cfg));
}
