use crate::error::RigError;
use crate::rig::{Capabilities, CwTextMode, Rig, RigInfo};
use cwld_types::{morse, CommandClass, RadioCommand, RadioMode, RadioStatus, RigEvent};

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
            },
            pending: vec![RigEvent::Connected(true)],
            received: Vec::new(),
            transmit_commands: 0,
            tx_ticks_left: 0,
            rng: 0x2545_F491_4F6C_DD1D,
            swr: 1.1,
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
        }
        Ok(())
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
        if self.status.transmitting {
            vec![RigEvent::Swr(self.swr)]
        } else {
            Vec::new()
        }
    }

    fn simulate_swr(&mut self, swr: f32) {
        self.swr = swr;
    }

    fn simulate_knob(&mut self, hz: u64) {
        self.turn_knob(hz);
    }
}
