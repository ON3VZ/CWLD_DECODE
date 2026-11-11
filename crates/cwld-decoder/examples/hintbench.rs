//! Do callsign hints (DX cluster spots next to the VFO) help, and can a wrong hint do harm?
//!
//! Every message holds a callsign the decoder has never seen. It is decoded three times: without
//! hints, with the right callsign among the hints, and with only wrong hints (one of them one
//! character away from the real call). Reported: wrong characters in percent and how often the
//! callsign came out exactly right.
//!
//! Run: cargo run --release -p cwld-decoder --example hintbench

use cwld_decoder::bench::levenshtein;
use cwld_decoder::{Pipeline, PipelineConfig};
use cwld_dsp::{generate, CwSpec, Rng};

fn call(rng: &mut Rng) -> String {
    const PRE: [&str; 14] = ["DL", "PA", "ON", "F", "G", "OK", "SP", "EA", "I", "OE", "HB9", "SM", "LA", "OZ"];
    const L: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let p = PRE[rng.below(PRE.len())];
    let mut s = String::from(p);
    if !p.ends_with('9') {
        s.push((b'0' + rng.below(10) as u8) as char);
    }
    for _ in 0..(2 + rng.below(2)) {
        s.push(L[rng.below(L.len())] as char);
    }
    s
}

/// The same call with one character replaced by another letter or digit of the same kind.
fn near_miss(c: &str, rng: &mut Rng) -> String {
    let mut v: Vec<char> = c.chars().collect();
    let i = v.len() - 1 - rng.below(2);
    let old = v[i];
    loop {
        let n = (b'A' + rng.below(26) as u8) as char;
        if n != old {
            v[i] = n;
            break;
        }
    }
    v.into_iter().collect()
}

/// The hardest wrong hint: the same call with one character that has exactly one dot or dash
/// more (E becomes I, T becomes M). That is what a lost element would look like the other way round.
fn one_element_more(c: &str, rng: &mut Rng) -> Option<String> {
    let v: Vec<char> = c.chars().collect();
    let start = rng.below(v.len());
    for k in 0..v.len() {
        let i = (start + k) % v.len();
        let pat = cwld_types::morse::pattern(v[i])?;
        for add in [".", "-"] {
            for cand in (b'A'..=b'Z').chain(b'0'..=b'9') {
                let ch = cand as char;
                if cwld_types::morse::pattern(ch) == Some(format!("{pat}{add}").as_str()) && ch.is_ascii_digit() == v[i].is_ascii_digit() {
                    let mut w = v.clone();
                    w[i] = ch;
                    return Some(w.into_iter().collect());
                }
            }
        }
    }
    None
}

fn decode(samples: &[f32], rate: u32, hints: &[String]) -> String {
    let mut cfg = PipelineConfig::new(rate, 700.0);
    cfg.auto_tone = false;
    let mut p = Pipeline::new(cfg);
    p.add_words(&["ON3VZ"]);
    let refs: Vec<&str> = hints.iter().map(|s| s.as_str()).collect();
    p.set_hints(&refs);
    let mut text = String::new();
    for c in samples.chunks((rate as usize / 40).max(1)) {
        p.push(c);
        text.push_str(&p.poll().committed);
    }
    p.flush();
    text.push_str(&p.poll().committed);
    text.trim().to_string()
}

#[derive(Default)]
struct Tally {
    chars: usize,
    errors: usize,
    calls: usize,
    calls_ok: usize,
    /// A wrong hint appeared in the text although it was never sent.
    planted: usize,
}

fn main() {
    let trials: usize = std::env::var("TRIALS").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let cases: Vec<(&str, CwSpec)> = vec![
        ("clean, +15 dB", CwSpec { wpm: 22.0, snr_db: Some(15.0), jitter: 0.03, ..Default::default() }),
        ("3 in 100 elements lost", CwSpec { wpm: 22.0, snr_db: Some(15.0), jitter: 0.03, lost_elements: 0.03, ..Default::default() }),
        ("6 in 100 elements lost, +6 dB", CwSpec { wpm: 22.0, snr_db: Some(6.0), jitter: 0.03, lost_elements: 0.06, ..Default::default() }),
        ("sloppy gaps (hand key)", CwSpec { wpm: 20.0, snr_db: Some(12.0), jitter: 0.10, gap_jitter: 0.35, ..Default::default() }),
        ("weak, 0 dB", CwSpec { wpm: 22.0, snr_db: Some(0.0), jitter: 0.03, ..Default::default() }),
        ("deep QSB", CwSpec { wpm: 22.0, snr_db: Some(10.0), jitter: 0.03, qsb_depth: 0.8, qsb_hz: 0.5, ..Default::default() }),
    ];
    println!("{:<32} {:>22} {:>22} {:>30}", "case", "no hints", "right hint", "only wrong hints");
    println!("{:<32} {:>22} {:>22} {:>30}", "", "CER%  call ok%", "CER%  call ok%", "CER%  call ok%  planted");
    for (name, base) in cases {
        let mut t = [Tally::default(), Tally::default(), Tally::default()];
        let mut rng = Rng::new(4242);
        for k in 0..trials {
            let dx = call(&mut rng);
            let text = match k % 3 {
                0 => format!("CQ CQ DE {dx} {dx} K"),
                1 => format!("ON3VZ DE {dx} UR 579 579 NAME HANS BK"),
                _ => format!("TU 5NN 14 {dx}"),
            };
            let mut wrong = vec![near_miss(&dx, &mut rng), call(&mut rng), call(&mut rng)];
            if let Some(w) = one_element_more(&dx, &mut rng) {
                wrong.push(w);
            }
            let mut right = wrong.clone();
            right.insert(1, dx.clone());
            let spec = CwSpec { text, seed: 9000 + k as u64, ..base.clone() };
            let g = generate(&spec);
            for (i, hints) in [Vec::new(), right, wrong.clone()].iter().enumerate() {
                let d = decode(&g.samples, spec.sample_rate, hints);
                t[i].chars += g.truth.chars().count();
                t[i].errors += levenshtein(&g.truth, &d);
                let sent = g.truth.split_whitespace().filter(|w| *w == dx).count();
                let got = d.split_whitespace().filter(|w| *w == dx).count();
                t[i].calls += sent;
                t[i].calls_ok += got.min(sent);
                if i == 2 {
                    t[i].planted += d.split_whitespace().filter(|w| wrong.iter().any(|x| x == w)).count();
                }
                if std::env::var("CWLD_SHOW").is_ok() && i > 0 {
                    println!("    [{i}] truth {} | decoded {}", g.truth, d);
                }
            }
        }
        let f = |x: &Tally| format!("{:5.1}  {:5.1}", 100.0 * x.errors as f64 / x.chars.max(1) as f64, 100.0 * x.calls_ok as f64 / x.calls.max(1) as f64);
        println!("{:<32} {:>22} {:>22} {:>22} {:>7}", name, f(&t[0]), f(&t[1]), f(&t[2]), t[2].planted);
    }
}
