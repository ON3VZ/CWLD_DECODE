//! Spelled words: `cargo run --release -p cwld-decoder --example spellbench`
//!
//! A word marked with `~` is keyed with a word gap (or more) between its letters, the way an
//! operator spells a name or a callsign. Measures the character error rate with and without
//! joining loose letters, and on texts where single letters belong alone (R R, E E, K), so a
//! gain on the first set does not hide a loss on the second.

use cwld_decoder::bench::{decode_samples, levenshtein};
use cwld_decoder::PipelineConfig;
use cwld_dsp::{generate, CwSpec};

const SPELLED: &[&str] = &[
    "NAME IS ~HANS HANS",
    "OP ~KRIS KRIS BK",
    "QTH ~GENT GENT",
    "DE ~DL4SAV DL4SAV K",
    "UR CALL ~ON3VZ ON3VZ",
    "MY NAME ~PIETER",
    "QTH ~HOBOKEN NR ANTWERP",
    "CALL IS ~F5XYZ K",
    "UR RST ~5NN TU",
    "NAME ~ANNA ES QTH ~LIER",
];

const LOOSE: &[&str] = &["R R TU 73", "73 E E", "QSL R 5NN K", "TU E E", "R R R FB", "BK R K", "K", "GM R TNX", "OK R R UR 599", "73 TU E E"];

fn main() {
    let trials = 3;
    println!("| Signal | Set | join off CER % | join on CER % | words right off | on |");
    println!("|---|---|---|---|---|---|");
    for (label, gap, snr, wpm) in [("gap 7, +15 dB, 20 WPM", 7.0, 15.0, 20.0), ("gap 10, +15 dB, 20 WPM", 10.0, 15.0, 20.0), ("gap 7, +6 dB, 25 WPM", 7.0, 6.0, 25.0), ("gap 12, +10 dB, 16 WPM", 12.0, 10.0, 16.0)] {
        for (set, texts) in [("spelled", SPELLED), ("loose letters", LOOSE)] {
            let mut res = [(0usize, 0usize, 0usize, 0usize); 2];
            for (ti, text) in texts.iter().enumerate() {
                for t in 0..trials {
                    let spec = CwSpec { text: text.to_string(), wpm, snr_db: Some(snr), jitter: 0.05, gap_jitter: 0.1, spell_gap: gap, seed: (ti * 31 + t) as u64 + 7, ..Default::default() };
                    let g = generate(&spec);
                    for (k, join) in [false, true].into_iter().enumerate() {
                        let mut cfg = PipelineConfig::new(8000, 700.0);
                        cfg.join_spelled = join;
                        let got = decode_samples(&g.samples, cfg);
                        if std::env::var("CWLD_SHOW").is_ok() {
                            println!("    {join:5} {:30} -> {got}", g.truth);
                        }
                        let r = &mut res[k];
                        r.0 += levenshtein(&g.truth, &got);
                        r.1 += g.truth.chars().count();
                        // The marked words (or every word for the loose set) exactly right.
                        let want: Vec<String> = text.split_whitespace().filter(|w| set != "spelled" || w.starts_with('~')).map(|w| w.trim_start_matches('~').to_string()).collect();
                        let words: Vec<&str> = got.split_whitespace().collect();
                        r.3 += want.len();
                        r.2 += want.iter().filter(|w| if set == "spelled" { words.contains(&w.as_str()) } else { got == g.truth }).count();
                    }
                }
            }
            let pct = |r: (usize, usize, usize, usize)| 100.0 * r.0 as f64 / r.1.max(1) as f64;
            let ok = |r: (usize, usize, usize, usize)| format!("{}/{}", r.2, r.3);
            println!("| {label} | {set} | {:.1} | {:.1} | {} | {} |", pct(res[0]), pct(res[1]), ok(res[0]), ok(res[1]));
        }
    }
}
