//! Transport layer — pluggable connectivity
//!
//! Day 1: QUIC (real), TCP/Tailscale/mDNS/WS/NATS/BT (stubs)

mod quic;
mod stubs;
mod types;

pub use quic::QuicTransport;
pub use types::{Transport, Connection, Listener};
