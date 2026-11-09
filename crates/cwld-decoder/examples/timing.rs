use cwld_dsp::frontend::{Frontend, FrontendConfig};
use cwld_dsp::{generate, CwSpec};
use std::time::Instant;
fn main() {
    let g = generate(&CwSpec { text: "CQ CQ DE ON3VZ".into(), snr_db: Some(10.0), carrier: Some((50.0, 15.0)), lead_s: 3.0, ..Default::default() });
    let t = Instant::now();
    let mut f = Frontend::new(FrontendConfig::new(8000, 700.0));
    let mut fr = Vec::new();
    for c in g.samples.chunks(200) { fr.extend(f.push(c)); }
    println!("frontend {:?} for {} s audio, notches {:?}, present {}", t.elapsed(), g.samples.len() / 8000, f.notches(), f.signal_present());
    let mut d = cwld_decoder::Decoder::new(Default::default());
    let t = Instant::now();
    for (i, c) in fr.chunks(5).enumerate() {
        d.push_llr(&c.iter().map(|x| x.llr).collect::<Vec<_>>());
        if i % 100 == 0 { println!("  {} frames {:?}", i * 5, t.elapsed()); }
    }
    println!("decoder {:?}", t.elapsed());
}
