//! Decodes a WAV recording exactly the way the app does: through the audio engine, with its
//! resampling to the decoder rate, the automatic tone search and the default settings.
//!
//! Usage: filedec <file.wav> [auto|<tone hz>]
//! Prints the pitches the decoder moved to, a hole in the audio if one was seen, and the text.

use cwld_audio::{Engine, EngineConfig, EngineEvent, Source};
use std::time::Duration;

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let mut cfg = EngineConfig::default();
    match a.get(1).map(|s| s.as_str()) {
        None | Some("auto") => cfg.auto_tone = true,
        Some(t) => {
            cfg.auto_tone = false;
            cfg.tone_hz = t.parse().expect("tone in Hz");
        }
    }
    let e = Engine::start(Source::File { path: a[0].clone().into(), realtime: false }, cfg).expect("start");
    let (mut text, mut tones) = (String::new(), Vec::new());
    let mut hole: Option<f32> = None;
    loop {
        match e.next_event(Duration::from_secs(20)) {
            Some(EngineEvent::Decode { text: t, .. }) => text.push_str(&t),
            Some(EngineEvent::Tone { hz }) => tones.push(format!("{hz:.0}")),
            Some(EngineEvent::Stats { hole_hz: Some(h), .. }) => hole = Some(h),
            Some(EngineEvent::Stopped { .. }) | None => break,
            _ => {}
        }
    }
    let hole = hole.map(|h| format!(" | hole at {h:.0} Hz")).unwrap_or_default();
    println!("tones {}{hole} | {}", tones.join(" "), text.trim());
}
