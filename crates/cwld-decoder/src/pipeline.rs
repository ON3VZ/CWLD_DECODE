//! Audio in, text out: the front-end and the decoder joined together.

use crate::decoder::{DecodeUpdate, Decoder, DecoderConfig};
use crate::lang::WordCorrector;
use cwld_dsp::frontend::{Frontend, FrontendConfig};
use cwld_dsp::seek::ToneSeeker;
use cwld_dsp::FRAME_RATE;

const PENDING_FRAMES: usize = 600;
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
    /// Only decode while the envelope looks like keyed CW (see `gate`).
    pub cw_gate: bool,
    /// Do not decode while the audio looks like speech or another wide signal (see `voice`).
    pub voice_gate: bool,
    /// Count weak marks in fast fading as key-down (see `FrontendConfig::weak_floor_sd`).
    pub fading: bool,
    /// See `FrontendConfig::weak_floor_sd` and `weak_cost`. `None` keeps the defaults.
    pub weak_marks: Option<(f64, f64, f64)>,
    /// Decide the word gaps from the sender's own spacing and word knowledge (see `spacing`).
    pub spacing: bool,
    /// Keep the key down and up times (for the paddle trainer), see `take_keying`.
    pub keying: bool,
    /// Experimental: take the key-down evidence from the small neural network (see `neural`),
    /// with this strength. A negative value adds the network to the classic evidence with that
    /// strength instead of replacing it. `None`: the classic evidence only.
    pub neural: Option<f32>,
    pub decoder: DecoderConfig,
}

impl PipelineConfig {
    pub fn new(sample_rate: u32, tone_hz: f64) -> Self {
        PipelineConfig { sample_rate, tone_hz, auto_tone: false, seek_db: 14.0, seek_quiet_s: 1.5, afc: true, min_separation: 2.0, bandwidth_factor: 1.2, noise_blanker: true, auto_notch: true, rice: false, track_fading: true, language_model: true, junk_strength: 5.0, cw_gate: true, voice_gate: true, fading: true, weak_marks: None, spacing: true, keying: false, neural: None, decoder: DecoderConfig::default() }
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
    gate: crate::gate::EnvelopeGate,
    /// The envelope looks like keyed CW (see `gate`). Closed until judged, so noise at the start
    /// does not produce text.
    cw_open: bool,
    noise_evals: u32,
    last_eval: u64,
    voice: crate::voice::VoiceDetector,
    /// Speech (or another wide signal) is heard: nothing is decoded until it has been clear a while.
    voice_heard: bool,
    voice_clear_evals: u32,
    last_voice_eval: u64,
    keying: KeyingRecorder,
    spacer: crate::spacing::Spacer,
    neural: Option<crate::neural::NeuralLlr>,
    /// Frames processed so far, and the count when text last came out of the decoder.
    frames_seen: u64,
    last_text_at: u64,
    /// The speech check as configured, restored when the paddle trainer stops.
    voice_gate_base: bool,
    /// Audio of the last seconds is processed again after a pitch move.
    replaying: bool,
}

/// Key down and key up runs in frames, with blips of one frame folded into the run around them.
#[derive(Default)]
struct KeyingRecorder {
    cur: bool,
    len: u32,
    runs: Vec<(bool, u32)>,
    /// Envelope floor (key up level), follows downward fast and upward slowly.
    floor: f32,
    /// Recent peak (key down level), decays slowly.
    peak: f32,
    frames: u32,
    /// Own tone detector over the whole audio passband, see `KeyTone`.
    tone: Option<KeyTone>,
}

/// Finds a keyed tone anywhere between 300 and 1200 Hz without knowing its pitch: every 5 ms the
/// level of the strongest of a row of overlapping 10 ms filters. The paddle trainer listens to
/// the radio's own sidetone, whose pitch is the radio's CW pitch setting, not the pitch the
/// decoder happens to be tuned to (a signal clicked before, with "Strongest" off).
struct KeyTone {
    /// One Hann-windowed quadrature pair per filter: (cos, sin) coefficients for the window.
    bank: Vec<(Vec<f32>, Vec<f32>)>,
    buf: std::collections::VecDeque<f32>,
    hop: usize,
    win: usize,
    since: usize,
}

impl KeyTone {
    fn new(sample_rate: u32) -> Self {
        let hop = (sample_rate as f64 / FRAME_RATE).round() as usize;
        let win = 2 * hop;
        let bank = (0..=9)
            .map(|k| {
                let hz = 300.0 + 100.0 * k as f64;
                let w = 2.0 * std::f64::consts::PI * hz / sample_rate as f64;
                let hann = |i: usize| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * (i as f64 + 0.5) / win as f64).cos();
                ((0..win).map(|i| (hann(i) * (w * i as f64).cos()) as f32).collect(), (0..win).map(|i| (hann(i) * (w * i as f64).sin()) as f32).collect())
            })
            .collect();
        KeyTone { bank, buf: std::collections::VecDeque::with_capacity(win), hop, win, since: 0 }
    }

