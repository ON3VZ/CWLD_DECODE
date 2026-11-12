use crate::error::RigError;
use cwld_types::{RadioCommand, RadioStatus, RigEvent};

/// How a radio takes CW text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CwTextMode {
    /// The radio takes blocks of at most `max` characters (Icom CI-V 0x17: 30).
    Chunked { max: usize },
    /// The radio has a text buffer of `size` characters that can be topped up
    /// while sending (QRP Labs QMX `KY`: 80).
    Buffered { size: usize },
    /// The radio cannot send text.
    None,
}

/// What a radio can do. The keyer and the interface ask for these instead of looking
/// at model names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capabilities {
    pub cw_text: CwTextMode,
    /// The radio hands out the text of its own decoder (QMX `TB`).
    pub radio_decoder_text: bool,
    /// The radio can be keyed through a DTR line of its USB serial port.
    pub dtr_keying: bool,
    /// The radio reports changes by itself (CI-V transceive, QMX `AI2`).
    pub pushes_status: bool,
    /// The radio reports the SWR while it transmits.
    pub swr_meter: bool,
    /// The radio can send its own spectrum scope (band scope) to the app.
    pub scope: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RigInfo {
    pub name: String,
    pub driver: &'static str,
}

/// A radio driver. Everything CWLD does towards a radio goes through [`Rig::execute`],
/// so there is exactly one place where commands can be checked.
pub trait Rig: Send {
    fn info(&self) -> RigInfo;
    fn capabilities(&self) -> Capabilities;
    fn execute(&mut self, cmd: &RadioCommand) -> Result<(), RigError>;
    /// Called regularly (about every 500 ms). Returns what changed since the last call.
    fn poll(&mut self) -> Vec<RigEvent>;
    fn status(&self) -> RadioStatus;
    /// A cheap poll for safety readings (SWR), called about every 100 ms. Slow work stays in `poll`.
    fn fast_poll(&mut self) -> Vec<RigEvent> {
        Vec::new()
    }
    /// Test helper: sets the SWR the simulator reports. Only the simulator does anything.
    fn simulate_swr(&mut self, _swr: f32) {}
    /// Test helper: the operator turns the knob. Only the simulator does anything.
    fn simulate_knob(&mut self, _hz: u64) {}
    /// The speaker volume and break-in as last read from the radio, when the driver can read them.
    fn audio_controls(&self) -> AudioControls {
        AudioControls::default()
    }
}

/// Settings of the radio that practice mode changes and puts back afterwards.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct AudioControls {
    /// AF level 0 to 255. On the IC-7300 MK II it also sets how loud the sidetone is on the USB
    /// audio, so practice mode never turns it down (measured by the operator).
    pub af_level: Option<u16>,
    /// 0 off, 1 semi, 2 full.
    pub break_in: Option<u8>,
    /// Squelch 0 (open) to 255 (closed).
    pub squelch: Option<u16>,
    /// Manual notch (MN) and automatic notch (AN) on the radio. A notch on the CW pitch cuts
    /// the wanted station out of the audio.
    pub notch: Option<bool>,
    pub auto_notch: Option<bool>,
    /// Twin PBT, inner and outer knob, 0 to 255; 128 is the middle (filter not shifted).
    pub pbt_inner: Option<u16>,
    pub pbt_outer: Option<u16>,
}

/// The value of a Twin PBT knob in the middle.
pub const PBT_CENTER: u16 = 128;

impl Rig for Box<dyn Rig> {
    fn info(&self) -> RigInfo {
        (**self).info()
    }
    fn capabilities(&self) -> Capabilities {
        (**self).capabilities()
    }
    fn execute(&mut self, cmd: &RadioCommand) -> Result<(), RigError> {
        (**self).execute(cmd)
    }
    fn poll(&mut self) -> Vec<RigEvent> {
        (**self).poll()
    }
    fn status(&self) -> RadioStatus {
        (**self).status()
    }
    fn fast_poll(&mut self) -> Vec<RigEvent> {
        (**self).fast_poll()
    }
    fn simulate_swr(&mut self, swr: f32) {
        (**self).simulate_swr(swr)
    }
    fn simulate_knob(&mut self, hz: u64) {
        (**self).simulate_knob(hz)
    }
    fn audio_controls(&self) -> AudioControls {
        (**self).audio_controls()
    }
}
