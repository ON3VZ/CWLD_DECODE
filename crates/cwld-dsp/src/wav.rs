//! Minimal WAV reading and writing: 16 bit PCM, mono (stereo is mixed down on reading).

use std::io::{self, Read, Write};

pub fn write_mono16<W: Write>(mut w: W, sample_rate: u32, samples: &[f32]) -> io::Result<()> {
    let data_len = (samples.len() * 2) as u32;
    w.write_all(b"RIFF")?;
    w.write_all(&(36 + data_len).to_le_bytes())?;
    w.write_all(b"WAVEfmt ")?;
    w.write_all(&16u32.to_le_bytes())?;
    w.write_all(&1u16.to_le_bytes())?; // PCM
    w.write_all(&1u16.to_le_bytes())?; // mono
    w.write_all(&sample_rate.to_le_bytes())?;
    w.write_all(&(sample_rate * 2).to_le_bytes())?;
    w.write_all(&2u16.to_le_bytes())?;
    w.write_all(&16u16.to_le_bytes())?;
    w.write_all(b"data")?;
    w.write_all(&data_len.to_le_bytes())?;
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
        w.write_all(&v.to_le_bytes())?;
    }
    Ok(())
}

/// Returns (sample rate, mono samples in -1..1).
pub fn read<R: Read>(mut r: R) -> io::Result<(u32, Vec<f32>)> {
    let mut bytes = Vec::new();
    r.read_to_end(&mut bytes)?;
    let bad = |m: &str| io::Error::new(io::ErrorKind::InvalidData, m.to_string());
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(bad("not a WAV file"));
    }
    let (mut rate, mut channels, mut bits, mut format) = (0u32, 0u16, 0u16, 0u16);
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let len = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let body = pos + 8;
        let end = (body + len).min(bytes.len());
        if id == b"fmt " && end - body >= 16 {
            format = u16::from_le_bytes(bytes[body..body + 2].try_into().unwrap());
            channels = u16::from_le_bytes(bytes[body + 2..body + 4].try_into().unwrap());
            rate = u32::from_le_bytes(bytes[body + 4..body + 8].try_into().unwrap());
            bits = u16::from_le_bytes(bytes[body + 14..body + 16].try_into().unwrap());
        } else if id == b"data" {
            if format != 1 || bits != 16 || channels == 0 || rate == 0 {
                return Err(bad("only 16 bit PCM WAV is supported"));
            }
            let frame = 2 * channels as usize;
            let out = bytes[body..end]
                .chunks_exact(frame)
                .map(|f| {
                    let sum: f32 = f.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0).sum();
                    sum / channels as f32
                })
                .collect();
            return Ok((rate, out));
        }
        pos = body + len + (len & 1);
    }
    Err(bad("no data chunk"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let s: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.05).sin() * 0.5).collect();
        let mut buf = Vec::new();
        write_mono16(&mut buf, 8000, &s).unwrap();
        let (rate, back) = read(&buf[..]).unwrap();
        assert_eq!(rate, 8000);
        assert_eq!(back.len(), s.len());
        for (a, b) in s.iter().zip(&back) {
            assert!((a - b).abs() < 1e-3);
        }
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(read(&b"hello world, not a wav"[..]).is_err());
    }
}