    /// Takes samples and gives one level per 5 ms frame.
    fn push(&mut self, samples: &[f32], out: &mut Vec<f32>) {
        for &x in samples {
            if self.buf.len() == self.win {
                self.buf.pop_front();
            }
            self.buf.push_back(x);
            self.since += 1;
            if self.since >= self.hop && self.buf.len() == self.win {
                self.since = 0;
                let mut best = 0.0f32;
                for (c, s) in &self.bank {
                    let (mut re, mut im) = (0.0f32, 0.0f32);
                    for (i, v) in self.buf.iter().enumerate() {
                        re += v * c[i];
                        im += v * s[i];
                    }
                    best = best.max(re * re + im * im);
                }
                out.push(best.sqrt());
            }
        }
    }
}

/// The key state for the paddle trainer comes from the envelope at the sidetone pitch, not from the
/// decoder's likelihoods: those need a few characters to settle and flip on noise. Down above
/// `KEY_DOWN_AT` of the way from floor to peak, up below `KEY_UP_AT` (hysteresis), and only when the
/// peak stands at least `KEY_MIN_RATIO` above the floor, so band noise alone never keys.
const KEY_DOWN_AT: f32 = 0.5;
const KEY_UP_AT: f32 = 0.3;
const KEY_MIN_RATIO: f32 = 4.0;
/// Per frame (5 ms): the peak halves in about 3 s; the floor is an average over about one second.
const KEY_PEAK_DECAY: f32 = 0.99885;
const KEY_FLOOR_RATE: f32 = 0.005;
const KEY_WARMUP_FRAMES: u32 = 100;

/// Runs shorter than this many frames (10 ms) are taken as noise on the edge of an element.
const KEYING_MIN_FRAMES: u32 = 2;

impl KeyingRecorder {
    /// Audio samples: the key state follows from the strongest tone in the passband.
    fn push_samples(&mut self, samples: &[f32], sample_rate: u32) {
        let mut envs = Vec::new();
        self.tone.get_or_insert_with(|| KeyTone::new(sample_rate)).push(samples, &mut envs);
        for e in envs {
            self.push_env(e);
        }
    }

    /// One frame of envelope: updates floor and peak and records the key state.
    fn push_env(&mut self, env: f32) {
        let env = env.max(0.0);
        self.frames += 1;
        if self.frames <= KEY_WARMUP_FRAMES {
            // The first half second only measures the floor (running mean); no key-down yet.
            self.floor += (env - self.floor) / self.frames as f32;
            self.peak = self.peak.max(env);
            self.push(false);
            return;
        }
        // The floor is the average key-up level (band noise), not its deepest dip, so ordinary
        // noise peaks stay well below `KEY_MIN_RATIO` times it. It does not move while the key is down.
        if !self.cur {
            self.floor += (env - self.floor) * KEY_FLOOR_RATE;
        }
        self.peak = (self.peak * KEY_PEAK_DECAY).max(env);
        let span = self.peak - self.floor;
        let real = self.peak > self.floor.max(1e-9) * KEY_MIN_RATIO;
        let down = real && if self.cur { env > self.floor + KEY_UP_AT * span } else { env > self.floor + KEY_DOWN_AT * span };
        self.push(down);
    }

    fn push(&mut self, down: bool) {
        if down == self.cur {
            self.len += 1;
            return;
        }
        if self.len < KEYING_MIN_FRAMES {
            if let Some(&(prev, plen)) = self.runs.last() {
                if prev == down {
                    // The short run was a blip: the run before it simply goes on.
                    self.runs.pop();
                    self.cur = down;
                    self.len += plen + 1;
                    return;
                }
            }
        }
        if self.len > 0 {
            self.runs.push((self.cur, self.len));
        }
        self.cur = down;
        self.len = 1;
    }
}

