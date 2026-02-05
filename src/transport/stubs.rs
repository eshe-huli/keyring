//! Transport stubs — placeholders for future implementations
//!
//! Each stub satisfies the Transport trait but returns meaningful errors.
//! Pick one up and implement it when ready.

use anyhow::{Result, bail};
use async_trait::async_trait;

use super::types::*;

// ─── TCP Transport (stub) ───

pub struct TcpTransport;

#[async_trait]
impl Transport for TcpTransport {
    async fn listen(&self, _addr: &str) -> Result<Box<dyn Listener>> {
        bail!("TCP transport not yet implemented — use QUIC")
    }
    async fn connect(&self, _peer: &str) -> Result<Box<dyn Connection>> {
        bail!("TCP transport not yet implemented — use QUIC")
    }
    async fn discover(&self) -> Result<Vec<PeerAddr>> {
        Ok(vec![])
    }
    fn name(&self) -> &str { "tcp" }
}

// ─── Tailscale Transport (stub) ───

pub struct TailscaleTransport;

#[async_trait]
impl Transport for TailscaleTransport {
    async fn listen(&self, _addr: &str) -> Result<Box<dyn Listener>> {
        bail!("Tailscale transport not yet implemented")
    }
    async fn connect(&self, _peer: &str) -> Result<Box<dyn Connection>> {
        bail!("Tailscale transport not yet implemented")
    }
    async fn discover(&self) -> Result<Vec<PeerAddr>> {
        // Future: parse `tailscale status --json` to find peers
        bail!("Tailscale discovery not yet implemented")
    }
    fn name(&self) -> &str { "tailscale" }
}

// ─── mDNS Transport (stub) ───

pub struct MdnsTransport;

#[async_trait]
impl Transport for MdnsTransport {
    async fn listen(&self, _addr: &str) -> Result<Box<dyn Listener>> {
        bail!("mDNS transport not yet implemented")
    }
    async fn connect(&self, _peer: &str) -> Result<Box<dyn Connection>> {
        bail!("mDNS transport not yet implemented")
    }
    async fn discover(&self) -> Result<Vec<PeerAddr>> {
        // Future: broadcast _keyring._udp.local and collect responses
        bail!("mDNS discovery not yet implemented")
    }
    fn name(&self) -> &str { "mdns" }
}

// ─── WebSocket Transport (stub) ───

pub struct WsTransport;

#[async_trait]
impl Transport for WsTransport {
    async fn listen(&self, _addr: &str) -> Result<Box<dyn Listener>> {
        bail!("WebSocket transport not yet implemented")
    }
    async fn connect(&self, _peer: &str) -> Result<Box<dyn Connection>> {
        bail!("WebSocket transport not yet implemented")
    }
    async fn discover(&self) -> Result<Vec<PeerAddr>> {
        Ok(vec![])
    }
    fn name(&self) -> &str { "ws" }
}

// ─── NATS Transport (stub) ───

pub struct NatsTransport;

#[async_trait]
impl Transport for NatsTransport {
    async fn listen(&self, _addr: &str) -> Result<Box<dyn Listener>> {
        bail!("NATS transport not yet implemented")
    }
    async fn connect(&self, _peer: &str) -> Result<Box<dyn Connection>> {
        bail!("NATS transport not yet implemented")
    }
    async fn discover(&self) -> Result<Vec<PeerAddr>> {
        bail!("NATS discovery not yet implemented")
    }
    fn name(&self) -> &str { "nats" }
}

// ─── Bluetooth Transport (stub) ───

pub struct BtTransport;

#[async_trait]
impl Transport for BtTransport {
    async fn listen(&self, _addr: &str) -> Result<Box<dyn Listener>> {
        bail!("Bluetooth transport not yet implemented")
    }
    async fn connect(&self, _peer: &str) -> Result<Box<dyn Connection>> {
        bail!("Bluetooth transport not yet implemented")
    }
    async fn discover(&self) -> Result<Vec<PeerAddr>> {
        bail!("Bluetooth discovery not yet implemented")
    }
    fn name(&self) -> &str { "bt" }
}

// ─── File Transport (stub) — sneakernet ───

pub struct FileTransport;

#[async_trait]
impl Transport for FileTransport {
    async fn listen(&self, _addr: &str) -> Result<Box<dyn Listener>> {
        bail!("File transport not yet implemented — future: watch directory for sync frames")
    }
    async fn connect(&self, _peer: &str) -> Result<Box<dyn Connection>> {
        bail!("File transport not yet implemented — future: write sync frames to directory")
    }
    async fn discover(&self) -> Result<Vec<PeerAddr>> {
        Ok(vec![])
    }
    fn name(&self) -> &str { "file" }
}
