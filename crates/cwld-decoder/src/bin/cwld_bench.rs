//! `cargo run --release -p cwld-decoder --bin cwld_bench` prints the character error rate
//! for a grid of signal to noise ratios and speeds, and for some difficult signals.

use cwld_decoder::bench::run_cell;
use cwld_decoder::PipelineConfig;
use cwld_dsp::gen::Interferer;
use cwld_dsp::CwSpec;

fn main() {
    let quick = std::env::args().any(|a| a == "--quick");
    let trials = if quick { 4 } else { 12 };
    // --plain switches off the noise blanker, the notch, the fading follower and the word
    // model, for a before/after comparison on the same signals.
    let plain = std::env::args().any(|a| a == "--plain");
    let pcfg = move |s: &CwSpec| {
        let mut p = PipelineConfig::new(s.sample_rate, 700.0);
        if let Some(a) = std::env::var("CWLD_NN").ok().and_then(|v| v.parse::<f32>().ok()) {
            p.neural = Some(a);
        }
        // CWLD_STEEP=4: the steep filter of that order. CWLD_WIDE=1: the wide filter.
        if let Some(o) = std::env::var("CWLD_STEEP").ok().and_then(|v| v.parse::<usize>().ok()) {
            p.steep_order = o;
        }
        if std::env::var("CWLD_WIDE").is_ok() {
            p.bandwidth_factor = 2.4;
        }
        if let Some(f) = std::env::var("CWLD_BWF").ok().and_then(|v| v.parse::<f64>().ok()) {
            p.bandwidth_factor = f;
        }
        if std::env::var("CWLD_NONB").is_ok() {
            p.noise_blanker = false;
        }
        if std::env::var("CWLD_NONOTCH").is_ok() {
            p.auto_notch = false;
        }
        if std::env::var("CWLD_NOJOIN").is_ok() {
            p.join_spelled = false;
        }
        if std::env::var("CWLD_NOGATE").is_ok() {
            p.cw_gate = false;
        }
        if std::env::var("CWLD_NOVOICE").is_ok() {
            p.voice_gate = false;
        }
        if let Ok(w) = std::env::var("WEAK") {
            let v: Vec<f64> = w.split(',').map(|x| x.parse().unwrap()).collect();
            p.weak_marks = Some((v[0], v[1], v[2]));
        }
        if plain {
            p.noise_blanker = false;
            p.auto_notch = false;
            p.track_fading = false;
            p.language_model = false;
            p.spacing = false;
            p.decoder.learn_style = false;
        }
        p
    };
    if plain {
        println!("PLAIN: noise blanker, notch, fading follower and word model off\n");
    }

    if std::env::args().any(|a| a == "--fists") {
        fists_only(trials, pcfg);
        return;
    }
    if std::env::args().any(|a| a == "--neighbours") {
        neighbours(trials, pcfg);
        return;
    }
    println!("Character error rate (%), SNR in 500 Hz, {trials} messages per cell\n");
    let wpms = [12.0, 20.0, 28.0, 36.0];
    print!("| SNR dB |");
    for w in wpms {
        print!(" {w} WPM |");
    }
    println!();
    println!("|---|{}", "---|".repeat(wpms.len()));
    // CWLD_SNRS=30,15 runs only those rows (the full bench takes long on a slow computer).
    let snrs: Vec<f64> = std::env::var("CWLD_SNRS").ok().map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_else(|| vec![30.0, 15.0, 10.0, 6.0, 3.0, 0.0, -3.0, -6.0]);
    for snr in snrs {
        print!("| {snr:+} |");
        for w in wpms {
            let base = CwSpec { wpm: w, snr_db: Some(snr), jitter: 0.05, ..Default::default() };
            let c = run_cell(&base, trials, 100 + w as u64, pcfg);
            print!(" {:.1} |", c.cer * 100.0);
        }
        println!();
    }
    if std::env::args().any(|a| a == "--grid") {
        return;
    }

    println!("\nDifficult signals, 20 WPM, SNR +10 dB unless noted\n");
    println!("| Case | CER % |");
    println!("|---|---|");
    let base = CwSpec { wpm: 20.0, snr_db: Some(10.0), jitter: 0.05, ..Default::default() };
    let cases: Vec<(&str, CwSpec)> = vec![
        ("hand keying, 15% jitter", CwSpec { jitter: 0.15, ..base.clone() }),
        ("sloppy spacing x1.5", CwSpec { gap_scale: 1.5, ..base.clone() }),
        ("short dashes (2.5)", CwSpec { dash_ratio: 2.5, ..base.clone() }),
        ("long dashes (3.6)", CwSpec { dash_ratio: 3.6, ..base.clone() }),
        ("deep QSB (0.8)", CwSpec { qsb_depth: 0.8, qsb_hz: 0.3, ..base.clone() }),
        ("tone drift 20 Hz", CwSpec { drift_hz: 20.0, ..base.clone() }),
        ("mistuned by 25 Hz", CwSpec { tone_hz: 725.0, ..base.clone() }),
        ("second station 60 Hz away, equal level", CwSpec { interferer: Some(Interferer { offset_hz: 60.0, rel_db: 0.0, wpm: 17.0 }), ..base.clone() }),
        ("second station 120 Hz away, +10 dB", CwSpec { interferer: Some(Interferer { offset_hz: 120.0, rel_db: 10.0, wpm: 24.0 }), ..base.clone() }),
        ("impulse noise, 15 clicks/s at +46 dB", CwSpec { impulses_per_s: 15.0, impulse_db: 46.0, ..base.clone() }),
        ("impulse noise, 40 clicks/s at +40 dB, SNR +6", CwSpec { impulses_per_s: 40.0, impulse_db: 40.0, snr_db: Some(6.0), ..base.clone() }),
        ("carrier 50 Hz away, +15 dB (present 3 s before)", CwSpec { carrier: Some((50.0, 15.0)), lead_s: 3.0, ..base.clone() }),
        ("carrier 120 Hz away, +25 dB (present 3 s before)", CwSpec { carrier: Some((120.0, 25.0)), lead_s: 3.0, ..base.clone() }),
        ("carrier 50 Hz away, +15 dB, starts with the signal", CwSpec { carrier: Some((50.0, 15.0)), ..base.clone() }),
        ("deep QSB at +3 dB", CwSpec { qsb_depth: 0.8, qsb_hz: 0.3, snr_db: Some(3.0), ..base.clone() }),
        ("slow 8 WPM at 0 dB", CwSpec { wpm: 8.0, snr_db: Some(0.0), ..base.clone() }),
        ("fast 45 WPM at 15 dB", CwSpec { wpm: 45.0, snr_db: Some(15.0), ..base.clone() }),
    ];
    for (name, spec) in cases {
        let c = run_cell(&spec, trials, 777, pcfg);
        println!("| {name} | {:.1} |", c.cer * 100.0);
    }
    if std::env::args().any(|a| a == "--difficult") {
        return;
    }

    neighbours(trials, pcfg);
    fists_only(trials, pcfg);
}

