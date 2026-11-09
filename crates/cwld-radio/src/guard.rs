use crate::error::RigError;
use crate::rig::{Capabilities, Rig, RigInfo};
use cwld_types::{CommandClass, PolicyHandle, RadioCommand, RadioStatus, RigEvent};

/// Valid keyer speeds. Speed 0 is refused on purpose: on a QRP Labs QMX `KS00;`
/// switches the radio to straight-key mode.
pub const MIN_WPM: u8 = 5;
pub const MAX_WPM: u8 = 60;

/// Wraps a radio driver and enforces the transmit policy.
///
/// Outside `Live` only commands of class `ReadOnly`, `Config` and `Stop` get through.
/// This is a whitelist: a command that is not explicitly safe is refused.
pub struct GuardedRig<R: Rig> {
    inner: R,
    policy: PolicyHandle,
    blocked: u64,
    passed_transmit: u64,
}

impl<R: Rig> GuardedRig<R> {
    pub fn new(inner: R, policy: PolicyHandle) -> Self {
        GuardedRig { inner, policy, blocked: 0, passed_transmit: 0 }
    }

    /// How many commands were refused so far (shown in the diagnostics screen).
    pub fn blocked_count(&self) -> u64 {
        self.blocked
    }

    /// How many commands of class `Transmit` were passed on to the radio.
    pub fn passed_transmit_count(&self) -> u64 {
        self.passed_transmit
    }

    pub fn inner(&self) -> &R {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn policy(&self) -> PolicyHandle {
        self.policy.clone()
    }
}

impl<R: Rig> Rig for GuardedRig<R> {
    fn info(&self) -> RigInfo {
        self.inner.info()
    }

    fn capabilities(&self) -> Capabilities {
        self.inner.capabilities()
    }

    fn execute(&mut self, cmd: &RadioCommand) -> Result<(), RigError> {
        let policy = self.policy.get();
        if cmd.class() == CommandClass::Transmit && !policy.may_transmit() {
            self.blocked += 1;
            return Err(RigError::Blocked { command: cmd.name(), policy });
        }
        if let RadioCommand::SetCwSpeed(wpm) = cmd {
            if !(MIN_WPM..=MAX_WPM).contains(wpm) {
                return Err(RigError::InvalidArgument(format!(
                    "speed must be {MIN_WPM} to {MAX_WPM} WPM, got {wpm}"
                )));
            }
        }
        if let RadioCommand::SendCw(text) = cmd {
            if text.trim().is_empty() {
                return Err(RigError::InvalidArgument("empty text".into()));
            }
        }
        let result = self.inner.execute(cmd);
        if cmd.class() == CommandClass::Transmit {
            self.passed_transmit += 1;
        }
        result
    }

    fn poll(&mut self) -> Vec<RigEvent> {
        self.inner.poll()
    }

    fn status(&self) -> RadioStatus {
        self.inner.status()
    }

    fn fast_poll(&mut self) -> Vec<RigEvent> {
        self.inner.fast_poll()
    }

    fn simulate_swr(&mut self, swr: f32) {
        self.inner.simulate_swr(swr);
    }

    fn audio_controls(&self) -> crate::AudioControls {
        self.inner.audio_controls()
    }

    fn simulate_knob(&mut self, hz: u64) {
        self.inner.simulate_knob(hz);
    }
}
