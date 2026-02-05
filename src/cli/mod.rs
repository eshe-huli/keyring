//! CLI — command-line interface for keyring

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "keyring", version, about = "Distributed agent mesh runtime")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Config file path
    #[arg(short, long, default_value = "~/.keyring/config.toml")]
    pub config: String,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new node identity and local store
    Init {
        /// Human-readable name for this node
        #[arg(long)]
        name: String,
    },

    /// Start the keyring daemon
    Listen {
        /// Address to listen on
        #[arg(long, default_value = "0.0.0.0:4433")]
        addr: String,
    },

    /// Connect to a remote peer
    Connect {
        /// Peer address (node-id@host:port or just host:port)
        peer: String,
    },

    /// Show node status, sync state, and peers
    Status,

    /// List connected peers
    Peers,

    /// Keyring (trust group) management
    Ring {
        #[command(subcommand)]
        action: RingAction,
    },

    /// Document operations
    #[command(alias = "doc")]
    Document {
        #[command(subcommand)]
        action: DocAction,
    },

    /// Sync operations
    Sync {
        #[command(subcommand)]
        action: Option<SyncAction>,
    },

    /// Task management
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },

    /// Plugin management
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
}

#[derive(Subcommand)]
pub enum RingAction {
    /// Create a new keyring
    Create { name: String },
    /// Invite a node to a keyring
    Invite {
        #[arg(long)]
        ring: String,
        node_id: String,
    },
    /// List keyrings
    List,
    /// Leave a keyring
    Leave { ring: String },
}

#[derive(Subcommand)]
pub enum DocAction {
    /// Store a file/document
    Put {
        path: String,
        #[arg(long)]
        ring: String,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// Retrieve a document
    Get { id: String },
    /// List documents
    List {
        #[arg(long)]
        ring: Option<String>,
        #[arg(long)]
        doc_type: Option<String>,
    },
    /// Show version history
    Log { id: String },
}

#[derive(Subcommand)]
pub enum SyncAction {
    /// Show sync status
    Status,
    /// Watch live sync events
    Watch,
}

#[derive(Subcommand)]
pub enum TaskAction {
    /// Submit a new task
    Submit {
        #[arg(long)]
        task_type: String,
        #[arg(long, value_delimiter = ',')]
        requires: Vec<String>,
    },
    /// List tasks
    List,
    /// Claim a task
    Claim { id: String },
}

#[derive(Subcommand)]
pub enum PluginAction {
    /// Install a WASM plugin
    Install { path: String },
    /// List installed plugins
    List,
    /// Uninstall a plugin
    Remove { name: String },
}