/// Stations next door: what a steeper filter is for.
fn neighbours(trials: usize, pcfg: impl Fn(&CwSpec) -> PipelineConfig + Copy) {
    println!("\nStations next door, 20 WPM, SNR +10 dB unless noted\n");
    println!("| Case | CER % |");
    println!("|---|---|");
    let base = CwSpec { wpm: 20.0, snr_db: Some(10.0), jitter: 0.05, ..Default::default() };
    let n = |off: f64, db: f64, wpm: f64| Some(Interferer { offset_hz: off, rel_db: db, wpm });
    let cases: Vec<(&str, CwSpec)> = vec![
        ("40 Hz away, equal level", CwSpec { interferer: n(40.0, 0.0, 22.0), ..base.clone() }),
        ("60 Hz away, +10 dB", CwSpec { interferer: n(60.0, 10.0, 26.0), ..base.clone() }),
        ("80 Hz away, +20 dB", CwSpec { interferer: n(-80.0, 20.0, 18.0), ..base.clone() }),
        ("100 Hz away, +20 dB", CwSpec { interferer: n(100.0, 20.0, 30.0), ..base.clone() }),
        ("150 Hz away, +30 dB", CwSpec { interferer: n(-150.0, 30.0, 25.0), ..base.clone() }),
        ("200 Hz away, +40 dB", CwSpec { interferer: n(200.0, 40.0, 28.0), ..base.clone() }),
        ("100 Hz away, +20 dB, 30 WPM wanted at +6 dB", CwSpec { wpm: 30.0, snr_db: Some(6.0), interferer: n(100.0, 20.0, 22.0), ..base.clone() }),
        ("70 Hz away, +15 dB, 12 WPM wanted", CwSpec { wpm: 12.0, interferer: n(70.0, 15.0, 20.0), ..base.clone() }),
    ];
    for (name, spec) in cases {
        let c = run_cell(&spec, trials, 909, pcfg);
        println!("| {name} | {:.1} |", c.cer * 100.0);
    }
}