impl Pipeline {
    pub fn new(cfg: PipelineConfig) -> Self {
        let mut fc = FrontendConfig::new(cfg.sample_rate, cfg.tone_hz);
        fc.afc = cfg.afc;
        fc.noise_blanker = cfg.noise_blanker;
        fc.auto_notch = cfg.auto_notch;
        fc.rice = cfg.rice;
        fc.track_fading = cfg.track_fading;
        if let Some((floor, cost, depth)) = cfg.weak_marks {
            fc.weak_floor_sd = floor;
            fc.weak_cost = cost;
            fc.weak_depth = depth;
        }
        let cfg_fading = cfg.fading;
        let mut words = WordCorrector::new(cfg.language_model);
        words.junk_strength = cfg.junk_strength;
        let junk = cfg.junk_strength;
        let cfg_gate = cfg.cw_gate;
        let cfg_rate = cfg.sample_rate;
        let cfg_voice = cfg.voice_gate;
        let cfg_spacing = cfg.spacing;
        let mut cfg = cfg;
        if cfg.keying {
            // A pipeline rebuilt during a paddle exercise (engine reset) keeps the speech check off.
            cfg.voice_gate = false;
        }
        let cfg_neural = cfg.neural;
        Pipeline { neural: cfg_neural.map(|a| { let mut n = crate::neural::NeuralLlr::new(a.abs()); n.mix = a < 0.0; n }), front: { let mut f = Frontend::new(fc); f.set_weak_marks(cfg_fading); f }, dec: Decoder::new(cfg.decoder.clone()), words, flushed: Vec::new(), seeker: Some(ToneSeeker::new(cfg.sample_rate, 300.0, 1200.0)), cfg, raw: std::collections::VecDeque::new(), quiet_updates: 0, last_usable: false, hold_frames: 0, pending: std::collections::VecDeque::new(), snr_db: None, gate: crate::gate::EnvelopeGate::new(), cw_open: !cfg_gate, noise_evals: 0, last_eval: 0, voice: crate::voice::VoiceDetector::new(cfg_rate), voice_heard: false, voice_clear_evals: 0, last_voice_eval: 0, keying: KeyingRecorder::default(), spacer: { let mut sp = crate::spacing::Spacer::new(cfg_spacing); sp.junk_strength = junk; sp }, voice_gate_base: cfg_voice, replaying: false, frames_seen: 0, last_text_at: 0 }
    }

    /// Keyed CW is in the envelope (not just noise at this pitch).
    pub fn cw_present(&self) -> bool {
        self.cw_open
    }

    /// Speech or another wide signal is heard (an SSB station, or the radio is not on CW).
    /// The middle of a hole in the audio (Hz): a stretch of the passband that is far quieter
    /// than both sides, as a notch filter on the radio leaves behind. `None`: no hole.
    pub fn hole_hz(&self) -> Option<f64> {
        self.seeker.as_ref().and_then(|s| s.hole())
    }

    pub fn voice_heard(&self) -> bool {
        self.voice_heard
    }

    /// Starts or stops keeping the key times.
    /// While the paddle trainer listens, the speech check is off: the only signal is the radio's
    /// own sidetone, and a sidetone with overtones or a little clipping looks like speech to it
    /// (measured, see `paddle_harmonics_experiment`).
    pub fn set_keying(&mut self, on: bool) {
        self.cfg.keying = on;
        self.keying = KeyingRecorder::default();
        self.cfg.voice_gate = self.voice_gate_base && !on;
        if on {
            self.voice_heard = false;
            self.voice_clear_evals = 0;
        }
    }

    /// The finished key runs since the last call, in milliseconds: (key down, length). The run
    /// under way stays, except a key-up longer than two seconds, which is handed out so a
    /// pause after the last character is seen.
    pub fn take_keying(&mut self) -> Vec<(bool, f32)> {
        let ms = 1000.0 / FRAME_RATE as f32;
        if !self.keying.cur && self.keying.len as f64 > 2.0 * FRAME_RATE && !self.keying.runs.is_empty() {
            let len = std::mem::take(&mut self.keying.len);
            self.keying.runs.push((false, len));
        }
        // The last finished run may still be extended by a blip fold: keep it back.
        let keep = usize::from(self.keying.len < KEYING_MIN_FRAMES && !self.keying.runs.is_empty());
        let n = self.keying.runs.len() - keep;
        self.keying.runs.drain(..n).map(|(d, l)| (d, l as f32 * ms)).collect()
    }

