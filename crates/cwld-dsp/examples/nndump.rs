//! Writes training data for the small key-down network: per frame the envelope, the classic
//! log-likelihood ratio and whether the key was really down, for many random synthetic signals.
//!
//! Output: little-endian f32 records `[file, env, llr, key, snr_db, wpm, qsb]`.
//! Run: cargo run --release -p cwld-dsp --example nndump -- <out.bin> <files> <seed>

use cwld_dsp::{generate, CwSpec, Frontend, FrontendConfig, Rng, FRAME_RATE};
use std::io::Write;

const WORDS: &[&str] = &[
    "CQ", "DE", "K", "UR", "RST", "599", "579", "5NN", "TU", "73", "NAME", "QTH", "OM", "ES", "FB", "TNX", "FER", "CALL", "RIG", "ANT", "PWR", "WX", "HW", "CPY", "BK", "AGN", "PSE", "QRS", "QRZ", "GM", "GA", "GE",
    "DIPOLE", "RAIN", "SUNNY", "COLD", "HANS", "PETER", "BERLIN", "LONDON", "PARIS", "TEST", "THE", "AND", "HERE", "WATTS", "100W", "5W",
];

fn text(rng: &mut Rng) -> String {
    const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut out = String::new();
    for _ in 0..(6 + rng.below(8)) {
        if rng.uniform() < 0.55 {
            out.push_str(WORDS[rng.below(WORDS.len())]);
        } else {
            for _ in 0..(2 + rng.below(5)) {
                out.push(A[rng.below(A.len())] as char);
            }
        }
        out.push(' ');
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.get(1).cloned().unwrap_or_else(|| "nn.bin".into());
    let files: usize = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(200);
    let seed: u64 = args.get(3).and_then(|v| v.parse().ok()).unwrap_or(1);
    let mut rng = Rng::new(seed);
    let mut w = std::io::BufWriter::new(std::fs::File::create(&out).expect("output file"));
    let mut total = 0usize;
    for f in 0..files {
        let wpm = 12.0 + rng.uniform() * 26.0;
        let snr = if rng.uniform() < 0.08 { None } else { Some(-8.0 + rng.uniform() * 23.0) };
        let qsb = if rng.uniform() < 0.5 { 0.0 } else { 0.3 + rng.uniform() * 0.6 };
        let spec = CwSpec {
            text: text(&mut rng),
            wpm,
            snr_db: snr,
            qsb_depth: qsb,
            qsb_hz: 0.1 + rng.uniform() * 1.4,
            jitter: rng.uniform() * 0.12,
            gap_jitter: rng.uniform() * 0.3,
            dash_ratio: 2.6 + rng.uniform() * 0.8,
            seed: seed.wrapping_mul(100_000).wrapping_add(f as u64),
            ..Default::default()
        };
        let g = generate(&spec);
        let mut fe = Frontend::new(FrontendConfig::new(g.sample_rate, spec.tone_hz));
        // As in the pipeline: weak marks in fading count.
        fe.set_weak_marks(true);
        // The pipeline sets the filter from the speed it estimates, which is never exact.
        fe.set_bandwidth(1.2 / (1.2 / wpm) * (0.8 + rng.uniform() * 0.45));
        let mut frames = Vec::new();
        for c in g.samples.chunks(200) {
            frames.extend(fe.push(c));
        }
        // Key state per frame, without any delay (the trainer aligns it).
        let mut edges: Vec<(f64, bool)> = Vec::new();
        let mut t = g.lead_s;
        for (down, d) in &g.keying {
            edges.push((t, *down));
            t += d;
        }
        edges.push((t, false));
        let mut e = 0usize;
        for (i, fr) in frames.iter().enumerate() {
            let now = (i as f64 + 0.5) / FRAME_RATE;
            while e + 1 < edges.len() && edges[e + 1].0 <= now {
                e += 1;
            }
            let key = if now >= edges[0].0 && edges[e].1 { 1.0f32 } else { 0.0 };
            for v in [f as f32, fr.env, fr.llr, key, snr.unwrap_or(40.0) as f32, wpm as f32, qsb as f32] {
                w.write_all(&v.to_le_bytes()).unwrap();
            }
        }
        total += frames.len();
    }
    w.flush().unwrap();
    eprintln!("{files} files, {total} frames -> {out}");
}
