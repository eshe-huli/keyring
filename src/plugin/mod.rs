//! Plugin runtime — WASM sandboxed extensions
//!
//! Plugin types: Transform, Hook, Resolver, Transport, Storage, Router

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Plugin manifest
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub permissions: HashSet<Permission>,
    pub triggers: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginType {
    /// Process data on write/read (encryption, compression)
    Transform,
    /// React to events (on_sync, on_change)
    Hook,
    /// Custom conflict resolution
    Resolver,
    /// Custom sync transport
    Transport,
    /// Custom storage backend
    Storage,
    /// Custom task routing
    Router,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    ReadDocument,
    WriteDocument,
    Network,
    FileSystem,
    Subprocess,
    None,
}

/// Plugin runtime — manages WASM plugin lifecycle
pub struct PluginRuntime {
    plugins: Vec<LoadedPlugin>,
}

struct LoadedPlugin {
    manifest: PluginManifest,
    // Future: wasmtime::Instance
}

impl PluginRuntime {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    /// Install a plugin from WASM bytes
    pub fn install(&mut self, manifest: PluginManifest, _wasm_bytes: &[u8]) -> anyhow::Result<()> {
        tracing::info!("Installing plugin: {} v{}", manifest.name, manifest.version);
        // TODO: Compile WASM module with wasmtime, validate permissions
        self.plugins.push(LoadedPlugin { manifest });
        Ok(())
    }

    /// List installed plugins
    pub fn list(&self) -> Vec<&PluginManifest> {
        self.plugins.iter().map(|p| &p.manifest).collect()
    }

    /// Uninstall a plugin
    pub fn uninstall(&mut self, name: &str) -> bool {
        let before = self.plugins.len();
        self.plugins.retain(|p| p.manifest.name != name);
        self.plugins.len() < before
    }
}