    pub fn voice_stats(&self) -> crate::voice::VoiceStats {
        self.voice.stats
    }

    pub fn gate_stats(&self) -> crate::gate::GateStats {
        self.gate.stats
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
        // The seeker keeps listening either way, so a one-shot "go to the strongest" has data.
        self.cfg.auto_tone = on;
    }

    /// Weak marks in fading on or off.
    pub fn set_fading(&mut self, on: bool) {
        self.cfg.fading = on;
        self.front.set_weak_marks(on);
    }

    /// AFC on or off (follows a drifting tone).
    pub fn set_afc(&mut self, on: bool) {
        self.cfg.afc = on;
        self.front.set_afc(on);
    }

    /// Once: moves to the strongest keyed tone heard in the last seconds. Returns the new pitch,
    /// or `None` when nothing clear is there or the pitch is already right.
    pub fn go_strongest(&mut self) -> Option<f64> {
        let c = self.seeker.as_ref()?.best()?;
        let cur = self.front.tone_hz();
        if c.score_db < 6.0 || (c.hz - cur).abs() < 5.0 {
            return None;
        }
        self.pending.clear();
        self.dec = Decoder::new(self.cfg.decoder.clone());
        self.words.discard();
        self.spacer.discard();
        self.front.retune(c.hz);
        self.quiet_updates = 0;
        self.hold_frames = (MANUAL_HOLD_S * FRAME_RATE) as usize;
        Some(c.hz)
    }

    /// Moves to the strongest tone when the current pitch has been quiet for a while.
    fn consider_move(&mut self, incoming: usize) {
        if !self.cfg.auto_tone {
            return;
        }
        let Some(sk) = &self.seeker else { return };
        let cur = self.front.tone_hz();
        // Is something still going on at the current pitch? A strong tone, or a weaker one the
        // decoder can still use.
        let here = sk.recent_score_at(cur).unwrap_or(0.0);
        let busy = here >= self.cfg.seek_db || (self.last_usable && here >= QUIET_BELOW_DB);
        // What is heard at this pitch may be the skirt of a much stronger tone next to it.
        // On a silent band (squelch closed, only the sidetone) that skirt stands far above
        // the floor, and the decoder stayed 100 Hz off the tone for good.
        let skirt = sk.best().is_some_and(|c| (c.hz - cur).abs() >= 20.0 && here < c.score_db - 10.0);
        let busy = busy && !skirt;
        self.quiet_updates = if busy { 0 } else { self.quiet_updates + 1 };
        let Some(c) = sk.best() else { return };

        if self.hold_frames > 0 || c.score_db < self.cfg.seek_db || c.steady < 3 || (c.hz - cur).abs() < 20.0 {
            return;
        }
        if (self.quiet_updates as f64) * 0.25 < self.cfg.seek_quiet_s {
            return;
        }
        // A small step while text came out in the last seconds: the decoder is already reading
        // this station a little off pitch and the AFC is pulling it in. Starting again would
        // lose what is being read, and hearing the audio again would give the end twice.
        if (c.hz - cur).abs() < 60.0 && self.frames_seen.saturating_sub(self.last_text_at) < (RAW_SECONDS * FRAME_RATE) as u64 {
            return;
        }
        // A new signal: what the decoder held of the old pitch is noise, and the last seconds
        // of audio are heard again at the new pitch, so the start of the signal is not lost.
        self.pending.clear();
        self.dec = Decoder::new(self.cfg.decoder.clone());
        self.words.discard();
        self.spacer.discard();
        self.front.retune(c.hz);
        let end = self.raw.len() - incoming;
        let replay: Vec<f32> = self.raw.range(..end).cloned().collect();
        self.quiet_updates = 0;
        self.hold_frames = (AUTO_HOLD_S * FRAME_RATE) as usize;
        // Twice: the first pass only lets the filter find the levels and the exact pitch of the
        // new signal, the second one reads it. Read in one pass, the first characters after a
        // move came out wrong because the levels were still those of the old pitch.
        self.replaying = true;
        self.process(&replay);
        self.pending.clear();
        self.dec = Decoder::new(self.cfg.decoder.clone());
        self.words.discard();
        self.spacer.discard();
        self.flushed.clear();
        self.front.drop_held();
        self.process(&replay);
        self.replaying = false;
    }

