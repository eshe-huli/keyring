# 🔑 Keyring

**Distributed agent mesh runtime.**

One binary. Zero external dependencies. Zero config to start.

```bash
# Node 1 (VPS)
keyring init --name onyx-key
keyring listen --addr 0.0.0.0:4433

# Node 2 (MacBook)
keyring init --name argus-key
keyring connect 38.242.159.237:4433
keyring ring create ben-personal
keyring ring invite onyx-key --ring ben-personal

# They're now syncing.
```

## What Is Keyring?

Keyring is a distributed runtime for autonomous agents. It provides the nervous system through which agents share state, coordinate work, synchronize data, and operate independently — online or offline, on any device, at any scale.

## The Six Laws

1. **Offline-first** — Any node must function fully disconnected
2. **Zero-config convergence** — Nodes reconnect and converge automatically
3. **No single point of failure** — No leader, no master, no central server
4. **Data sovereignty** — Each node owns its data. Sharing is explicit
5. **Transport agnostic** — QUIC, TCP, NATS, Bluetooth, USB stick
6. **Scale invariant** — Same binary for 2 nodes and 2000 nodes

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                  KEYRING NODE                        │
│              (Single Rust Binary)                    │
├─────────────────────────────────────────────────────┤
│  API Layer      │ gRPC │ HTTP │ Unix Socket │ CLI   │
├─────────────────────────────────────────────────────┤
│  Coordinator    │ Presence │ Tasks │ Scheduler       │
├─────────────────────────────────────────────────────┤
│  CRDT Engine    │ Automerge │ HLC │ Vector Clocks   │
├─────────────────────────────────────────────────────┤
│  Sync Engine    │ Merkle DAG │ Delta Sync │ Frames  │
├─────────────────────────────────────────────────────┤
│  Content Store  │ BLAKE3 │ redb │ S3 (optional)     │
├─────────────────────────────────────────────────────┤
│  Plugin Runtime │ WASM (wasmtime)                    │
├─────────────────────────────────────────────────────┤
│  Transports     │ ✓ QUIC │ ○ TCP │ ○ Tailscale │ … │
└─────────────────────────────────────────────────────┘
```

## Stack

| Crate | Purpose |
|-------|---------|
| `redb` | Local KV store (pure Rust, ACID) |
| `automerge` | CRDTs (conflict-free data types) |
| `blake3` | Content-addressed hashing |
| `ed25519-dalek` | Identity & signing |
| `quinn` | QUIC transport |
| `tonic` | gRPC API |
| `wasmtime` | WASM plugin sandbox |
| `ractor` | Actor model (Erlang-style supervision) |

Zero C dependencies in the critical path. Pure Rust. Cross-compiles to Linux, macOS, Windows, ARM.

## Commands

```bash
keyring init --name <name>          # Create node identity
keyring listen --addr 0.0.0.0:4433  # Start daemon
keyring connect <host:port>         # Connect to peer
keyring status                      # Show node info
keyring peers                       # List connected peers

keyring ring create <name>          # Create trust group
keyring ring invite --ring <name> <node-id>
keyring ring list

keyring doc put <file> --ring <name> --tags <t1,t2>
keyring doc get <id>
keyring doc list --ring <name>

keyring sync                        # Force sync
keyring sync status
keyring sync watch                  # Live sync events

keyring task submit --type <type> --requires <cap1,cap2>
keyring task list
keyring task claim <id>

keyring plugin install <file.wasm>
keyring plugin list
```

## Transports

| Transport | Status | Description |
|-----------|--------|-------------|
| QUIC | ✅ Implemented | Primary mesh communication |
| TCP | ○ Stub | Fallback, LAN |
| Tailscale | ○ Stub | WireGuard mesh VPN |
| mDNS | ○ Stub | LAN auto-discovery |
| WebSocket | ○ Stub | Browser/mobile nodes |
| NATS | ○ Stub | Pub/sub fan-out |
| Bluetooth | ○ Stub | Proximity sync |
| File | ○ Stub | Sneakernet/USB |

## Building

```bash
cargo build --release
```

Binary output: `target/release/keyring`

## License

Apache-2.0

## Authors

Built by the Keyring team.
