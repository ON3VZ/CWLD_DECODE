//! Radio-independent commands, events and status.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RadioMode {
    Cw,
    CwReverse,
    Usb,
    Lsb,
    Am,
    Fm,
    Fsk,
    Other,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RadioStatus {
    pub connected: bool,
    pub frequency_hz: u64,
    pub mode: RadioMode,
    pub filter: Option<u8>,
    pub cw_wpm: u8,
    pub transmitting: bool,
    /// Normalised S-meter, 0 (S0) to 100 (S9+60).
    pub s_meter: u8,
}

/// Unsolicited or polled changes reported by a radio driver.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum RigEvent {
    Connected(bool),
    Frequency(u64),
    Mode(RadioMode),
    Tx(bool),
    SMeter(u8),
    CwSpeed(u8),
    /// Text keyed with the paddle on the radio, decoded by the radio. Pieces, not whole lines.
    PaddleText(String),
    /// Chunks still waiting to be sent to the radio (not the one being sent).
    Queue(Vec<String>),
    /// SWR reading while the radio transmits. Every reading is reported, not only changes.
    Swr(f32),
    /// Something the operator should know. A stable code, the interface translates it.
    Warning(String),
    /// One sweep of the radio's own spectrum scope (band scope).
    Scope(ScopeSweep),
}

/// One sweep of a radio's spectrum scope: the level at evenly spaced points from `lo_hz` to
/// `hi_hz`. On an Icom the values run from 0 to 160.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeSweep {
    pub lo_hz: u64,
    pub hi_hz: u64,
    /// True in centre mode (the scope follows the VFO), false for fixed edges.
    pub centered: bool,
    /// The radio says the data is out of range (no picture this time).
    pub out_of_range: bool,
    pub data: Vec<u8>,
}

/// Everything CWLD can ask a radio to do. There is no other way to reach a radio driver.
#[derive(Clone, Debug, PartialEq)]
pub enum RadioCommand {
    ReadStatus,
    SetFrequency(u64),
    SetMode(RadioMode),
    SetCwSpeed(u8),
    SendCw(String),
    StopCw,
    SetPtt(bool),
    KeyDown(bool),
    /// Raw bytes for diagnostics. Treated as transmit-capable: default deny.
    Raw(Vec<u8>),
    /// Switches the output of the radio's band scope to the app on or off. Receive only.
    SetScope(bool),
    /// Volume of the radio's own speaker and headphones, 0 to 255 (Icom AF level).
    SetAfLevel(u16),
    /// Break-in: 0 off, 1 semi, 2 full. Off means the paddle only makes the sidetone. Setting it
    /// does not key the radio by itself.
    SetBreakIn(u8),
    /// Squelch level 0 to 255 (Icom SQL). Fully closed silences the band in the headphones while
    /// the sidetone and the USB audio stay. Receive only.
    SetSquelch(u16),
}

/// Safety class of a command. Outside `TxPolicy::Live` only the first three are allowed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandClass {
    /// Reads state, changes nothing.
    ReadOnly,
    /// Changes a setting that cannot cause a transmission.
    Config,
    /// Stops a transmission. Always allowed, in every policy.
    Stop,
    /// Can cause the radio to transmit. Only allowed in `Live`.
    Transmit,
}

impl RadioCommand {
    /// The class of this command. This `match` has no wildcard on purpose: adding a new
    /// command forces the author to decide its safety class (compile error otherwise).
    pub fn class(&self) -> CommandClass {
        match self {
            RadioCommand::ReadStatus => CommandClass::ReadOnly,
            RadioCommand::SetFrequency(_) => CommandClass::Config,
            RadioCommand::SetMode(_) => CommandClass::Config,
            RadioCommand::SetCwSpeed(_) => CommandClass::Config,
            RadioCommand::StopCw => CommandClass::Stop,
            RadioCommand::SendCw(_) => CommandClass::Transmit,
            RadioCommand::SetPtt(_) => CommandClass::Transmit,
            RadioCommand::KeyDown(_) => CommandClass::Transmit,
            RadioCommand::Raw(_) => CommandClass::Transmit,
            RadioCommand::SetScope(_) => CommandClass::Config,
            RadioCommand::SetAfLevel(_) => CommandClass::Config,
            RadioCommand::SetBreakIn(_) => CommandClass::Config,
            RadioCommand::SetSquelch(_) => CommandClass::Config,
        }
    }

