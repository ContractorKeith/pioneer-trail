//! Serializable contracts for UI-driven minigames.
//!
//! The simulation owns when a minigame may start and accepts its one result;
//! renderers use the supplied seed to run the interactive portion deterministically.
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
    /// Ammunition available when the session began. Rafting always has zero.
    pub ammo_available: u32,
}
