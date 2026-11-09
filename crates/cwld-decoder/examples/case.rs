use cwld_decoder::bench::{decode_samples, levenshtein, TEXTS};
use cwld_decoder::PipelineConfig;
use cwld_dsp::{generate, CwSpec};
fn main() {
    let which = std::env::args().nth(1).unwrap_or_default();
    let base = CwSpec { wpm: 20.0, snr_db: Some(10.0), jitter: 0.05, ..Default::default() };
    let spec = match which.as_str() {
        "carrier" => CwSpec { carrier: Some((50.0, 15.0)), lead_s: 3.0, ..base },
        "carrier0" => CwSpec { carrier: Some((50.0, 15.0)), ..base },
        "qsb" => CwSpec { qsb_depth: 0.8, qsb_hz: 0.3, ..base },
        "qsb3" => CwSpec { qsb_depth: 0.8, qsb_hz: 0.3, snr_db: Some(3.0), ..base },
        "m3" => CwSpec { snr_db: Some(-3.0), ..base },
        "imp" => CwSpec { impulses_per_s: 30.0, impulse_db: 26.0, snr_db: Some(3.0), ..base },
        "int" => CwSpec { interferer: Some(cwld_dsp::gen::Interferer { offset_hz: 60.0, rel_db: 0.0, wpm: 17.0 }), ..base },
        "int2" => CwSpec { interferer: Some(cwld_dsp::gen::Interferer { offset_hz: 120.0, rel_db: 10.0, wpm: 24.0 }), ..base },
        "imp2" => CwSpec { impulses_per_s: 40.0, impulse_db: 40.0, snr_db: Some(6.0), ..base },
        "imp3" => CwSpec { impulses_per_s: 15.0, impulse_db: 46.0, snr_db: Some(10.0), ..base },
        _ => base,
    };
    let (mut e, mut n) = (0, 0);
    for (i, t) in TEXTS.iter().enumerate() {
        let g = generate(&CwSpec { text: t.to_string(), seed: 50 + i as u64, ..spec.clone() });
        let mut cfg = PipelineConfig::new(8000, 700.0);
        if std::env::args().any(|a| a == "nonb") { cfg.noise_blanker = false; }
        if std::env::args().any(|a| a == "nonotch") { cfg.auto_notch = false; }
        if std::env::args().any(|a| a == "rice") { cfg.rice = true; }
        if std::env::args().any(|a| a == "notrack") { cfg.track_fading = false; }
        if std::env::args().any(|a| a == "nolm") { cfg.language_model = false; }
        for a in std::env::args() {
            if let Some(v) = a.strip_prefix("es=") { cfg.decoder.evidence_scale = v.parse().unwrap(); }
            if let Some(v) = a.strip_prefix("junk=") { cfg.junk_strength = v.parse().unwrap(); }
            if let Some(v) = a.strip_prefix("bw=") { cfg.bandwidth_factor = v.parse().unwrap(); }
        }
        let got = decode_samples(&g.samples, cfg);
        let d = levenshtein(&g.truth, &got);
        e += d; n += g.truth.len();
        println!("{} {:?}\n    {:?}", if d == 0 { "ok " } else { "ERR" }, g.truth, got);
    }
    println!("CER {:.1}%", 100.0 * e as f64 / n as f64);
}