    /// One example of every variant, used by tests to cover the whole command set.
    pub fn examples() -> Vec<RadioCommand> {
        vec![
            RadioCommand::ReadStatus,
            RadioCommand::SetFrequency(7_030_000),
            RadioCommand::SetMode(RadioMode::Cw),
            RadioCommand::SetCwSpeed(20),
            RadioCommand::SendCw("CQ".to_string()),
            RadioCommand::StopCw,
            RadioCommand::SetPtt(true),
            RadioCommand::KeyDown(true),
            RadioCommand::Raw(vec![0xFE, 0xFE]),
            RadioCommand::SetScope(true),
            RadioCommand::SetAfLevel(0),
            RadioCommand::SetBreakIn(0),
            RadioCommand::SetSquelch(255),
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            RadioCommand::ReadStatus => "read_status",
            RadioCommand::SetFrequency(_) => "set_frequency",
            RadioCommand::SetMode(_) => "set_mode",
            RadioCommand::SetCwSpeed(_) => "set_cw_speed",
            RadioCommand::SendCw(_) => "send_cw",
            RadioCommand::StopCw => "stop_cw",
            RadioCommand::SetPtt(_) => "set_ptt",
            RadioCommand::KeyDown(_) => "key_down",
            RadioCommand::Raw(_) => "raw",
            RadioCommand::SetScope(_) => "set_scope",
            RadioCommand::SetAfLevel(_) => "set_af_level",
            RadioCommand::SetBreakIn(_) => "set_break_in",
            RadioCommand::SetSquelch(_) => "set_squelch",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_are_as_designed() {
        use CommandClass::*;
        let expected = [
            ("read_status", ReadOnly),
            ("set_frequency", Config),
            ("set_mode", Config),
            ("set_cw_speed", Config),
            ("send_cw", Transmit),
            ("stop_cw", Stop),
            ("set_ptt", Transmit),
            ("key_down", Transmit),
            ("raw", Transmit),
            ("set_scope", Config),
            ("set_af_level", Config),
            ("set_break_in", Config),
            ("set_squelch", Config),
        ];
        let examples = RadioCommand::examples();
        assert_eq!(examples.len(), expected.len());
        for (cmd, (name, class)) in examples.iter().zip(expected) {
            assert_eq!(cmd.name(), name);
            assert_eq!(cmd.class(), class, "{name}");
        }
    }
}

/// How sure the decoder is, for the meter next to the S-meter.
///
/// `confidence` is an **estimate** from the decoder's own character probabilities. Only in
/// practice mode, where the keyed text is known, `accuracy` holds the real share of correct
/// characters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecoderStats {
    /// 0.0 to 1.0, mean character confidence over the last stretch of text.
    pub confidence: f32,
    /// Signal to noise ratio in the decoder filter, when a signal is present.
    pub snr_db: Option<f32>,
    pub accuracy: Option<f32>,
    /// The bandwidth the decoder filter has now, in Hz (the settings of the filter, not measured).
    pub bandwidth_hz: Option<f32>,
}

/// Number of characters the confidence meter looks back over.
pub const CONFIDENCE_WINDOW: usize = 30;

/// Mean of the last `CONFIDENCE_WINDOW` character confidences. `None` without any text.
pub fn mean_confidence(conf: &[f32]) -> Option<f32> {
    let tail = &conf[conf.len().saturating_sub(CONFIDENCE_WINDOW)..];
    if tail.is_empty() {
        return None;
    }
    Some((tail.iter().sum::<f32>() / tail.len() as f32).clamp(0.0, 1.0))
}

#[cfg(test)]
mod stats_tests {
    use super::*;

    #[test]
    fn mean_looks_only_at_the_last_window() {
        assert_eq!(mean_confidence(&[]), None);
        let mut v = vec![0.0f32; 50];
        v.extend(vec![1.0f32; CONFIDENCE_WINDOW]);
        assert_eq!(mean_confidence(&v), Some(1.0));
        assert_eq!(mean_confidence(&[0.5, 1.0]), Some(0.75));
    }
}
