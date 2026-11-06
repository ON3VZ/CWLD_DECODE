//! Audio for CWLD. The sound card callback only copies samples into a lock-free ring buffer.
//! A separate thread does everything else: resampling to 8 kHz, the waterfall, recording and
//! the decoder. The interface never waits for audio, and audio never waits for the interface.

pub mod device;
pub mod engine;
pub mod fft;
pub mod record;
pub mod resample;
pub mod ring;
pub mod waterfall;

pub use device::{list_inputs, InputDevice};
pub use engine::{Engine, EngineConfig, EngineEvent, Source, TentativeOut, Tuning};
