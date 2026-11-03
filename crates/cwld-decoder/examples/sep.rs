use cwld_dsp::frontend::{Frontend, FrontendConfig};
use cwld_dsp::{generate, CwSpec};
fn main() {
    for snr in [3.0, 0.0, -3.0, -6.0, -9.0] {
        let mut r = vec![];
        for seed in 1..6u64 {
            let g = generate(&CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K UR RST 599 BK".into(), snr_db: Some(snr), wpm: 20.0, seed, lead_s: 3.0, ..Default::default() });
            let mut f = Frontend::new(FrontendConfig::new(8000, 700.0));
            for c in g.samples.chunks(4000) { f.push(c); }
            r.push(format!("{:.2}", f.presence_ratio()));
        }
        let mut n = vec![];
        for seed in 1..30u64 {
            let g = generate(&CwSpec { text: "".into(), snr_db: Some(snr), lead_s: 20.0, trail_s: 0.0, seed, ..Default::default() });
            let mut f = Frontend::new(FrontendConfig::new(8000, 700.0));
            let mut mx = 0.0f64;
            for (i,c) in g.samples.chunks(4000).enumerate() { f.push(c); if i>=10 { mx = mx.max(f.presence_ratio()); } }
            n.push(format!("{:.2}", mx));
        }
        println!("{snr:+} dB  signal ratio {:?}   noise-only max ratio {:?}", r, n);
    }
}
