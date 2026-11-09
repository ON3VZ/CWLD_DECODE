use cwld_dsp::frontend::{Frontend, FrontendConfig};
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (sr, s) = cwld_dsp::wav::read(std::fs::File::open(&a[0]).unwrap()).unwrap();
    let tone: f64 = a[1].parse().unwrap();
    let mut f = Frontend::new(FrontendConfig::new(sr, tone));
    let mut line = String::new();
    for (i, c) in s.chunks(sr as usize).enumerate() {
        f.push(c);
        let sep = f.levels().map(|l| l.separation).unwrap_or(0.0);
        line.push_str(&format!("{}s:{:.1}{} ", i, sep, if f.signal_present() { "*" } else { "" }));
    }
    println!("{line}");
}
