//! Listening again: the last seconds of audio (see `Engine::rewind`) decoded a second time,
//! at the same pitch or another one, and the last minute saved as a WAV file.
//!
//! The decoder that runs live is not touched. A fresh decoder with the same settings reads the
//! stretch twice: the first pass only lets the filter find the levels and the exact pitch, the
//! second one reads, so the first characters come out with the right levels too.

use crate::engine::{Rewind, DECODE_RATE};
use cwld_decoder::Pipeline;
use cwld_dsp::seek::ToneSeeker;
use serde::Serialize;
use std::path::Path;

/// Shortest and longest stretch that can be decoded again, in seconds.
pub const REPLAY_MIN_S: f64 = 5.0;
pub const REPLAY_MAX_S: f64 = 60.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReplayTone {
    /// The pitch the live decoder listens to now, with its own choice of following the strongest.
    Current,
    /// This pitch, without moving away from it.
    Fixed(f64),
    /// The strongest keyed tone in the stretch.
    Strongest,
}

/// A keyed tone heard in the stretch: its pitch and how far it stood out of the noise at best.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct HeardTone {
    pub hz: f64,
    pub score_db: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ReplayResult {
    pub text: String,
    /// One confidence (0 to 1) per character of `text`.
    pub conf: Vec<f32>,
    /// The pitch that was read (after the AFC).
    pub tone_hz: f64,
    /// Length of the stretch actually decoded (less than asked when the buffer is shorter).
    pub seconds: f64,
    pub wpm: f32,
    /// Keyed tones in the stretch, strongest first, to choose another pitch from.
    pub tones: Vec<HeardTone>,
}

/// The keyed tones in a stretch of audio at `DECODE_RATE`: up to `max` of them, at least 40 Hz
/// apart, strongest first, each at the best score it reached. Weaker than 10 dB above the noise
/// is left out (noise alone scores up to about 8 dB, see `seek`).
pub fn heard_tones(samples: &[f32], max: usize) -> Vec<HeardTone> {
    let mut sk = ToneSeeker::new(DECODE_RATE, 300.0, 1200.0);
    // Best score per 10 Hz.
    let mut best: std::collections::BTreeMap<i64, (f64, f64)> = std::collections::BTreeMap::new();
    for chunk in samples.chunks(400) {
        if sk.push(chunk) {
            for c in sk.peaks(10.0) {
                let key = (c.hz / 10.0).round() as i64;
                let e = best.entry(key).or_insert((c.hz, c.score_db));
                if c.score_db > e.1 {
                    *e = (c.hz, c.score_db);
                }
            }
        }
    }
    let mut all: Vec<HeardTone> = best.values().filter(|(_, s)| *s >= 10.0).map(|&(hz, score_db)| HeardTone { hz, score_db }).collect();
    all.sort_by(|a, b| b.score_db.partial_cmp(&a.score_db).unwrap());
    let mut out: Vec<HeardTone> = Vec::new();
    for t in all {
        if out.len() >= max {
            break;
        }
        if out.iter().all(|o| (o.hz - t.hz).abs() >= 40.0) {
            out.push(t);
        }
    }
    out
}

/// The last `seconds` of the rewind buffer.
pub fn tail(r: &Rewind, seconds: f64) -> &[f32] {
    let n = ((seconds.clamp(REPLAY_MIN_S, REPLAY_MAX_S)) * DECODE_RATE as f64) as usize;
    &r.samples[r.samples.len().saturating_sub(n)..]
}

