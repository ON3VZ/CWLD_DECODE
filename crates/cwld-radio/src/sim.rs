use crate::error::RigError;
use crate::rig::{Capabilities, CwTextMode, Rig, RigInfo};
use cwld_types::{morse, CommandClass, RadioCommand, RadioMode, RadioStatus, RigEvent, ScopeSweep, SplitState};

/// Made-up stations for the simulated band scope (absolute frequencies in Hz, strength 0 to 160).
const SIM_STATIONS: [(u64, u8); 5] = [(7_012_300, 120), (7_018_000, 70), (7_023_600, 95), (7_025_400, 140), (7_041_200, 85)];
/// The simulated scope shows ±25 kHz around the VFO, like the radio in centre mode.
pub const SIM_SCOPE_HALF_SPAN: u64 = 25_000;

const POLL_SECONDS: f64 = 0.5;

/// A simulated radio. It behaves like a small CW transceiver and records every command
/// it receives, so tests can prove what did (and did not) reach the radio.
pub struct SimRig {
    status: RadioStatus,
    pending: Vec<RigEvent>,
    received: Vec<RadioCommand>,
    transmit_commands: u64,
    tx_ticks_left: u32,
    rng: u64,
    swr: f32,
    scope: bool,
    audio: crate::AudioControls,
    ticks: u64,
}

impl SimRig {
    pub fn new() -> Self {
        SimRig {
            status: RadioStatus {
                connected: true,
                frequency_hz: 7_025_400,
                mode: RadioMode::Cw,
                filter: Some(2),
                cw_wpm: 22,
                transmitting: false,
                s_meter: 55,
                split: Default::default(),
            },
            pending: vec![RigEvent::Connected(true)],
            received: Vec::new(),
            transmit_commands: 0,
            tx_ticks_left: 0,
            rng: 0x2545_F491_4F6C_DD1D,
            swr: 1.1,
            scope: false,
            audio: crate::AudioControls { af_level: Some(128), break_in: Some(1), squelch: Some(0), notch: Some(false), auto_notch: Some(false), pbt_inner: Some(crate::PBT_CENTER), pbt_outer: Some(crate::PBT_CENTER) },
            ticks: 0,
        }
    }

    /// Number of commands of class `Transmit` that reached this radio.
    pub fn transmit_command_count(&self) -> u64 {
        self.transmit_commands
    }

    /// All commands received so far (most recent last, capped at 1000).
    pub fn received(&self) -> &[RadioCommand] {
        &self.received
    }

    /// Simulates the operator turning the tuning knob on the radio.
    pub fn turn_knob(&mut self, hz: u64) {
        self.status.frequency_hz = hz;
        self.pending.push(RigEvent::Frequency(hz));
    }

    /// One sweep of the made-up band scope: noise, and the stations keyed on and off.
    fn scope_sweep(&mut self) -> ScopeSweep {
        let c = self.status.frequency_hz;
        let (lo, hi) = (c.saturating_sub(SIM_SCOPE_HALF_SPAN), c + SIM_SCOPE_HALF_SPAN);
        let n = 475usize;
        let step = (hi - lo) as f64 / n as f64;
        self.ticks += 1;
        let mut data = Vec::with_capacity(n);
        for i in 0..n {
            let f = lo as f64 + i as f64 * step;
            let mut v = 18.0 + (self.next_rand() % 14) as f64;
            for (k, (sf, level)) in SIM_STATIONS.iter().enumerate() {
                let keyed = (self.ticks / (2 + k as u64)) % 3 != 0;
                let d = (f - *sf as f64) / 120.0;
                if keyed {
                    v = v.max(f64::from(*level) * (-d * d).exp());
                }
            }
            data.push(v.min(160.0) as u8);
        }
        ScopeSweep { lo_hz: lo, hi_hz: hi, centered: true, out_of_range: false, data }
    }

    fn next_rand(&mut self) -> u64 {
        // xorshift64: small, deterministic, good enough for a fake S-meter.
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x
    }
}

impl Default for SimRig {
    fn default() -> Self {
        Self::new()
    }
}