    /// Wide filter: double bandwidth for drifting or chirpy signals.
    pub fn set_wide(&mut self, wide: bool) {
        self.cfg.bandwidth_factor = if wide { 2.4 } else { 1.2 };
        let wpm = self.dec.wpm().unwrap_or(20.0);
        self.front.set_bandwidth(self.cfg.bandwidth_factor / (1.2 / wpm));
    }

    /// The slowest and fastest speed to look for, or a fixed speed.
    pub fn set_speed_range(&mut self, min: f64, max: f64, fixed: Option<f64>) {
        self.cfg.decoder.wpm_min = min;
        self.cfg.decoder.wpm_max = max;
        self.cfg.decoder.fixed_wpm = fixed;
        self.dec.set_speed_range(min, max, fixed);
    }

    /// The bandwidth of the decoder filter now, in Hz.
    pub fn bandwidth_hz(&self) -> f64 {
        self.front.bandwidth_hz()
    }

    /// The separation of signal and noise below which the evidence is ignored.
    pub fn set_min_separation(&mut self, v: f32) {
        self.cfg.min_separation = v;
    }

    pub fn push(&mut self, samples: &[f32]) {
        if self.cfg.keying {
            // Before anything else and from the audio itself: a pitch move replays old audio
            // through `process`, which must not be counted twice.
            self.keying.push_samples(samples, self.cfg.sample_rate);
        }
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
        if self.cfg.voice_gate {
            self.voice.push(samples, self.front.tone_hz());
            if self.voice.evals != self.last_voice_eval {
                self.last_voice_eval = self.voice.evals;
                let v = self.voice.stats;
                if v.is_voice() {
                    if !self.voice_heard {
                        // What the decoder held so far was speech: drop it.
                        self.pending.clear();
                        self.dec = Decoder::new(self.cfg.decoder.clone());
                        self.words.discard();
        self.spacer.discard();
                    }
                    self.voice_heard = true;
                    self.voice_clear_evals = 0;
                } else if v.is_clear() {
                    self.voice_clear_evals += 1;
                    if self.voice_clear_evals >= crate::voice::REOPEN_AFTER {
                        self.voice_heard = false;
                    }
                }
            }
        }
        let frames = self.front.push(samples);
        // With the network on, everything below works on its delayed stream of frames.
        let frames = match &mut self.neural {
            Some(nn) => nn.push_frames(&frames),
            None => frames,
        };
        if !self.replaying {
            self.frames_seen += frames.len() as u64;
        }
        if self.front.take_notch_changed() && !self.pending.is_empty() {
            // Kept frames were measured with the carrier still in: they are worthless, but
            // they still count as time.
            let n = self.pending.len();
            self.pending.clear();
            self.dec.push_llr(&vec![0.0; n]);
        }
        let envs: Vec<f32> = frames.iter().map(|f| f.env).collect();
        self.gate.push(&envs, self.front.bandwidth_hz());
        if self.cfg.cw_gate && self.gate.evals != self.last_eval {
            self.last_eval = self.gate.evals;
            let g = self.gate.stats;
            if g.short_frac >= crate::gate::NOISE_SHORT_FRAC && g.marks >= 8 {
                self.noise_evals += 1;
                if self.noise_evals >= crate::gate::CLOSE_AFTER {
                    self.cw_open = false;
                }
            } else if g.short_frac <= crate::gate::OPEN_SHORT_FRAC {
                self.noise_evals = 0;
                self.cw_open = g.marks >= 3;
            }
        }
        let sep = self.front.levels().map(|l| l.separation).unwrap_or(0.0);
        let usable = self.cw_open && !self.voice_heard && self.front.signal_present() && sep >= self.cfg.min_separation;
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

    /// Callsigns expected on this frequency (cluster spots next to the VFO). Replaces the old ones.
    pub fn set_hints(&mut self, calls: &[&str]) {
        self.words.model.set_hints(calls);
    }

    pub fn flush(&mut self) {
        if let Some(nn) = &mut self.neural {
            let rest: Vec<f32> = nn.flush_frames().iter().map(|f| f.llr).collect();
            if self.last_usable {
                self.dec.push_llr(&rest);
            }
        }
        self.dec.flush();
        let up = self.dec.poll();
        let mut out = self.words.push(&self.spacer.push(&up, &self.words.model));
        out.extend(self.words.push(&self.spacer.flush(&self.words.model)));
        out.extend(self.words.flush());
        self.flushed.extend(out);
    }

    /// New final text (whole words once the word model is on) and the tentative rest.
    pub fn poll(&mut self) -> DecodeUpdate {
        let raw = self.dec.poll();
        if raw.committed.chars().any(|c| c != ' ') {
            self.last_text_at = self.frames_seen;
        }
        let mut done = std::mem::take(&mut self.flushed);
        let spaced = self.spacer.push(&raw, &self.words.model);
        done.extend(self.words.push(&spaced));
        let mut tentative = self.words.held();
        tentative.extend(self.spacer.held());
        tentative.extend(raw.tentative.iter().cloned());
        DecodeUpdate {
            committed: done.iter().map(|x| x.0).collect(),
            committed_conf: done.iter().map(|x| x.1).collect(),
            committed_alts: Vec::new(),
            committed_strength: Vec::new(),
            committed_gap: Vec::new(),
            committed_intra: Vec::new(),
            silence_units: raw.silence_units,
            char_gap_units: raw.char_gap_units,
            spacing_changed: false,
            tentative,
            wpm: raw.wpm,
            locked: raw.locked,
        }
    }

    pub fn frame_rate() -> f64 {
        FRAME_RATE
    }
}

#[cfg(test)]
mod paddle_harmonics_experiment {
    use super::*;
    use cwld_dsp::{generate, CwSpec};

