//! A signal away from the decoder's pitch: how much of the start is lost when the decoder has
//! to find it first? Usage: seekbench
use cwld_decoder::bench::run_cell;
use cwld_decoder::PipelineConfig;
use cwld_dsp::CwSpec;

fn main() {
    for (tone, wpm) in [(615.0, 20.0), (520.0, 28.0), (850.0, 16.0)] {
        let mut row = String::new();
        for snr in [15.0, 6.0, 3.0] {
            let base = CwSpec { wpm, snr_db: Some(snr), jitter: 0.05, tone_hz: tone, lead_s: 1.0, ..Default::default() };
            let c = run_cell(&base, 10, 4711, |_| {
                let mut p = PipelineConfig::new(8000, 700.0);
                p.auto_tone = true;
                p.min_separation = 3.5;
                if std::env::var("NOWEAK").is_ok() { p.weak_marks = Some((0.0, 1.0, 4.0)); }
                if std::env::var("NOSPACE").is_ok() { p.spacing = false; }
                p
            });
            row += &format!(" {:5.1}", c.cer * 100.0);
        }
        println!("signal at {tone} Hz, {wpm} WPM, decoder starts at 700 Hz, CER % at +15/+6/+3 dB:{row}");
    }
}
