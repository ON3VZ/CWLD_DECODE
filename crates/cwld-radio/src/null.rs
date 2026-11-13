//! The "no radio" driver: shown while nothing is connected. Everything fails politely,
//! except stop, which is always fine.

use crate::error::RigError;
use crate::rig::{Capabilities, CwTextMode, Rig, RigInfo};
use cwld_types::{RadioCommand, RadioMode, RadioStatus, RigEvent};

pub struct NullRig {
    announced: bool,
}

impl NullRig {
    pub fn new() -> Self {
        NullRig { announced: false }
    }
}

impl Default for NullRig {
    fn default() -> Self {
        Self::new()
    }
}

impl Rig for NullRig {
    fn info(&self) -> RigInfo {
        RigInfo { name: String::new(), driver: "none" }
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities { cw_text: CwTextMode::None, radio_decoder_text: false, dtr_keying: false, pushes_status: false, swr_meter: false, scope: false }
    }

    fn execute(&mut self, cmd: &RadioCommand) -> Result<(), RigError> {
        match cmd {
            RadioCommand::StopCw => Ok(()),
            _ => Err(RigError::NotConnected),
        }
    }

    fn poll(&mut self) -> Vec<RigEvent> {
        if self.announced {
            Vec::new()
        } else {
            self.announced = true;
            vec![RigEvent::Connected(false)]
        }
    }

    fn status(&self) -> RadioStatus {
        RadioStatus {
            connected: false,
            frequency_hz: 0,
            mode: RadioMode::Cw,
            filter: None,
            cw_wpm: 20,
            transmitting: false,
            s_meter: 0,
            split: Default::default(),
        }
    }
}
