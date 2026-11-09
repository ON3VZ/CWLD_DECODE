//! Signal processing for CWLD. No Tauri, no audio devices: plain functions on sample slices,
//! so the test bench and the command line tools can use them too.

pub mod band;
pub mod clean;
pub mod frontend;
pub mod gen;
pub mod rng;
pub mod seek;
pub mod wav;

pub use frontend::{Frontend, FrontendConfig, FRAME_RATE};
pub use band::{BandSignal, PracticeBand};
pub use gen::{generate, CwSpec};
pub use rng::Rng;
