use cwld_dsp::seek::ToneSeeker;
use cwld_dsp::{generate, CwSpec};
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (sr, s) = if let Some(snr) = a[0].strip_prefix("gen:") {
        let g = generate(&CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K PSE K".into(), tone_hz: 615.0, snr_db: Some(snr.parse().unwrap()), lead_s: if snr == "noise" { 0.0 } else { 0.5 }, ..Default::default() });
        (8000, g.samples)
    } else {
        cwld_dsp::wav::read(std::fs::File::open(&a[0]).unwrap()).unwrap()
    };
    let mut k = ToneSeeker::new(sr, 300.0, 1200.0);
    let mut line = String::new();
    for c in s.chunks(2000) { if k.push(c) { let b = k.best().unwrap(); line += &format!("{:.0}:{:.0}/{} ", b.hz, b.score_db, b.steady); } }
    println!("{line}");
}
