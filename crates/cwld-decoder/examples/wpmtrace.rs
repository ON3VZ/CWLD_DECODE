use cwld_decoder::pipeline::Pipeline;
use cwld_decoder::PipelineConfig;
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let (sr, s) = cwld_dsp::wav::read(std::fs::File::open(&a[0]).unwrap()).unwrap();
    let tone: f64 = a[1].parse().unwrap();
    let mut p = Pipeline::new(PipelineConfig::new(sr, tone));
    let chunk = (sr as f64 * 0.025) as usize;
    let mut t = 0.0;
    let mut line = String::new();
    for c in s.chunks(chunk) {
        p.push(c);
        t += c.len() as f64 / sr as f64;
        let u = p.poll();
        line.push_str(&u.committed);
        if (t * 2.0).floor() != ((t - 0.025) * 2.0).floor() {
            println!("{:5.1}s wpm {:5.1} locked {} | {}", t, u.wpm, u.locked, line);
            line.clear();
        }
    }
}
