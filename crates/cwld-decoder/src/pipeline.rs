//! Audio in, text out: the front-end and the decoder joined together.

use crate::decoder::{DecodeUpdate, Decoder, DecoderConfig};
use crate::lang::WordCorrector;
use cwld_dsp::frontend::{Frontend, FrontendConfig};
use cwld_dsp::seek::ToneSeeker;
use cwld_dsp::FRAME_RATE;

const PENDING_FRAMES: usize = 400;
/// Audio kept for listening again after a move to another pitch.
const RAW_SECONDS: f64 = 3.0;
/// After a click on the waterfall the decoder stays put this long.
const MANUAL_HOLD_S: f64 = 10.0;
/// After a move by itself the decoder stays put this long, so it does not jump back and forth.
const AUTO_HOLD_S: f64 = 2.0;
/// Below this score (dB) a usable but faint signal no longer keeps the decoder at its pitch.
const QUIET_BELOW_DB: f64 = 12.0;

#[derive(Clone, Debug)]
pub struct PipelineConfig {
    pub sample_rate: u32,
    pub tone_hz: f64,
    /// Move to the strongest keyed tone by itself whenever the current pitch is quiet.
    pub auto_tone: bool,
    /// Score (peak over median, dB) a tone needs before the decoder moves to it by itself.
    pub seek_db: f64,
    /// The current pitch must be quiet this long before the decoder moves away.
    pub seek_quiet_s: f64,
    pub afc: bool,
    /// Below this separation of signal and noise the evidence is ignored.
    pub min_separation: f32,
    /// The front-end bandwidth follows the speed: bandwidth = factor / unit seconds.
    pub bandwidth_factor: f64,
    pub noise_blanker: bool,
    pub auto_notch: bool,
    pub rice: bool,
    pub track_fading: bool,
    /// Word knowledge: Q-codes, abbreviations, callsign shapes, junk removal.
    pub language_model: bool,
    /// Mark strength below which a lone E or T (and similar) counts as a noise blip.
    pub junk_strength: f32,
    pub decoder: DecoderConfig,
}

impl PipelineConfig {
    pub fn new(sample_rate: u32, tone_hz: f64) -> Self {
        PipelineConfig { sample_rate, tone_hz, auto_tone: false, seek_db: 14.0, seek_quiet_s: 1.5, afc: true, min_separation: 2.0, bandwidth_factor: 1.2, noise_blanker: true, auto_notch: true, rice: false, track_fading: true, language_model: true, junk_strength: 5.0, decoder: DecoderConfig::default() }
    }
}

pub struct Pipeline {
    cfg: PipelineConfig,
    front: Frontend,
    dec: Decoder,
    words: WordCorrector,
    flushed: Vec<(char, f32)>,
    seeker: Option<ToneSeeker>,
    /// The last few seconds of audio, so a move to another pitch can listen again from the
    /// moment the kept (not yet decoded) evidence starts.
    raw: std::collections::VecDeque<f32>,
    /// Seeker measurements (0.25 s each) since the current pitch last carried a signal.
    quiet_updates: usize,
    last_usable: bool,
    /// Frames during which the decoder may not move by itself (after a move or a click).
    hold_frames: usize,
    /// Evidence kept back while no signal is judged present, so the start of a signal that
    /// is only recognised a moment later is not lost.
    pending: std::collections::VecDeque<f32>,
    snr_db: Option<f32>,
}

