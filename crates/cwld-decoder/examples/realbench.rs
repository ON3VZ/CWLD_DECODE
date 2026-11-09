//! Real recordings from the IC-7300 MK II, decoded the way the app does it (start at 700 Hz).
//! Usage: realbench <dir with the 8 kHz WAV files> <squelch> [<squelch> ...]
use cwld_decoder::bench::{decode_samples, run_cell};
use cwld_decoder::PipelineConfig;
use cwld_dsp::CwSpec;

fn wav(p: &str) -> Vec<f32> {
    cwld_dsp::wav::read(std::fs::File::open(p).unwrap()).unwrap().1
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let dir = &args[0];
    let noise = wav(&format!("{dir}/cwld-20261104-134601-7047.2kHz_8k.wav"));
    let f2 = wav(&format!("{dir}/cwld-20261104-134804-7035.4kHz_8k.wav"));
    let f3 = wav(&format!("{dir}/cwld-20261104-134852-7012.0kHz_8k.wav"));
    for ms in args[1..].iter().map(|x| x.parse::<f32>().unwrap()) {
        let cfg = |tone: f64| {
            let mut p = PipelineConfig::new(8000, tone);
            p.min_separation = ms;
            p.auto_tone = true;
            p
        };
        let n = decode_samples(&noise, cfg(700.0));
        println!("sep {ms}: noise -> {} chars: {n}", n.replace(' ', "").len());
        println!("   f2: {}", decode_samples(&f2, cfg(700.0)));
        println!("   f3: {}", decode_samples(&f3, cfg(700.0)));
        let mut row = String::new();
        for snr in [3.0, 0.0, -3.0] {
            let base = CwSpec { wpm: 20.0, snr_db: Some(snr), jitter: 0.05, tone_hz: 615.0, ..Default::default() };
            let c = run_cell(&base, 6, 777, |_| cfg(700.0));
            row += &format!(" {:5.1}", c.cer * 100.0);
        }
        println!("   synthetic at 615 Hz, start 700, CER% at +3/0/-3 dB:{row}");
    }
}
