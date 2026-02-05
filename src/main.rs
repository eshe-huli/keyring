#![allow(dead_code)]
//! Keyring — Distributed Agent Mesh Runtime
//!
//! One binary. Zero external dependencies. Zero config to start.
//!
//! Architecture:
//!   API Layer → Coordinator → CRDT Engine → Sync Engine → Content Store
//!   + Plugin Runtime (WASM)
//!
//! See ARCHITECTURE.md for the full 50-iteration design.

mod api;
mod cli;
mod config;
mod coordinator;
mod crdt;
mod identity;
mod plugin;
mod store;
mod sync;
mod transport;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

use cli::{Cli, Commands};
use config::Config;
use identity::NodeIdentity;
use store::ContentStore;
use transport::Transport; // Import trait for method resolution

fn init_logging(verbose: bool) {
    let filter = if verbose {
        EnvFilter::new("keyring=debug,info")
    } else {
        EnvFilter::new("keyring=info,warn")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn data_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "keyring")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".keyring")
        })
}

fn identity_path() -> PathBuf {
    data_dir().join("identity.bin")
}

fn config_path() -> PathBuf {
    data_dir().join("config.toml")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_logging(cli.verbose);

    match cli.command {
        Commands::Init { name } => cmd_init(&name)?,
        Commands::Listen { addr } => cmd_listen(&addr).await?,
        Commands::Connect { peer } => cmd_connect(&peer).await?,
        Commands::Status => cmd_status()?,
        Commands::Peers => cmd_peers()?,
        Commands::Ring { action } => cmd_ring(action)?,
        Commands::Document { action } => cmd_document(action)?,
        Commands::Sync { action } => cmd_sync(action)?,
        Commands::Task { action } => cmd_task(action)?,
        Commands::Plugin { action } => cmd_plugin(action)?,
    }

    Ok(())
}

/// Initialize a new node
fn cmd_init(name: &str) -> Result<()> {
    let id_path = identity_path();

    if id_path.exists() {
        let existing = NodeIdentity::load(&id_path)?;
        println!("Node already initialized:");
        println!("  ID:   {}", existing.node_id());
        println!("  Name: {}", existing.name());
        println!("\nTo reinitialize, delete: {}", id_path.display());
        return Ok(());
    }

    // Generate identity
    let identity = NodeIdentity::generate(name.to_string());
    identity.save(&id_path)?;

    // Create config
    let config = Config::default_for(name);
    config.save(&config_path())?;

    // Initialize content store
    let _store = ContentStore::open(&data_dir())?;

    println!("✓ Node initialized");
    println!("  ID:     {}", identity.node_id());
    println!("  Name:   {}", identity.name());
    println!("  Data:   {}", data_dir().display());
    println!("  Config: {}", config_path().display());
    println!("\nNext: keyring listen --addr 0.0.0.0:4433");

    Ok(())
}

/// Start listening for connections
async fn cmd_listen(addr: &str) -> Result<()> {
    let id_path = identity_path();
    if !id_path.exists() {
        anyhow::bail!("Node not initialized. Run: keyring init --name <name>");
    }

    let identity = NodeIdentity::load(&id_path)?;
    let _store = ContentStore::open(&data_dir())?;

    println!("🔑 Keyring node starting");
    println!("  ID:   {}", identity.node_id());
    println!("  Name: {}", identity.name());
    println!("  Addr: {}", addr);

    // Start QUIC transport
    let quic = transport::QuicTransport::new();
    let listener: Box<dyn transport::Listener> = quic.listen(addr).await?;

    println!("  QUIC: listening on {}", listener.local_addr()?);
    println!("\nWaiting for connections... (Ctrl+C to stop)");

    // Accept loop
    loop {
        match listener.accept().await {
            Ok(conn) => {
                let peer = conn.peer_addr();
                tracing::info!("Peer connected: {} via {}", peer.addr, peer.transport);
                // TODO: Spawn sync task for this connection
            }
            Err(e) => {
                tracing::warn!("Accept error: {}", e);
            }
        }
    }
}

