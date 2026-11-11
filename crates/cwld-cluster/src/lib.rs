//! DX cluster client of CWLD. It reads spots from a cluster node over Telnet and never sends any.

mod client;
mod spot;
mod telnet;

pub use client::{probe, probe_all, Client, Config, Event, Node, Probe};
pub use spot::{detect_software, in_cw_segment, init_commands, parse_spot, valid_call, Software, Spot, SpotMode};