/// Decodes the last `seconds` again with the live decoder's settings.
pub fn replay(r: &Rewind, seconds: f64, tone: ReplayTone) -> ReplayResult {
    let raw = tail(r, seconds);
    let audio: Vec<f32> = raw.iter().map(|s| s * r.gain).collect();
    let tones = heard_tones(&audio, 4);
    let mut cfg = r.config.clone();
    cfg.sample_rate = DECODE_RATE;
    cfg.keying = false;
    match tone {
        ReplayTone::Current => cfg.tone_hz = r.tone_hz,
        ReplayTone::Fixed(hz) => {
            cfg.tone_hz = hz.clamp(150.0, 2500.0);
            cfg.auto_tone = false;
        }
        ReplayTone::Strongest => {
            cfg.tone_hz = tones.first().map_or(r.tone_hz, |t| t.hz);
            cfg.auto_tone = false;
        }
    }
    let mut p = Pipeline::new(cfg);
    let words: Vec<&str> = r.words.iter().map(|s| s.as_str()).collect();
    p.add_words(&words);
    let hints: Vec<&str> = r.hints.iter().map(|s| s.as_str()).collect();
    p.set_hints(&hints);
    let chunk = (DECODE_RATE / 40) as usize;
    // First pass: levels and pitch only.
    for c in audio.chunks(chunk) {
        p.push(c);
        let _ = p.poll();
    }
    p.restart_text();
    let (mut text, mut conf, mut wpm) = (String::new(), Vec::new(), 0.0f32);
    for c in audio.chunks(chunk) {
        p.push(c);
        let up = p.poll();
        text.push_str(&up.committed);
        conf.extend(up.committed_conf);
        if up.wpm > 0.0 {
            wpm = up.wpm;
        }
    }
    p.flush();
    let up = p.poll();
    text.push_str(&up.committed);
    conf.extend(up.committed_conf);
    // Leading and trailing spaces carry no information.
    let start = text.len() - text.trim_start().len();
    let trimmed = text.trim().to_string();
    let conf: Vec<f32> = conf.into_iter().skip(start).take(trimmed.chars().count()).collect();
    ReplayResult { text: trimmed, conf, tone_hz: p.tone_hz(), seconds: raw.len() as f64 / DECODE_RATE as f64, wpm, tones }
}

