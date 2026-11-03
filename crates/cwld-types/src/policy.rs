//! The transmit policy: what CWLD is allowed to do towards the radio.
//!
//! * `Live`: may transmit, but only after a conscious action of the operator.
//! * `ListenOnly`: never transmits. The radio may be in any mode (even SSB).
//! * `Practice`: never transmits. Audio comes from a file or generator, sending is simulated.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TxPolicy {
    Live,
    #[default]
    ListenOnly,
    Practice,
}

impl TxPolicy {
    /// Only `Live` may send transmit commands to the radio.
    pub fn may_transmit(self) -> bool {
        matches!(self, TxPolicy::Live)
    }

    /// The policy to start with, given the one used last.
    /// CWLD never starts in `Live`: that always needs a conscious click.
    pub fn startup_from_last(last: TxPolicy) -> TxPolicy {
        match last {
            TxPolicy::Live => TxPolicy::ListenOnly,
            other => other,
        }
    }

    fn to_u8(self) -> u8 {
        match self {
            TxPolicy::Live => 0,
            TxPolicy::ListenOnly => 1,
            TxPolicy::Practice => 2,
        }
    }

    fn from_u8(v: u8) -> TxPolicy {
        match v {
            0 => TxPolicy::Live,
            2 => TxPolicy::Practice,
            // Unknown values fall back to the safe side.
            _ => TxPolicy::ListenOnly,
        }
    }
}

/// A shared, thread-safe handle on the current policy.
#[derive(Clone, Debug)]
pub struct PolicyHandle(Arc<AtomicU8>);

impl PolicyHandle {
    pub fn new(initial: TxPolicy) -> Self {
        PolicyHandle(Arc::new(AtomicU8::new(initial.to_u8())))
    }

    pub fn get(&self) -> TxPolicy {
        TxPolicy::from_u8(self.0.load(Ordering::SeqCst))
    }

    pub fn set(&self, policy: TxPolicy) {
        self.0.store(policy.to_u8(), Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_safe() {
        assert_eq!(TxPolicy::default(), TxPolicy::ListenOnly);
        assert!(!TxPolicy::default().may_transmit());
    }

    #[test]
    fn never_starts_in_live() {
        assert_eq!(TxPolicy::startup_from_last(TxPolicy::Live), TxPolicy::ListenOnly);
        assert_eq!(TxPolicy::startup_from_last(TxPolicy::Practice), TxPolicy::Practice);
        assert_eq!(TxPolicy::startup_from_last(TxPolicy::ListenOnly), TxPolicy::ListenOnly);
    }

    #[test]
    fn handle_is_shared_between_clones() {
        let a = PolicyHandle::new(TxPolicy::ListenOnly);
        let b = a.clone();
        b.set(TxPolicy::Live);
        assert_eq!(a.get(), TxPolicy::Live);
        a.set(TxPolicy::Practice);
        assert_eq!(b.get(), TxPolicy::Practice);
    }

    #[test]
    fn unknown_stored_value_is_safe() {
        assert_eq!(TxPolicy::from_u8(99), TxPolicy::ListenOnly);
    }

    #[test]
    fn serde_names_are_stable() {
        assert_eq!(serde_json::to_string(&TxPolicy::ListenOnly).unwrap(), "\"listen_only\"");
        assert_eq!(serde_json::from_str::<TxPolicy>("\"practice\"").unwrap(), TxPolicy::Practice);
    }
}
