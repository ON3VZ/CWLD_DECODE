//! How long are noise blips behind the soft and the steep filter? Prints, for pure noise and for
//! keyed CW, the share of marks shorter than `k` noise correlation times (see `gate`).
//! `cargo run --release -p cwld-decoder --example gatesteep`

use cwld_dsp::frontend::{Frontend, FrontendConfig};
use cwld_dsp::{generate, CwSpec, FRAME_RATE};

fn short_frac(env: &[f32], bw: f64, k: f64) -> (f64, usize) {
    let mut v = env.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let thr = 0.5 * (v[v.len() / 4] + v[(v.len() as f64 * 0.97) as usize]);
    let tau = k * FRAME_RATE / bw;
    let (mut marks, mut short, mut run) = (0usize, 0usize, 0usize);
    for &e in env {
        if e > thr {
            run += 1;
        } else if run > 0 {
            marks += 1;
            if (run as f64) < tau {
                short += 1;
            }
            run = 0;
        }
    }
    (short as f64 / marks.max(1) as f64, marks)
}

fn main() {
    for (label, text, snr) in [("noise", "", 0.0), ("CW +3 dB", "CQ CQ DE ON3VZ ON3VZ K UR 599 599 NAME HANS QTH BERLIN", 3.0), ("CW +10 dB", "CQ CQ DE ON3VZ ON3VZ K UR 599 599 NAME HANS QTH BERLIN", 10.0)] {
        for bw in [12.0, 20.0, 36.0] {
            for order in [0usize, 4, 6] {
                print!("{label:10} bw {bw:4} order {order}:");
                for k in [1.0, 1.1, 1.15, 1.2] {
                    let (mut lo, mut hi) = (1.0f64, 0.0f64);
                    for seed in 1..=6 {
                        let g = generate(&CwSpec { text: text.into(), wpm: bw / 1.0, snr_db: Some(snr), lead_s: if text.is_empty() { 30.0 } else { 0.5 }, trail_s: 0.5, seed, ..Default::default() });
                        let mut c = FrontendConfig::new(8000, 700.0);
                        c.bandwidth_hz = bw;
                        c.steep_order = order;
                        c.afc = false;
                        let mut f = Frontend::new(c);
                        let env: Vec<f32> = f.push(&g.samples).iter().skip(400).map(|x| x.env).collect();
                        // Windows of 4 s, like the gate.
                        for w in env.chunks(800).filter(|w| w.len() == 800) {
                            let (s, m) = short_frac(w, bw, k);
                            if m >= 8 {
                                lo = lo.min(s);
                                hi = hi.max(s);
                            }
                        }
                    }
                    print!("  k={k}: {lo:.2}..{hi:.2}");
                }
                println!();
            }
        }
    }
}
