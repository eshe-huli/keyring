//! Coordinator — presence, task routing, scheduling, health
//!
//! Key insight: coordination state is just documents in the __system__ keyring.
//! No separate service needed.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::identity::{KeyringId, NodeId};
use crate::crdt::HLC;

/// Node presence information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Presence {
    pub node_id: NodeId,
    pub status: NodeStatus,
    pub capabilities: HashSet<String>,
    pub resources: Resources,
    pub last_seen: HLC,
    pub heartbeat: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Offline,
    Busy,
    Away,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resources {
    pub cpu_usage: f32,
    pub memory_gb: f32,
    pub disk_gb: f32,
}

/// A task for distributed execution
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: uuid::Uuid,
    pub task_type: String,
    pub requires: HashSet<String>,
    pub prefers: HashSet<String>,
    pub payload: Vec<u8>,
    pub priority: Priority,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub status: TaskStatus,
    pub created_by: NodeId,
    pub created_at: HLC,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Claimed(NodeId),
    Running(NodeId),
    Done(Vec<u8>),
    Failed(String),
}

/// Scheduled task template
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub id: uuid::Uuid,
    pub cron: String,
    pub task_type: String,
    pub requires: HashSet<String>,
    pub timezone: String,
    pub owner: KeyringId,
    pub enabled: bool,
}

/// Coordinator — reads/writes __system__ keyring for coordination
pub struct Coordinator {
    node_id: NodeId,
    presence_map: HashMap<NodeId, Presence>,
    tasks: HashMap<uuid::Uuid, Task>,
}

impl Coordinator {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            presence_map: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    /// Update our own presence
    pub fn heartbeat(&mut self, capabilities: HashSet<String>, resources: Resources) -> Presence {
        let presence = Presence {
            node_id: self.node_id,
            status: NodeStatus::Online,
            capabilities,
            resources,
            last_seen: HLC::now(self.node_id),
            heartbeat: self.presence_map
                .get(&self.node_id)
                .map(|p| p.heartbeat + 1)
                .unwrap_or(1),
        };
        self.presence_map.insert(self.node_id, presence.clone());
        presence
    }

    /// Route a task to the best available node
    pub fn route_task(&self, task: &Task) -> Option<NodeId> {
        let mut candidates: Vec<_> = self.presence_map.values()
            .filter(|p| p.status == NodeStatus::Online)
            .filter(|p| task.requires.is_subset(&p.capabilities))
            .collect();

        if candidates.is_empty() {
            return None;
        }

        // Score candidates
        candidates.sort_by(|a, b| {
            let score_a = self.score_node(a, task);
            let score_b = self.score_node(b, task);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Deterministic tiebreak: lowest NodeId wins
        Some(candidates[0].node_id)
    }

    fn score_node(&self, node: &Presence, task: &Task) -> f32 {
        let mut score = 0.0f32;

        // Preferred capabilities bonus
        let preferred_matches = task.prefers.intersection(&node.capabilities).count();
        score += preferred_matches as f32 * 10.0;

        // Resource availability
        score += (1.0 - node.resources.cpu_usage) * 5.0;
        score += node.resources.memory_gb;
        score += node.resources.disk_gb * 0.1;

        score
    }

    /// Submit a new task
    pub fn submit_task(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }

    /// Get online peers
    pub fn online_peers(&self) -> Vec<&Presence> {
        self.presence_map.values()
            .filter(|p| p.status == NodeStatus::Online)
            .collect()
    }
}