fn fists_only(trials: usize, pcfg: impl Fn(&CwSpec) -> PipelineConfig + Copy) {
    println!("\nSloppy fists, 20 WPM, SNR +15 dB unless noted\n");
    println!("| Case | CER % |");
    println!("|---|---|");
    let base = CwSpec { wpm: 20.0, snr_db: Some(15.0), jitter: 0.03, ..Default::default() };
    let fists: Vec<(&str, CwSpec)> = vec![
        ("letters far apart (char 6, word 14), like spelling a name", CwSpec { char_gap: 6.0, word_gap: 14.0, gap_jitter: 0.1, ..base.clone() }),
        ("Farnsworth learner (char 9, word 21)", CwSpec { char_gap: 9.0, word_gap: 21.0, gap_jitter: 0.1, ..base.clone() }),
        ("words run together (word gap 4.5)", CwSpec { word_gap: 4.5, gap_jitter: 0.08, ..base.clone() }),
        ("characters run together (char gap 2.2)", CwSpec { char_gap: 2.2, gap_jitter: 0.08, ..base.clone() }),
        ("heavy weighting (gap inside a character 1.8, char 5, word 10)", CwSpec { intra_gap: 1.8, char_gap: 5.0, word_gap: 10.0, gap_jitter: 0.1, ..base.clone() }),
        ("light weighting (gap inside a character 0.6)", CwSpec { intra_gap: 0.6, gap_jitter: 0.1, ..base.clone() }),
        ("uneven gaps, 30 percent spread", CwSpec { gap_jitter: 0.3, ..base.clone() }),
        ("uneven gaps 30 percent and elements 15 percent (straight key)", CwSpec { gap_jitter: 0.3, jitter: 0.15, ..base.clone() }),
        ("bug: long dashes (4.2) and tight dots, char 4", CwSpec { dash_ratio: 4.2, intra_gap: 0.8, char_gap: 4.0, word_gap: 9.0, gap_jitter: 0.15, ..base.clone() }),
        ("speed swings 15 percent", CwSpec { speed_wobble: 0.15, gap_jitter: 0.1, ..base.clone() }),
        ("speed swings 25 percent, uneven gaps", CwSpec { speed_wobble: 0.25, gap_jitter: 0.2, ..base.clone() }),
        ("letters far apart at 30 WPM (char 6, word 13)", CwSpec { wpm: 30.0, char_gap: 6.0, word_gap: 13.0, gap_jitter: 0.1, ..base.clone() }),
        ("heavy weighting at 35 WPM, +6 dB", CwSpec { wpm: 35.0, snr_db: Some(6.0), intra_gap: 1.8, char_gap: 5.0, word_gap: 10.0, gap_jitter: 0.1, ..base.clone() }),
        ("spells slowly (char 8, word 18), then keys normally", CwSpec { char_gap: 8.0, word_gap: 18.0, change_after: 0.34, char_gap2: 3.0, word_gap2: 7.0, gap_jitter: 0.1, ..base.clone() }),
        ("keys normally, then spells slowly (char 8, word 18)", CwSpec { change_after: 0.34, char_gap2: 8.0, word_gap2: 18.0, gap_jitter: 0.1, ..base.clone() }),
        ("wide (char 6, word 13), then tight (char 2.5, word 5.5)", CwSpec { char_gap: 6.0, word_gap: 13.0, change_after: 0.34, char_gap2: 2.5, word_gap2: 5.5, gap_jitter: 0.1, ..base.clone() }),
        ("speed goes from 18 to 28 WPM", CwSpec { wpm: 18.0, change_after: 0.34, wpm2: 28.0, ..base.clone() }),
        ("speed goes from 30 to 18 WPM", CwSpec { wpm: 30.0, change_after: 0.34, wpm2: 18.0, ..base.clone() }),
        ("spells slowly at 25 WPM, then tight and 32 WPM", CwSpec { wpm: 25.0, char_gap: 8.0, word_gap: 18.0, change_after: 0.34, char_gap2: 2.8, word_gap2: 6.0, wpm2: 32.0, gap_jitter: 0.1, ..base.clone() }),
        ("3 in 100 dots and dashes lost in short fades", CwSpec { lost_elements: 0.03, ..base.clone() }),
        ("6 in 100 dots and dashes lost, +6 dB", CwSpec { lost_elements: 0.06, snr_db: Some(6.0), ..base.clone() }),
        ("uneven gaps 30 percent at +3 dB", CwSpec { gap_jitter: 0.3, snr_db: Some(3.0), ..base.clone() }),
    ];
    for (name, spec) in fists {
        let c = run_cell(&spec, trials, 4242, pcfg);
        println!("| {name} | {:.1} |", c.cer * 100.0);
    }
}