impl Pipeline {
    pub fn new(cfg: PipelineConfig) -> Self {
        let mut fc = FrontendConfig::new(cfg.sample_rate, cfg.tone_hz);
        fc.afc = cfg.afc;
        fc.noise_blanker = cfg.noise_blanker;
        fc.auto_notch = cfg.auto_notch;
        fc.rice = cfg.rice;
        fc.track_fading = cfg.track_fading;
        let mut words = WordCorrector::new(cfg.language_model);
        words.junk_strength = cfg.junk_strength;
        Pipeline { front: Frontend::new(fc), dec: Decoder::new(cfg.decoder.clone()), words, flushed: Vec::new(), seeker: cfg.auto_tone.then(|| ToneSeeker::new(cfg.sample_rate, 300.0, 1200.0)), cfg, raw: std::collections::VecDeque::new(), quiet_updates: 0, last_usable: false, hold_frames: 0, pending: std::collections::VecDeque::new(), snr_db: None }
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

    /// The operator chose a pitch (a click on the waterfall). Automatic moves wait a while.
    pub fn retune(&mut self, hz: f64) {
        self.front.retune(hz);
        self.quiet_updates = 0;
        self.hold_frames = (MANUAL_HOLD_S * FRAME_RATE) as usize;
    }

    /// Switches the automatic search for the strongest tone on or off.
    pub fn set_auto_tone(&mut self, on: bool) {
        self.cfg.auto_tone = on;
        if on && self.seeker.is_none() {
            self.seeker = Some(ToneSeeker::new(self.cfg.sample_rate, 300.0, 1200.0));
        }
        if !on {
            self.seeker = None;
        }
    }

    /// Moves to the strongest tone when the current pitch has been quiet for a while.
    fn consider_move(&mut self, incoming: usize) {
        let Some(sk) = &self.seeker else { return };
        let cur = self.front.tone_hz();
        // Is something still going on at the current pitch? A strong tone, or a weaker one the
        // decoder can still use.
        let here = sk.recent_score_at(cur).unwrap_or(0.0);
        let busy = here >= self.cfg.seek_db || (self.last_usable && here >= QUIET_BELOW_DB);
        self.quiet_updates = if busy { 0 } else { self.quiet_updates + 1 };
        let Some(c) = sk.best() else { return };
        if self.hold_frames > 0 || c.score_db < self.cfg.seek_db || c.steady < 3 || (c.hz - cur).abs() < 20.0 {
            return;
        }
        if (self.quiet_updates as f64) * 0.25 < self.cfg.seek_quiet_s {
            return;
        }
        // A new signal: what the decoder held of the old pitch is noise, and the last seconds
        // of audio are heard again at the new pitch, so the start of the signal is not lost.
        self.pending.clear();
        self.dec = Decoder::new(self.cfg.decoder.clone());
        self.words.discard();
        self.front.retune(c.hz);
        let end = self.raw.len() - incoming;
        let replay: Vec<f32> = self.raw.range(..end).cloned().collect();
        self.quiet_updates = 0;
        self.hold_frames = (AUTO_HOLD_S * FRAME_RATE) as usize;
        self.process(&replay);
    }

    /// The separation of signal and noise below which the evidence is ignored.
    pub fn set_min_separation(&mut self, v: f32) {
        self.cfg.min_separation = v;
    }

    pub fn push(&mut self, samples: &[f32]) {
        self.raw.extend(samples.iter().cloned());
        let max = (RAW_SECONDS * self.cfg.sample_rate as f64) as usize;
        if self.raw.len() > max {
            let extra = self.raw.len() - max;
            self.raw.drain(..extra);
        }
        let measured = self.seeker.as_mut().is_some_and(|sk| sk.push(samples));
        if measured {
            self.consider_move(samples.len());
        }
        self.process(samples);
    }

    fn process(&mut self, samples: &[f32]) {
        let frames = self.front.push(samples);
        if self.front.take_notch_changed() && !self.pending.is_empty() {
            // Kept frames were measured with the carrier still in: they are worthless, but
            // they still count as time.
            let n = self.pending.len();
            self.pending.clear();
            self.dec.push_llr(&vec![0.0; n]);
        }
        let sep = self.front.levels().map(|l| l.separation).unwrap_or(0.0);
        let usable = self.front.signal_present() && sep >= self.cfg.min_separation;
        self.hold_frames = self.hold_frames.saturating_sub(frames.len());
        self.last_usable = usable;
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
            // Frames that fall out of the hold buffer still count as time: the decoder gets
            // "no evidence" for them. Dropping them would glue the text before and after a
            // fade together and break the timing.
            let mut lost = 0;
            while self.pending.len() > PENDING_FRAMES {
                self.pending.pop_front();
                lost += 1;
            }
            if lost > 0 {
                self.dec.push_llr(&vec![0.0; lost]);
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

    /// Words the model should know: own callsign, name, QTH, callsigns from the log.
    pub fn add_words(&mut self, words: &[&str]) {
        for w in words {
            self.words.model.add(w);
        }
    }

    pub fn flush(&mut self) {
        self.dec.flush();
        let up = self.dec.poll();
        let mut out = self.words.push(&up);
        out.extend(self.words.flush());
        self.flushed.extend(out);
    }

    /// New final text (whole words once the word model is on) and the tentative rest.
    pub fn poll(&mut self) -> DecodeUpdate {
        let raw = self.dec.poll();
        let mut done = std::mem::take(&mut self.flushed);
        done.extend(self.words.push(&raw));
        let mut tentative = self.words.held();
        tentative.extend(raw.tentative.iter().cloned());
        DecodeUpdate {
            committed: done.iter().map(|x| x.0).collect(),
            committed_conf: done.iter().map(|x| x.1).collect(),
            committed_alts: Vec::new(),
            committed_strength: Vec::new(),
            tentative,
            wpm: raw.wpm,
            locked: raw.locked,
        }
    }

    pub fn frame_rate() -> f64 {
        FRAME_RATE
    }
}
