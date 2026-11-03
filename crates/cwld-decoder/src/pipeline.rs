//! Audio in, text out: the front-end and the decoder joined together.

use crate::decoder::{DecodeUpdate, Decoder, DecoderConfig};
use cwld_dsp::frontend::{find_tone, Frontend, FrontendConfig};
use cwld_dsp::FRAME_RATE;

const PENDING_FRAMES: usize = 400;

#[derive(Clone, Debug)]
pub struct PipelineConfig {
    pub sample_rate: u32,
    pub tone_hz: f64,
    /// Look for the strongest tone by itself while no signal has been found.
    pub auto_tone: bool,
    pub afc: bool,
    /// Below this separation of signal and noise the evidence is ignored.
    pub min_separation: f32,
    /// The front-end bandwidth follows the speed: bandwidth = factor / unit seconds.
    pub bandwidth_factor: f64,
    pub decoder: DecoderConfig,
}

impl PipelineConfig {
    pub fn new(sample_rate: u32, tone_hz: f64) -> Self {
        PipelineConfig { sample_rate, tone_hz, auto_tone: false, afc: true, min_separation: 2.0, bandwidth_factor: 1.2, decoder: DecoderConfig::default() }
    }
}

pub struct Pipeline {
    cfg: PipelineConfig,
    front: Frontend,
    dec: Decoder,
    probe: Vec<f32>,
    /// Evidence kept back while no signal is judged present, so the start of a signal that
    /// is only recognised a moment later is not lost.
    pending: std::collections::VecDeque<f32>,
    snr_db: Option<f32>,
}

impl Pipeline {
    pub fn new(cfg: PipelineConfig) -> Self {
        let mut fc = FrontendConfig::new(cfg.sample_rate, cfg.tone_hz);
        fc.afc = cfg.afc;
        Pipeline { front: Frontend::new(fc), dec: Decoder::new(cfg.decoder.clone()), cfg, probe: Vec::new(), pending: std::collections::VecDeque::new(), snr_db: None }
    }

    pub fn sample_rate(&self) -> u32 {
        self.cfg.sample_rate
    }

    pub fn tone_hz(&self) -> f64 {
        self.front.tone_hz()
    }

    pub fn wpm(&self) -> Option<f64> {
        self.dec.wpm()
    }

    /// Signal to noise in the filter bandwidth, in dB, if a signal is present.
    pub fn snr_db(&self) -> Option<f32> {
        self.snr_db
    }

    pub fn retune(&mut self, hz: f64) {
        self.front.retune(hz);
    }

    pub fn push(&mut self, samples: &[f32]) {
        if self.cfg.auto_tone && self.probe.len() < self.cfg.sample_rate as usize * 2 {
            self.probe.extend_from_slice(samples);
            if self.probe.len() >= self.cfg.sample_rate as usize {
                if let Some((f, ratio)) = find_tone(&self.probe, self.cfg.sample_rate, 300.0, 1200.0) {
                    if ratio > 40.0 {
                        self.front.retune(f);
                        self.cfg.auto_tone = false;
                        self.probe.clear();
                    }
                }
            }
        }
        let frames = self.front.push(samples);
        let sep = self.front.levels().map(|l| l.separation).unwrap_or(0.0);
        let usable = self.front.signal_present() && sep >= self.cfg.min_separation;
        if usable {
            if !self.pending.is_empty() {
                // Re-score the kept frames with the levels known now.
                let held: Vec<f32> = self.pending.drain(..).map(|e| self.front.llr_for(e)).collect();
                self.dec.push_llr(&held);
            }
            let llr: Vec<f32> = frames.iter().map(|f| f.llr).collect();
            self.dec.push_llr(&llr);
        } else {
            self.pending.extend(frames.iter().map(|f| f.env));
            while self.pending.len() > PENDING_FRAMES {
                self.pending.pop_front();
            }
        }
        if let Some(l) = self.front.levels() {
            if l.noise_sd > 0.0 && usable {
                // Power ratio of signal to noise in the filter, using E[e^2] of Rayleigh noise.
                let noise_pow = (l.noise as f64 / (std::f64::consts::PI / 2.0).sqrt()).powi(2) * 2.0;
                let sig_pow = (l.signal as f64).powi(2);
                self.snr_db = Some((10.0 * (sig_pow / noise_pow.max(1e-18)).log10()) as f32);
            } else {
                self.snr_db = None;
            }
        }
        if let Some(wpm) = self.dec.wpm() {
            let u_sec = 1.2 / wpm;
            self.front.set_bandwidth(self.cfg.bandwidth_factor / u_sec);
        }
    }

    pub fn flush(&mut self) {
        self.dec.flush();
    }

    pub fn poll(&mut self) -> DecodeUpdate {
        self.dec.poll()
    }

    pub fn frame_rate() -> f64 {
        FRAME_RATE
    }
}
