//! Shared types for CWLD.
//!
//! This crate has no knowledge of Tauri, audio or serial ports, so that every other
//! crate (and the later test tooling) can use it freely.

pub mod morse;
pub mod policy;
pub mod radio;
pub mod settings;

pub use policy::{PolicyHandle, TxPolicy};
pub use radio::{mean_confidence, CommandClass, DecoderStats, RadioCommand, RadioMode, RadioStatus, RigEvent, ScopeSweep, CONFIDENCE_WINDOW};
pub use settings::{ContestProfile, ContestSettings, ContestVar, RecvField, SentTemplate, UserWord, DefaultRst, GAIN_MAX_DB, MAX_WPM_DEFAULT, TABS_AUTO_CLOSE_DEFAULT, TABS_AUTO_CLOSE_MAX, WPM_CEILING, WPM_FLOOR, SQUELCH_DEFAULT, SQUELCH_MAX, SQUELCH_MIN, AudioSettings, Language, LinkSettings, LookupSettings, PaddleProgress, CharStat, PaddleSession, RadioKind, RadioSettings, PracticeSettings, ScopeSize, Settings, Station, StationProfile, SwrSettings, Theme, default_profiles};
