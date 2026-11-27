use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MinigameKind {
    Hunt,
    Raft,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MinigameSession {
    pub kind: MinigameKind,
    pub seed: u64,
    pub ammo_available: u32,
}
