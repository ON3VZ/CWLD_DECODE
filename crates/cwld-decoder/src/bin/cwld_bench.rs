//! `cargo run --release -p cwld-decoder --bin cwld_bench` prints the character error rate
//! for a grid of signal to noise ratios and speeds, and for some difficult signals.

use cwld_decoder::bench::run_cell;
use cwld_decoder::PipelineConfig;
use cwld_dsp::gen::Interferer;
use cwld_dsp::CwSpec;

fn main() {
    let quick = std::env::args().any(|a| a == "--quick");
    let trials = if quick { 4 } else { 12 };
    let pcfg = |s: &CwSpec| PipelineConfig::new(s.sample_rate, 700.0);

    println!("Character error rate (%), SNR in 500 Hz, {trials} messages per cell\n");
    let wpms = [12.0, 20.0, 28.0, 36.0];
    print!("| SNR dB |");
    for w in wpms {
        print!(" {w} WPM |");
    }
    println!();
    println!("|---|{}", "---|".repeat(wpms.len()));
    for snr in [30.0, 15.0, 10.0, 6.0, 3.0, 0.0, -3.0, -6.0] {
        print!("| {snr:+} |");
        for w in wpms {
            let base = CwSpec { wpm: w, snr_db: Some(snr), jitter: 0.05, ..Default::default() };
            let c = run_cell(&base, trials, 100 + w as u64, pcfg);
            print!(" {:.1} |", c.cer * 100.0);
        }
        println!();
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
        ("slow 8 WPM at 0 dB", CwSpec { wpm: 8.0, snr_db: Some(0.0), ..base.clone() }),
        ("fast 45 WPM at 15 dB", CwSpec { wpm: 45.0, snr_db: Some(15.0), ..base.clone() }),
    ];
    for (name, spec) in cases {
        let c = run_cell(&spec, trials, 777, pcfg);
        println!("| {name} | {:.1} |", c.cer * 100.0);
    }
}