    #[test]
    #[ignore]
    fn print_sidetone_with_overtones() {
        for (label, h2, h3, h5, clip) in [
            ("pure sine", 0.0, 0.0, 0.0, 1.0),
            ("2nd -20 dB", 0.1, 0.0, 0.0, 1.0),
            ("3rd -10 dB, 5th -14 dB (square-ish)", 0.0, 0.33, 0.2, 1.0),
            ("hard clipped at 30 percent", 0.0, 0.0, 0.0, 0.3),
        ] {
            let g = generate(&CwSpec { text: "PARIS KMRSU PARIS".into(), tone_hz: 600.0, snr_db: None, lead_s: 5.0, trail_s: 3.0, ..Default::default() });
            // The generator gives the keyed sine; rebuild the envelope from it and add overtones.
            let sr = 8000.0;
            let mut env = 0.0f32;
            let s: Vec<f32> = g.samples.iter().enumerate().map(|(i, x)| {
                env = env.max(x.abs()) * 0.995 + x.abs() * 0.005;
                let t = i as f64 / sr;
                let w = 2.0 * std::f64::consts::PI * 600.0 * t;
                let over = (h2 * (2.0 * w).sin() + h3 * (3.0 * w).sin() + h5 * (5.0 * w).sin()) as f32;
                let y = x + env.min(1.0) * over;
                y.clamp(-clip, clip) * 0.5
            }).collect();
            let mut cfg = PipelineConfig::new(8000, 700.0);
            cfg.auto_tone = true;
            cfg.min_separation = 3.5;
            let mut p = Pipeline::new(cfg);
            p.set_keying(true);
            let (mut text, mut downs, mut voice_blocks) = (String::new(), 0, 0);
            for b in s.chunks(160) {
                p.push(b);
                text.push_str(&p.poll().committed);
                downs += p.take_keying().iter().filter(|r| r.0).count();
                if p.voice_heard() { voice_blocks += 1; }
            }
            println!("{label:40} text={:?} downs={downs} voice_blocks={voice_blocks} stats={:?}", text.trim(), p.voice_stats());
        }
    }
}

#[cfg(test)]
mod paddle_keying {
    use super::*;
    use cwld_dsp::{generate, CwSpec};

    /// Elements (dots and dashes) of a text: the key-downs a perfect reading gives.
    fn elements(text: &str) -> usize {
        let table = [("P", 4), ("A", 2), ("R", 3), ("I", 2), ("S", 3), ("K", 3), ("M", 2), ("U", 3)];
        text.chars().filter(|c| *c != ' ').map(|c| table.iter().find(|(k, _)| k.starts_with(c)).unwrap().1).sum()
    }

