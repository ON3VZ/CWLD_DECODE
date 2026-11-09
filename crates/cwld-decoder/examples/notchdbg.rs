use cwld_dsp::clean::AutoNotch;
use cwld_dsp::{generate, CwSpec};
fn main() {
    let g = generate(&CwSpec { text: "CQ CQ CQ DE ON3VZ ON3VZ K".into(), snr_db: Some(10.0), carrier: Some((50.0, 15.0)), lead_s: 3.0, jitter: 0.05, ..Default::default() });
    let mut an = AutoNotch::new(8000);
    for (i, x) in g.samples.iter().enumerate() {
        an.process(*x, 700.0);
        if an.take_changed() { println!("{:.2}s {:?}", i as f64 / 8000.0, an.active_notches()); }
    }
}
