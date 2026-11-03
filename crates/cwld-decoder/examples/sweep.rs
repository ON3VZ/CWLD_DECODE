use cwld_decoder::bench::run_cell;
use cwld_decoder::PipelineConfig;
use cwld_dsp::CwSpec;
fn main() {
    let snrs = [0.0, -3.0, -6.0];
    println!("mark_penalty | CER% at SNR {:?} (20 WPM, 6 msgs)", snrs);
    for pen in [0.0, 1.0, 2.0, 3.5] {
        let mut row = String::new();
        for snr in snrs {
            let base = CwSpec { wpm: 20.0, snr_db: Some(snr), jitter: 0.05, ..Default::default() };
            let c = run_cell(&base, 6, 4242, |s| {
                let mut p = PipelineConfig::new(s.sample_rate, 700.0);
                p.decoder.mark_penalty = pen;
                p
            });
            row += &format!(" {:6.1}", c.cer * 100.0);
        }
        println!("{pen:>4} |{row}");
    }
}
