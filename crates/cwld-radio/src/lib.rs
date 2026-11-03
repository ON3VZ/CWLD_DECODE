//! Radio layer for CWLD.
//!
//! * [`Rig`] is the trait every radio driver implements.
//! * [`GuardedRig`] wraps any driver and enforces the transmit policy.
//! * [`SimRig`] is a simulated radio, used for tests and for working without a radio.

pub mod civ;
pub mod clock;
mod error;
mod guard;
mod icom;
mod null;
pub mod kenwood;
pub mod keyer;
pub mod ports;
mod rig;
mod sim;
pub mod transport;

pub use error::RigError;
pub use clock::{Clock, ManualClock, SystemClock};
pub use guard::GuardedRig;
pub use icom::IcomRig;
pub use kenwood::QmxRig;
pub use null::NullRig;
pub use ports::{list_ports, PortInfo};
pub use rig::{Capabilities, CwTextMode, Rig, RigInfo};
pub use sim::SimRig;
pub use transport::{MockHandle, MockTransport, SerialTransport, Transport};