/// Writes the whole rewind buffer (up to a minute, before the gain) as a 16-bit mono WAV file at
/// 8 kHz. Returns its length in seconds.
pub fn save_wav(r: &Rewind, path: &Path) -> std::io::Result<f64> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let f = std::io::BufWriter::new(std::fs::File::create(path)?);
    cwld_dsp::wav::write_mono16(f, DECODE_RATE, &r.samples)?;
    Ok(r.samples.len() as f64 / DECODE_RATE as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cwld_decoder::PipelineConfig;
    use cwld_dsp::{generate, CwSpec};

    fn rewind_of(samples: Vec<f32>, tone: f64) -> Rewind {
        let mut config = PipelineConfig::new(DECODE_RATE, tone);
        config.auto_tone = true;
        config.min_separation = 3.5;
        Rewind { samples, gain: 1.0, tone_hz: tone, config, words: Vec::new(), hints: Vec::new() }
    }

    /// Two stations at once: 600 Hz and 900 Hz, the second one 6 dB weaker.
    fn two_stations() -> (Vec<f32>, String, String) {
        let a = generate(&CwSpec { text: "CQ CQ DE ON3VZ ON3VZ K".into(), tone_hz: 600.0, snr_db: Some(15.0), lead_s: 0.3, trail_s: 2.0, seed: 3, ..Default::default() });
        let b = generate(&CwSpec { text: "QRZ DE DL1ABC DL1ABC K".into(), tone_hz: 900.0, wpm: 24.0, snr_db: None, lead_s: 0.6, trail_s: 2.0, seed: 4, ..Default::default() });
        let n = a.samples.len().max(b.samples.len());
        let mut mix = vec![0.0f32; n];
        for (i, v) in a.samples.iter().enumerate() {
            mix[i] += v;
        }
        for (i, v) in b.samples.iter().enumerate() {
            mix[i] += v * 0.5;
        }
        (mix, a.truth, b.truth)
    }

    #[test]
    fn the_same_stretch_reads_again_from_its_first_character() {
        let (mix, a, _) = two_stations();
        let r = replay(&rewind_of(mix, 600.0), 30.0, ReplayTone::Fixed(600.0));
        assert_eq!(r.text, a);
        assert_eq!(r.conf.len(), r.text.chars().count());
        assert!((r.tone_hz - 600.0).abs() < 15.0);
    }

    #[test]
    fn another_pitch_reads_the_other_station() {
        let (mix, _, b) = two_stations();
        let r = replay(&rewind_of(mix, 600.0), 30.0, ReplayTone::Fixed(900.0));
        // The 600 Hz station starts first and may leave a single E in front.
        assert!(cwld_decoder::bench::levenshtein(&r.text, &b) <= 2, "{}", r.text);
        assert!(r.text.contains("DL1ABC DL1ABC"), "{}", r.text);
    }

    #[test]
    fn both_stations_are_offered_as_pitches() {
        let (mix, _, _) = two_stations();
        let r = replay(&rewind_of(mix, 600.0), 30.0, ReplayTone::Current);
        let hz: Vec<f64> = r.tones.iter().map(|t| t.hz).collect();
        assert!(hz.iter().any(|h| (h - 600.0).abs() < 15.0), "{hz:?}");
        assert!(hz.iter().any(|h| (h - 900.0).abs() < 15.0), "{hz:?}");
        assert!((r.tones[0].hz - 600.0).abs() < 15.0, "strongest first: {hz:?}");
    }

    #[test]
    fn strongest_finds_the_station_when_the_decoder_was_elsewhere() {
        let (mix, a, _) = two_stations();
        let mut rw = rewind_of(mix, 1100.0);
        rw.config.auto_tone = false;
        let r = replay(&rw, 30.0, ReplayTone::Strongest);
        assert_eq!(r.text, a);
    }

    #[test]
    fn only_the_last_seconds_are_read() {
        let first = generate(&CwSpec { text: "TEST TEST".into(), snr_db: Some(20.0), lead_s: 0.5, trail_s: 0.5, ..Default::default() });
        let second = generate(&CwSpec { text: "RST 599 599".into(), snr_db: Some(20.0), lead_s: 0.5, trail_s: 2.0, seed: 2, ..Default::default() });
        let secs = (second.samples.len() as f64 / 8000.0).ceil();
        let mut s = first.samples.clone();
        s.extend(generate(&CwSpec { text: "".into(), snr_db: Some(20.0), lead_s: 15.0, trail_s: 0.0, seed: 9, ..Default::default() }).samples);
        s.extend(second.samples);
        let r = replay(&rewind_of(s, 700.0), secs, ReplayTone::Fixed(700.0));
        assert_eq!(r.text, "RST 599 599");
        assert!((r.seconds - secs).abs() < 0.01);
    }

    #[test]
    fn noise_only_gives_no_text_and_no_pitches() {
        let g = generate(&CwSpec { text: "".into(), snr_db: Some(10.0), lead_s: 20.0, trail_s: 0.0, ..Default::default() });
        let r = replay(&rewind_of(g.samples, 700.0), 20.0, ReplayTone::Current);
        assert_eq!(r.text, "");
        assert!(r.tones.is_empty(), "{:?}", r.tones);
    }

    #[test]
    fn the_wav_holds_the_buffer() {
        let g = generate(&CwSpec { text: "TEST".into(), snr_db: Some(20.0), ..Default::default() });
        let rw = rewind_of(g.samples.clone(), 700.0);
        let path = std::env::temp_dir().join(format!("cwld-rewind-{}.wav", std::process::id()));
        let secs = save_wav(&rw, &path).unwrap();
        let (rate, back) = cwld_dsp::wav::read(std::fs::File::open(&path).unwrap()).unwrap();
        assert_eq!(rate, 8000);
        assert_eq!(back.len(), g.samples.len());
        assert!((secs - g.samples.len() as f64 / 8000.0).abs() < 1e-9);
        let _ = std::fs::remove_file(path);
    }
}
