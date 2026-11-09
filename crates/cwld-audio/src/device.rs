//! Sound card inputs through cpal. Without the `device` feature (tests, CI) there are none.

use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct InputDevice {
    pub name: String,
    /// The system's default input.
    pub default: bool,
    /// Looks like a radio: "USB Audio CODEC" (Icom) or a QMX.
    pub suggested: bool,
}

pub fn looks_like_radio(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.contains("usb audio codec") || n.contains("qmx") || n.contains("ic-7300")
}

#[cfg(not(feature = "device"))]
pub fn list_inputs() -> Vec<InputDevice> {
    Vec::new()
}

#[cfg(feature = "device")]
pub fn list_inputs() -> Vec<InputDevice> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    let default = host.default_input_device().and_then(|d| d.name().ok());
    let mut out = Vec::new();
    if let Ok(devs) = host.input_devices() {
        for d in devs {
            if let Ok(name) = d.name() {
                out.push(InputDevice { default: Some(&name) == default.as_ref(), suggested: looks_like_radio(&name), name });
            }
        }
    }
    out
}

#[cfg(feature = "device")]
pub(crate) mod open {
    use crate::ring::Producer;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// Opens an input. `name` None or empty: a radio-looking input, else the default input.
    /// The stream must stay alive (and on this thread) for as long as audio is wanted.
    pub fn open(name: Option<&str>, mut prod: Producer, failed: Arc<AtomicBool>) -> Result<(cpal::Stream, u32, String), String> {
        let host = cpal::default_host();
        let wanted = name.filter(|n| !n.is_empty());
        let device = match wanted {
            Some(n) => host
                .input_devices()
                .map_err(|e| e.to_string())?
                .find(|d| d.name().map(|x| x == n).unwrap_or(false))
                .ok_or_else(|| format!("input not found: {n}"))?,
            None => {
                let radio = host.input_devices().ok().and_then(|mut it| it.find(|d| d.name().map(|x| super::looks_like_radio(&x)).unwrap_or(false)));
                radio.or_else(|| host.default_input_device()).ok_or_else(|| "no audio input".to_string())?
            }
        };
        let dev_name = device.name().unwrap_or_default();
        let config = device.default_input_config().map_err(|e| e.to_string())?;
        let channels = config.channels().max(1) as usize;
        let rate = config.sample_rate().0;
        let fmt = config.sample_format();
        let stream_cfg: cpal::StreamConfig = config.into();
        let fail = failed.clone();
        let on_err = move |_e: cpal::StreamError| fail.store(true, Ordering::Relaxed);

        // The callback mixes to mono into a small stack buffer: no allocation, no lock.
        macro_rules! build {
            ($t:ty, $conv:expr) => {{
                let conv = $conv;
                device.build_input_stream(
                    &stream_cfg,
                    move |data: &[$t], _: &cpal::InputCallbackInfo| {
                        let mut mono = [0f32; 512];
                        let mut n = 0;
                        for frame in data.chunks(channels) {
                            let s: f32 = frame.iter().map(|v| conv(*v)).sum::<f32>() / channels as f32;
                            mono[n] = s;
                            n += 1;
                            if n == mono.len() {
                                prod.push(&mono);
                                n = 0;
                            }
                        }
                        if n > 0 {
                            prod.push(&mono[..n]);
                        }
                    },
                    on_err,
                    None,
                )
            }};
        }
        let stream = match fmt {
            cpal::SampleFormat::F32 => build!(f32, |v: f32| v),
            cpal::SampleFormat::I16 => build!(i16, |v: i16| v as f32 / 32768.0),
            cpal::SampleFormat::U16 => build!(u16, |v: u16| (v as f32 - 32768.0) / 32768.0),
            cpal::SampleFormat::I32 => build!(i32, |v: i32| v as f32 / 2_147_483_648.0),
            other => return Err(format!("unsupported sample format {other:?}")),
        }
        .map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;
        Ok((stream, rate, dev_name))
    }
}

/// Playback of the practice band on the default output (headphones or speakers).
#[cfg(feature = "device")]
pub(crate) mod output {
    use crate::ring::Consumer;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    /// Most audio the ring may hold before the oldest is skipped: keeps the delay short.
    const MAX_BACKLOG: usize = 3200;

    /// Opens the default output and plays `cons` (mono at `in_rate`) on every channel.
    pub fn open(mut cons: Consumer, in_rate: u32) -> Result<cpal::Stream, String> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or_else(|| "no audio output".to_string())?;
        let config = device.default_output_config().map_err(|e| e.to_string())?;
        let channels = config.channels().max(1) as usize;
        let step = in_rate as f64 / config.sample_rate().0 as f64;
        let fmt = config.sample_format();
        let stream_cfg: cpal::StreamConfig = config.into();
        let mut prev = 0f32;
        let mut cur = 0f32;
        let mut frac = 0f64;
        // Linear interpolation from the band rate to the device rate. An empty ring plays silence.
        let mut next = move || {
            while cons.len() > MAX_BACKLOG {
                cons.pop();
            }
            while frac >= 1.0 {
                prev = cur;
                cur = cons.pop().unwrap_or(0.0);
                frac -= 1.0;
            }
            let v = prev + (cur - prev) * frac as f32;
            frac += step;
            v
        };
        macro_rules! build {
            ($t:ty, $conv:expr) => {{
                let conv = $conv;
                device.build_output_stream(
                    &stream_cfg,
                    move |data: &mut [$t], _: &cpal::OutputCallbackInfo| {
                        for frame in data.chunks_mut(channels) {
                            let v = conv(next());
                            for s in frame.iter_mut() {
                                *s = v;
                            }
                        }
                    },
                    |_e| {},
                    None,
                )
            }};
        }
        let stream = match fmt {
            cpal::SampleFormat::F32 => build!(f32, |v: f32| v),
            cpal::SampleFormat::I16 => build!(i16, |v: f32| (v.clamp(-1.0, 1.0) * 32767.0) as i16),
            cpal::SampleFormat::U16 => build!(u16, |v: f32| ((v.clamp(-1.0, 1.0) * 32767.0) + 32768.0) as u16),
            cpal::SampleFormat::I32 => build!(i32, |v: f32| (v.clamp(-1.0, 1.0) as f64 * 2_147_483_647.0) as i32),
            other => return Err(format!("unsupported sample format {other:?}")),
        }
        .map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radio_inputs_are_recognised() {
        assert!(looks_like_radio("Microphone (USB Audio CODEC )"));
        assert!(looks_like_radio("Line (QMX Transceiver)"));
        assert!(!looks_like_radio("Microphone Array (Realtek(R) Audio)"));
    }
}
