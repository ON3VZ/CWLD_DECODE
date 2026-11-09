// Dumps waterfall rows of an 8 kHz WAV as text (one row per line), for checking the picture.
use cwld_audio::waterfall::Waterfall;
fn main() {
    let p = std::env::args().nth(1).unwrap();
    let (_, s) = cwld_dsp::wav::read(std::fs::File::open(p).unwrap()).unwrap();
    let mut wf = Waterfall::new(200.0, 1400.0);
    for r in wf.push(&s) {
        println!("{}", r.bins.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(" "));
    }
}
