//! The audio thread: reads a source, resamples to 8 kHz, feeds the waterfall, the recorder and
//! the decoder, and reports everything as events. Commands go in through a channel.

use crate::record::Recorder;
use crate::resample::Resampler;
#[cfg(feature = "device")]
use crate::ring::{ring, Consumer};
use crate::waterfall::Waterfall;
use cwld_decoder::{Pipeline, PipelineConfig};
use serde::Serialize;
use std::path::PathBuf;
#[cfg(feature = "device")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, TryRecvError};
#[cfg(feature = "device")]
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub const DECODE_RATE: u32 = 8000;

#[derive(Clone, Debug)]
pub enum Source {
    /// A sound card input. `None` or empty: a radio-looking input, else the default input.
    Device { name: Option<String> },
    /// A WAV file, played at real speed (`realtime`) or as fast as possible.
    File { path: PathBuf, realtime: bool },
    /// Samples in memory (tests, practice mode).
    Samples { rate: u32, data: Vec<f32>, realtime: bool },
}

#[derive(Clone, Debug)]
pub struct EngineConfig {
    pub tone_hz: f64,
    /// Words the decoder's word model should know: own callsigns, name, QTH.
    pub words: Vec<String>,
    pub waterfall_lo_hz: f64,
    pub waterfall_hi_hz: f64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig { tone_hz: 700.0, words: Vec::new(), waterfall_lo_hz: 200.0, waterfall_hi_hz: 1400.0 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TentativeOut {
    pub best: char,
    pub confidence: f32,
    pub alts: Vec<(char, f32)>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EngineEvent {
    Started { source: String, rate: u32 },
    /// New final text with a confidence per character, and the not yet final tail.
    Decode { text: String, conf: Vec<f32>, tentative: Vec<TentativeOut>, wpm: f32 },
    Waterfall { bins: Vec<u8>, lo_hz: f64, hz_per_bin: f64 },
    /// Input level in dB below full scale, about ten times a second.
    Level { dbfs: f32 },
    /// Mean confidence of the last characters, and the signal to noise ratio in the filter.
    Stats { confidence: Option<f32>, snr_db: Option<f32> },
    Tone { hz: f64 },
    Recording { active: bool, seconds: f64, path: String },
    /// Samples were lost because the audio thread could not keep up.
    Overrun { samples: u64 },
    /// The source ended (file) or failed (`reason`).
    Stopped { reason: Option<String> },
}

enum Command {
    Retune(f64),
    StartRecording(PathBuf),
    StopRecording,
    AddWords(Vec<String>),
    Stop,
}

pub struct Engine {
    cmd: Sender<Command>,
    events: Receiver<EngineEvent>,
    thread: Option<JoinHandle<()>>,
}

impl Engine {
    /// Starts the audio thread. Waits until the source is open, so an error (no device) comes
    /// back here instead of as a late event.
    pub fn start(source: Source, cfg: EngineConfig) -> Result<Engine, String> {
        let (cmd_tx, cmd_rx) = channel();
        let (ev_tx, ev_rx) = channel();
        let (ready_tx, ready_rx) = sync_channel::<Result<(), String>>(1);
        let thread = std::thread::Builder::new()
            .name("cwld-audio".into())
            .spawn(move || run(source, cfg, cmd_rx, ev_tx, ready_tx))
            .map_err(|e| e.to_string())?;
        match ready_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => Ok(Engine { cmd: cmd_tx, events: ev_rx, thread: Some(thread) }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => Err("audio start timed out".into()),
        }
    }

    /// Everything that happened since the last call.
    pub fn drain(&self) -> Vec<EngineEvent> {
        let mut out = Vec::new();
        while let Ok(e) = self.events.try_recv() {
            out.push(e);
        }
        out
    }

    /// Blocks up to `timeout` for the next event (tests).
    pub fn next_event(&self, timeout: Duration) -> Option<EngineEvent> {
        self.events.recv_timeout(timeout).ok()
    }

    pub fn retune(&self, hz: f64) {
        let _ = self.cmd.send(Command::Retune(hz));
    }

    pub fn start_recording(&self, path: PathBuf) {
        let _ = self.cmd.send(Command::StartRecording(path));
    }

    pub fn stop_recording(&self) {
        let _ = self.cmd.send(Command::StopRecording);
    }

    pub fn add_words(&self, words: Vec<String>) {
        let _ = self.cmd.send(Command::AddWords(words));
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.cmd.send(Command::Stop);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// Where samples come from inside the thread.
enum Input {
    #[cfg(feature = "device")]
    Ring { cons: Consumer, failed: Arc<AtomicBool>, reported_drops: u64, _stream: cpal::Stream },
    Memory { data: Vec<f32>, pos: usize, realtime: bool, rate: u32, started: Instant },
}

fn open(source: Source) -> Result<(Input, u32, String), String> {
    match source {
        Source::Samples { rate, data, realtime } => Ok((Input::Memory { data, pos: 0, realtime, rate, started: Instant::now() }, rate, "samples".into())),
        Source::File { path, realtime } => {
            let f = std::fs::File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let (rate, data) = cwld_dsp::wav::read(f).map_err(|e| format!("{}: {e}", path.display()))?;
            Ok((Input::Memory { data, pos: 0, realtime, rate, started: Instant::now() }, rate, path.display().to_string()))
        }
        #[cfg(feature = "device")]
        Source::Device { name } => {
            // Two seconds of buffer at 48 kHz: plenty for a thread that wakes every 20 ms.
            let (prod, cons) = ring(96_000 * 2);
            let failed = Arc::new(AtomicBool::new(false));
            let (stream, rate, dev) = crate::device::open::open(name.as_deref(), prod, failed.clone())?;
            Ok((Input::Ring { cons, failed, reported_drops: 0, _stream: stream }, rate, dev))
        }
        #[cfg(not(feature = "device"))]
        Source::Device { .. } => Err("this build has no sound card support".into()),
    }
}

struct Worker {
    ev: Sender<EngineEvent>,
    rate: u32,
    resampler: Resampler,
    pipe: Pipeline,
    wf: Waterfall,
    rec: Option<Recorder>,
    level_acc: f64,
    level_n: usize,
    conf_window: Vec<f32>,
    last_tentative: String,
    last_stats: Instant,
    last_rec_report: Instant,
    scratch: Vec<f32>,
}

impl Worker {
    fn send(&self, e: EngineEvent) {
        let _ = self.ev.send(e);
    }

    fn process(&mut self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        if let Some(r) = self.rec.as_mut() {
            match r.write(samples) {
                Ok(true) => {}
                Ok(false) => self.stop_recording(),
                Err(e) => {
                    self.send(EngineEvent::Stopped { reason: Some(format!("recording: {e}")) });
                    self.rec = None;
                }
            }
        }
        for s in samples {
            self.level_acc += (*s as f64).powi(2);
            self.level_n += 1;
        }
        if self.level_n >= (self.rate / 10) as usize {
            let rms = (self.level_acc / self.level_n as f64).sqrt();
            self.send(EngineEvent::Level { dbfs: (20.0 * (rms + 1e-9).log10()) as f32 });
            self.level_acc = 0.0;
            self.level_n = 0;
        }
        self.scratch.clear();
        let mut lo = std::mem::take(&mut self.scratch);
        self.resampler.process(samples, &mut lo);
        for row in self.wf.push(&lo) {
            self.send(EngineEvent::Waterfall { bins: row.bins, lo_hz: row.lo_hz, hz_per_bin: row.hz_per_bin });
        }
        self.pipe.push(&lo);
        self.scratch = lo;
        self.poll_decoder();
    }

    fn poll_decoder(&mut self) {
        let up = self.pipe.poll();
        let tentative: Vec<TentativeOut> = up.tentative.iter().map(|t| TentativeOut { best: t.best, confidence: t.confidence, alts: t.alts.clone() }).collect();
        let tent_text: String = tentative.iter().map(|t| t.best).collect();
        if !up.committed.is_empty() || tent_text != self.last_tentative {
            for (c, p) in up.committed.chars().zip(up.committed_conf.iter()) {
                if c != ' ' {
                    self.conf_window.push(*p);
                }
            }
            let excess = self.conf_window.len().saturating_sub(cwld_types::CONFIDENCE_WINDOW);
            self.conf_window.drain(..excess);
            self.last_tentative = tent_text;
            self.send(EngineEvent::Decode { text: up.committed, conf: up.committed_conf, tentative, wpm: up.wpm });
        }
        if self.last_stats.elapsed() >= Duration::from_millis(500) {
            self.send_stats();
        }
        if let Some(r) = &self.rec {
            if self.last_rec_report.elapsed() >= Duration::from_secs(1) {
                self.last_rec_report = Instant::now();
                self.send(EngineEvent::Recording { active: true, seconds: r.seconds(), path: r.path().display().to_string() });
            }
        }
    }

    fn send_stats(&mut self) {
        self.last_stats = Instant::now();
        self.send(EngineEvent::Stats { confidence: cwld_types::mean_confidence(&self.conf_window), snr_db: self.pipe.snr_db() });
    }

    fn stop_recording(&mut self) {
        if let Some(r) = self.rec.take() {
            let secs = r.seconds();
            match r.finish() {
                Ok(p) => self.send(EngineEvent::Recording { active: false, seconds: secs, path: p.display().to_string() }),
                Err(e) => self.send(EngineEvent::Stopped { reason: Some(format!("recording: {e}")) }),
            }
        }
    }

    fn command(&mut self, c: Command) -> bool {
        match c {
            Command::Retune(hz) => {
                let hz = hz.clamp(150.0, 2500.0);
                self.pipe.retune(hz);
                self.send(EngineEvent::Tone { hz });
            }
            Command::StartRecording(path) => {
                self.stop_recording();
                match Recorder::create(&path, self.rate) {
                    Ok(r) => {
                        self.send(EngineEvent::Recording { active: true, seconds: 0.0, path: r.path().display().to_string() });
                        self.rec = Some(r);
                    }
                    Err(e) => self.send(EngineEvent::Stopped { reason: Some(format!("recording: {e}")) }),
                }
            }
            Command::StopRecording => self.stop_recording(),
            Command::AddWords(w) => {
                let refs: Vec<&str> = w.iter().map(|s| s.as_str()).collect();
                self.pipe.add_words(&refs);
            }
            Command::Stop => return false,
        }
        true
    }
}

fn run(source: Source, cfg: EngineConfig, cmds: Receiver<Command>, ev: Sender<EngineEvent>, ready: std::sync::mpsc::SyncSender<Result<(), String>>) {
    let (mut input, rate, name) = match open(source) {
        Ok(x) => x,
        Err(e) => {
            let _ = ready.send(Err(e));
            return;
        }
    };
    let _ = ready.send(Ok(()));
    let mut pcfg = PipelineConfig::new(DECODE_RATE, cfg.tone_hz);
    pcfg.auto_tone = false;
    let mut pipe = Pipeline::new(pcfg);
    let refs: Vec<&str> = cfg.words.iter().map(|s| s.as_str()).collect();
    pipe.add_words(&refs);
    let mut w = Worker {
        ev,
        rate,
        resampler: Resampler::new(rate, DECODE_RATE),
        pipe,
        wf: Waterfall::new(cfg.waterfall_lo_hz, cfg.waterfall_hi_hz),
        rec: None,
        level_acc: 0.0,
        level_n: 0,
        conf_window: Vec::new(),
        last_tentative: String::new(),
        last_stats: Instant::now(),
        last_rec_report: Instant::now(),
        scratch: Vec::new(),
    };
    w.send(EngineEvent::Started { source: name, rate });
    w.send(EngineEvent::Tone { hz: cfg.tone_hz });
    let mut buf: Vec<f32> = Vec::new();
    loop {
        loop {
            match cmds.try_recv() {
                Ok(c) => {
                    if !w.command(c) {
                        w.stop_recording();
                        return;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    w.stop_recording();
                    return;
                }
            }
        }
        buf.clear();
        match &mut input {
            #[cfg(feature = "device")]
            Input::Ring { cons, failed, reported_drops, .. } => {
                cons.pop_all(&mut buf);
                let d = cons.dropped();
                if d > *reported_drops {
                    w.send(EngineEvent::Overrun { samples: d - *reported_drops });
                    *reported_drops = d;
                }
                if failed.load(Ordering::Relaxed) {
                    w.stop_recording();
                    w.send(EngineEvent::Stopped { reason: Some("the audio input stopped (unplugged?)".into()) });
                    return;
                }
                w.process(&buf);
                std::thread::sleep(Duration::from_millis(20));
            }
            Input::Memory { data, pos, realtime, rate, started } => {
                let end = if *realtime {
                    // As many samples as real time allows.
                    ((started.elapsed().as_secs_f64() * *rate as f64) as usize).min(data.len())
                } else {
                    (*pos + 4096).min(data.len())
                };
                if end > *pos {
                    buf.extend_from_slice(&data[*pos..end]);
                    *pos = end;
                }
                let done = *pos >= data.len();
                w.process(&buf);
                if done {
                    w.pipe.flush();
                    w.poll_decoder();
                    w.send_stats();
                    w.stop_recording();
                    w.send(EngineEvent::Stopped { reason: None });
                    // Stay alive for commands until the owner drops the engine.
                    loop {
                        match cmds.recv() {
                            Ok(Command::Stop) | Err(_) => return,
                            Ok(_) => {}
                        }
                    }
                }
                if *realtime {
                    std::thread::sleep(Duration::from_millis(20));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cwld_dsp::{generate, CwSpec};

    fn collect_until_stopped(e: &Engine) -> Vec<EngineEvent> {
        let mut out = Vec::new();
        while let Some(ev) = e.next_event(Duration::from_secs(20)) {
            let stop = matches!(ev, EngineEvent::Stopped { .. });
            out.push(ev);
            if stop {
                break;
            }
        }
        out
    }

    fn text_of(events: &[EngineEvent]) -> String {
        events.iter().filter_map(|e| if let EngineEvent::Decode { text, .. } = e { Some(text.as_str()) } else { None }).collect::<String>().trim().to_string()
    }

    fn upsample(x: &[f32], factor: usize) -> Vec<f32> {
        // Simple sample-and-hold is fine here: the resampler filters the images away.
        x.iter().flat_map(|v| std::iter::repeat(*v).take(factor)).collect()
    }

    #[test]
    fn decodes_48khz_audio_end_to_end() {
        let g = generate(&CwSpec { text: "CQ CQ DE ON3VZ K".into(), wpm: 20.0, snr_db: Some(15.0), sample_rate: 48_000, ..Default::default() });
        let e = Engine::start(Source::Samples { rate: 48_000, data: g.samples, realtime: false }, EngineConfig::default()).unwrap();
        let ev = collect_until_stopped(&e);
        assert!(matches!(ev.first(), Some(EngineEvent::Started { rate: 48_000, .. })));
        assert_eq!(text_of(&ev), g.truth);
        assert!(ev.iter().any(|e| matches!(e, EngineEvent::Waterfall { .. })));
        assert!(ev.iter().any(|e| matches!(e, EngineEvent::Level { .. })));
        assert!(matches!(ev.last(), Some(EngineEvent::Stopped { reason: None })));
    }

    #[test]
    fn retune_moves_the_decoder_to_another_tone() {
        let g = generate(&CwSpec { text: "TEST TEST DE ON3VZ".into(), tone_hz: 550.0, snr_db: Some(15.0), lead_s: 1.0, ..Default::default() });
        let data = upsample(&g.samples, 6);
        let e = Engine::start(Source::Samples { rate: 48_000, data, realtime: false }, EngineConfig::default()).unwrap();
        e.retune(550.0);
        let ev = collect_until_stopped(&e);
        assert!(ev.iter().any(|x| *x == EngineEvent::Tone { hz: 550.0 }));
        assert_eq!(text_of(&ev), g.truth);
    }

    #[test]
    fn records_while_decoding() {
        let g = generate(&CwSpec { text: "QRZ DE F5XYZ".into(), snr_db: Some(20.0), ..Default::default() });
        let path = std::env::temp_dir().join(format!("cwld-engine-{}.wav", std::process::id()));
        let n = g.samples.len();
        let e = Engine::start(Source::Samples { rate: 8000, data: g.samples, realtime: false }, EngineConfig::default()).unwrap();
        e.start_recording(path.clone());
        let ev = collect_until_stopped(&e);
        assert!(ev.iter().any(|x| matches!(x, EngineEvent::Recording { active: false, .. })));
        let (rate, back) = cwld_dsp::wav::read(std::fs::File::open(&path).unwrap()).unwrap();
        assert_eq!(rate, 8000);
        // The recording starts when the command arrives, a few chunks may already be past.
        assert!(back.len() > n / 2 && back.len() <= n, "{} of {n}", back.len());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn a_missing_file_is_an_error_at_start() {
        let r = Engine::start(Source::File { path: "/nope/missing.wav".into(), realtime: false }, EngineConfig::default());
        assert!(r.is_err());
    }

    #[test]
    fn own_words_are_known_to_the_word_model() {
        let cfg = EngineConfig { words: vec!["HOBOKEN".into()], ..Default::default() };
        let g = generate(&CwSpec { text: "QTH HOBOKEN".into(), snr_db: Some(20.0), ..Default::default() });
        let e = Engine::start(Source::Samples { rate: 8000, data: g.samples, realtime: false }, cfg).unwrap();
        assert_eq!(text_of(&collect_until_stopped(&e)), "QTH HOBOKEN");
    }
}