    fn keyed(spec: CwSpec, overtones: bool) -> (Vec<(bool, f32)>, String) {
        keyed_at(spec, overtones, true, 700.0)
    }

    fn keyed_at(spec: CwSpec, overtones: bool, auto_tone: bool, decoder_hz: f64) -> (Vec<(bool, f32)>, String) {
        let mut env = 0.0f32;
        let samples: Vec<f32> = generate(&spec)
            .samples
            .iter()
            .enumerate()
            .map(|(i, x)| {
                if !overtones {
                    return *x;
                }
                env = env.max(x.abs()) * 0.995 + x.abs() * 0.005;
                let w = 2.0 * std::f64::consts::PI * spec.tone_hz * i as f64 / 8000.0;
                (x + env.min(1.0) * (0.33 * (3.0 * w).sin() + 0.2 * (5.0 * w).sin()) as f32).clamp(-0.6, 0.6)
            })
            .collect();
        let mut cfg = PipelineConfig::new(8000, decoder_hz);
        cfg.auto_tone = auto_tone;
        cfg.min_separation = 3.5;
        let mut p = Pipeline::new(cfg);
        p.set_keying(true);
        let (mut runs, mut text) = (vec![], String::new());
        for b in samples.chunks(160) {
            p.push(b);
            text.push_str(&p.poll().committed);
            runs.extend(p.take_keying());
        }
        (runs, text)
    }

    #[test]
    fn a_sidetone_with_overtones_is_decoded_not_taken_for_speech() {
        // Found with ON3VZ (0.13.2): with the speech check on, this gave no text at all.
        let (runs, text) = keyed(CwSpec { text: "PARIS KMRSU PARIS".into(), tone_hz: 600.0, lead_s: 5.0, trail_s: 3.0, ..Default::default() }, true);
        assert!(text.contains("KMRSU"), "{text:?}");
        assert_eq!(runs.iter().filter(|r| r.0).count(), elements("PARIS KMRSU PARIS"));
    }

    #[test]
    fn key_times_are_exact_clean_and_with_band_noise() {
        for snr in [None, Some(40.0), Some(25.0)] {
            let (runs, _) = keyed(CwSpec { text: "PARIS KMRSU".into(), tone_hz: 600.0, snr_db: snr, lead_s: 10.0, trail_s: 3.0, ..Default::default() }, false);
            let downs: Vec<f32> = runs.iter().filter(|r| r.0).map(|r| r.1).collect();
            assert_eq!(downs.len(), elements("PARIS KMRSU"), "snr {snr:?}: {runs:?}");
            // 20 WPM: dot 60 ms, dash 180 ms, within one frame or two.
            assert!(downs.iter().all(|d| (*d - 60.0).abs() <= 15.0 || (*d - 180.0).abs() <= 15.0), "snr {snr:?}: {downs:?}");
        }
    }

    #[test]
    fn noise_alone_never_keys() {
        let (runs, _) = keyed(CwSpec { text: "".into(), snr_db: Some(20.0), lead_s: 8.0, trail_s: 0.0, ..Default::default() }, false);
        assert_eq!(runs.iter().filter(|r| r.0).count(), 0, "{runs:?}");
    }

    #[test]
    fn key_times_do_not_depend_on_the_decoder_pitch() {
        // Found with ON3VZ (0.14.0): with "Strongest" off and the decoder left on another pitch
        // (a signal clicked earlier), the sidetone was in the audio but never registered.
        for (auto, hz, tone) in [(false, 700.0, 600.0), (false, 450.0, 600.0), (false, 500.0, 800.0), (true, 1000.0, 400.0), (false, 600.0, 600.0)] {
            for snr in [None, Some(30.0)] {
                let (runs, _) = keyed_at(CwSpec { text: "PARIS KMRSU".into(), tone_hz: tone, snr_db: snr, lead_s: 6.0, trail_s: 3.0, ..Default::default() }, false, auto, hz);
                let downs: Vec<f32> = runs.iter().filter(|r| r.0).map(|r| r.1).collect();
                assert_eq!(downs.len(), elements("PARIS KMRSU"), "auto {auto} decoder {hz} sidetone {tone} snr {snr:?}: {downs:?}");
                assert!(downs.iter().all(|d| (*d - 60.0).abs() <= 15.0 || (*d - 180.0).abs() <= 15.0), "{downs:?}");
            }
        }
    }
}