impl Rig for SimRig {
    fn info(&self) -> RigInfo {
        RigInfo { name: "Simulated radio".to_string(), driver: "sim" }
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            cw_text: CwTextMode::Chunked { max: 30 },
            radio_decoder_text: false,
            dtr_keying: false,
            pushes_status: true,
            swr_meter: true,
            scope: true,
        }
    }

    fn execute(&mut self, cmd: &RadioCommand) -> Result<(), RigError> {
        if self.received.len() >= 1000 {
            self.received.remove(0);
        }
        self.received.push(cmd.clone());
        if cmd.class() == CommandClass::Transmit {
            self.transmit_commands += 1;
        }
        match cmd {
            RadioCommand::ReadStatus => {}
            RadioCommand::SetFrequency(hz) => {
                self.status.frequency_hz = *hz;
                self.pending.push(RigEvent::Frequency(*hz));
            }
            RadioCommand::SetMode(m) => {
                self.status.mode = *m;
                self.pending.push(RigEvent::Mode(*m));
            }
            RadioCommand::SetCwSpeed(w) => {
                self.status.cw_wpm = *w;
                self.pending.push(RigEvent::CwSpeed(*w));
            }
            RadioCommand::SendCw(text) => {
                let secs = morse::duration_seconds(text, self.status.cw_wpm).unwrap_or(text.len() as f64 * 0.5);
                self.tx_ticks_left = (secs / POLL_SECONDS).ceil().max(1.0) as u32;
                if !self.status.transmitting {
                    self.status.transmitting = true;
                    self.pending.push(RigEvent::Tx(true));
                }
            }
            RadioCommand::StopCw => {
                self.tx_ticks_left = 0;
                if self.status.transmitting {
                    self.status.transmitting = false;
                    self.pending.push(RigEvent::Tx(false));
                }
            }
            RadioCommand::SetPtt(on) | RadioCommand::KeyDown(on) => {
                if self.status.transmitting != *on {
                    self.status.transmitting = *on;
                    self.pending.push(RigEvent::Tx(*on));
                }
            }
            RadioCommand::Raw(_) => {}
            RadioCommand::SetScope(on) => self.scope = *on,
            RadioCommand::SetAfLevel(v) => self.audio.af_level = Some((*v).min(255)),
            RadioCommand::SetBreakIn(v) => self.audio.break_in = Some((*v).min(2)),
            RadioCommand::SetSquelch(v) => self.audio.squelch = Some((*v).min(255)),
            RadioCommand::SetNotch(on) => self.audio.notch = Some(*on),
            RadioCommand::SetAutoNotch(on) => self.audio.auto_notch = Some(*on),
            RadioCommand::CenterPbt => {
                self.audio.pbt_inner = Some(crate::PBT_CENTER);
                self.audio.pbt_outer = Some(crate::PBT_CENTER);
            }
            RadioCommand::SetSplit(tx_hz) => {
                self.status.split = SplitState { on: true, tx_hz: Some(*tx_hz) };
                self.pending.push(RigEvent::Split(self.status.split));
            }
            RadioCommand::SplitOff => {
                if self.status.split.on {
                    self.status.split = SplitState::default();
                    self.pending.push(RigEvent::Split(self.status.split));
                }
            }
        }
        Ok(())
    }

    fn audio_controls(&self) -> crate::AudioControls {
        self.audio
    }

    fn poll(&mut self) -> Vec<RigEvent> {
        if self.tx_ticks_left > 0 {
            self.tx_ticks_left -= 1;
            if self.tx_ticks_left == 0 && self.status.transmitting {
                self.status.transmitting = false;
                self.pending.push(RigEvent::Tx(false));
            }
        }
        // A gently wandering S-meter.
        let r = self.next_rand();
        let step = (r % 7) as i32 - 3;
        let next = (i32::from(self.status.s_meter) + step).clamp(20, 90) as u8;
        if next != self.status.s_meter {
            self.status.s_meter = next;
            self.pending.push(RigEvent::SMeter(next));
        }
        std::mem::take(&mut self.pending)
    }

    fn status(&self) -> RadioStatus {
        self.status.clone()
    }

    fn fast_poll(&mut self) -> Vec<RigEvent> {
        let mut out = Vec::new();
        if self.status.transmitting {
            out.push(RigEvent::Swr(self.swr));
        }
        if self.scope {
            out.push(RigEvent::Scope(self.scope_sweep()));
        }
        out
    }

    fn simulate_swr(&mut self, swr: f32) {
        self.swr = swr;
    }

    fn simulate_knob(&mut self, hz: u64) {
        self.turn_knob(hz);
    }
}
