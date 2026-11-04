//! Records audio to a 16 bit mono WAV file while it plays, so test recordings can be made from
//! the app and played into other decoders (CwGet reads WAV files) for a fair comparison.

use std::fs::File;
use std::io::{self, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// One hour at most: a forgotten recording must not fill the disk.
pub const MAX_SECONDS: u32 = 3600;

pub struct Recorder {
    out: BufWriter<File>,
    path: PathBuf,
    rate: u32,
    samples: u32,
}

impl Recorder {
    pub fn create(path: &Path, rate: u32) -> io::Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut out = BufWriter::new(File::create(path)?);
        // Header with zero lengths. They are filled in by `finish`.
        out.write_all(b"RIFF")?;
        out.write_all(&36u32.to_le_bytes())?;
        out.write_all(b"WAVEfmt ")?;
        out.write_all(&16u32.to_le_bytes())?;
        out.write_all(&1u16.to_le_bytes())?;
        out.write_all(&1u16.to_le_bytes())?;
        out.write_all(&rate.to_le_bytes())?;
        out.write_all(&(rate * 2).to_le_bytes())?;
        out.write_all(&2u16.to_le_bytes())?;
        out.write_all(&16u16.to_le_bytes())?;
        out.write_all(b"data")?;
        out.write_all(&0u32.to_le_bytes())?;
        Ok(Recorder { out, path: path.to_path_buf(), rate, samples: 0 })
    }

    /// Returns false once the maximum length is reached (the caller then finishes).
    pub fn write(&mut self, samples: &[f32]) -> io::Result<bool> {
        let max = MAX_SECONDS * self.rate;
        for s in samples {
            if self.samples >= max {
                return Ok(false);
            }
            let v = (s.clamp(-1.0, 1.0) * 32767.0).round() as i16;
            self.out.write_all(&v.to_le_bytes())?;
            self.samples += 1;
        }
        Ok(true)
    }

    pub fn seconds(&self) -> f64 {
        self.samples as f64 / self.rate as f64
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn finish(mut self) -> io::Result<PathBuf> {
        let data = self.samples * 2;
        self.out.flush()?;
        let mut f = self.out.into_inner().map_err(|e| e.into_error())?;
        f.seek(SeekFrom::Start(4))?;
        f.write_all(&(36 + data).to_le_bytes())?;
        f.seek(SeekFrom::Start(40))?;
        f.write_all(&data.to_le_bytes())?;
        f.flush()?;
        Ok(self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_recording_reads_back() {
        let dir = std::env::temp_dir().join(format!("cwld-rec-{}", std::process::id()));
        let path = dir.join("test.wav");
        let mut r = Recorder::create(&path, 48_000).unwrap();
        let x: Vec<f32> = (0..4800).map(|i| (i as f32 * 0.01).sin() * 0.5).collect();
        r.write(&x).unwrap();
        r.write(&x).unwrap();
        assert!((r.seconds() - 0.2).abs() < 1e-9);
        let p = r.finish().unwrap();
        let (rate, back) = cwld_dsp::wav::read(std::fs::File::open(&p).unwrap()).unwrap();
        assert_eq!(rate, 48_000);
        assert_eq!(back.len(), 9600);
        assert!((back[100] - x[100]).abs() < 1e-3);
        let _ = std::fs::remove_dir_all(dir);
    }
}
