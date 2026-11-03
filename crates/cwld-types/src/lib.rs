//! Shared types for CWLD.
//!
//! This crate has no knowledge of Tauri, audio or serial ports, so that every other
//! crate (and the later test tooling) can use it freely.

pub mod morse;
pub mod policy;
pub mod radio;
pub mod settings;

pub use policy::{PolicyHandle, TxPolicy};
pub use radio::{mean_confidence, CommandClass, DecoderStats, RadioCommand, RadioMode, RadioStatus, RigEvent, CONFIDENCE_WINDOW};
pub use settings::{Language, RadioKind, RadioSettings, Settings, Station, SwrSettings, Theme};