/// Connect to a remote peer
async fn cmd_connect(peer: &str) -> Result<()> {
    let id_path = identity_path();
    if !id_path.exists() {
        anyhow::bail!("Node not initialized. Run: keyring init --name <name>");
    }

    let identity = NodeIdentity::load(&id_path)?;
    println!("Connecting to {} as {}", peer, identity.node_id().short());

    let quic = transport::QuicTransport::new();
    let conn: Box<dyn transport::Connection> = quic.connect(peer).await?;

    println!("✓ Connected to {}", conn.peer_addr().addr);

    // TODO: Start sync protocol
    conn.close().await?;
    Ok(())
}

/// Show node status
fn cmd_status() -> Result<()> {
    let id_path = identity_path();
    if !id_path.exists() {
        println!("Node not initialized. Run: keyring init --name <name>");
        return Ok(());
    }

    let identity = NodeIdentity::load(&id_path)?;
    let store = ContentStore::open(&data_dir())?;
    let system_docs = store.list_documents(&identity::KeyringId::system())?;

    println!("🔑 Keyring Status");
    println!("  Node ID:  {}", identity.node_id());
    println!("  Name:     {}", identity.name());
    println!("  Created:  {}", identity.created_at());
    println!("  Data dir: {}", data_dir().display());
    println!("  System docs: {}", system_docs.len());
    println!("\n  Transports:");
    println!("    ✓ quic     (implemented)");
    println!("    ○ tcp      (stub)");
    println!("    ○ tailscale (stub)");
    println!("    ○ mdns     (stub)");
    println!("    ○ ws       (stub)");
    println!("    ○ nats     (stub)");
    println!("    ○ bt       (stub)");
    println!("    ○ file     (stub)");

    Ok(())
}

/// List peers
fn cmd_peers() -> Result<()> {
    println!("No connected peers. Start with: keyring listen");
    Ok(())
}

fn cmd_ring(action: cli::RingAction) -> Result<()> {
    match action {
        cli::RingAction::Create { name } => {
            let identity = NodeIdentity::load(&identity_path())?;
            let ring = identity::KeyringMembership::new(name.clone(), identity.node_id());
            println!("✓ Keyring created: {} ({})", name, ring.id);
        }
        cli::RingAction::List => {
            println!("Keyrings: (none yet)");
        }
        cli::RingAction::Invite { ring, node_id } => {
            println!("TODO: Invite {} to ring {}", node_id, ring);
        }
        cli::RingAction::Leave { ring } => {
            println!("TODO: Leave ring {}", ring);
        }
    }
    Ok(())
}

fn cmd_document(action: cli::DocAction) -> Result<()> {
    match action {
        cli::DocAction::Put { path, ring, tags } => {
            println!("TODO: Put {} into ring {} with tags {:?}", path, ring, tags);
        }
        cli::DocAction::Get { id } => {
            println!("TODO: Get document {}", id);
        }
        cli::DocAction::List { ring, doc_type } => {
            println!("TODO: List docs (ring={:?}, type={:?})", ring, doc_type);
        }
        cli::DocAction::Log { id } => {
            println!("TODO: Show log for {}", id);
        }
    }
    Ok(())
}

fn cmd_sync(action: Option<cli::SyncAction>) -> Result<()> {
    match action {
        None => println!("TODO: Force sync now"),
        Some(cli::SyncAction::Status) => println!("TODO: Sync status"),
        Some(cli::SyncAction::Watch) => println!("TODO: Watch sync events"),
    }
    Ok(())
}

fn cmd_task(action: cli::TaskAction) -> Result<()> {
    match action {
        cli::TaskAction::Submit { task_type, requires } => {
            println!("TODO: Submit task {} requiring {:?}", task_type, requires);
        }
        cli::TaskAction::List => println!("TODO: List tasks"),
        cli::TaskAction::Claim { id } => println!("TODO: Claim task {}", id),
    }
    Ok(())
}

fn cmd_plugin(action: cli::PluginAction) -> Result<()> {
    match action {
        cli::PluginAction::Install { path } => {
            println!("TODO: Install plugin from {}", path);
        }
        cli::PluginAction::List => {
            println!("Installed plugins: (none)");
        }
        cli::PluginAction::Remove { name } => {
            println!("TODO: Remove plugin {}", name);
        }
    }
    Ok(())
}
