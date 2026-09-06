//! Durable family milestones that survive saves and replay deterministically.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FamilyState {
    #[serde(default)]
    pub pregnancies: Vec<Pregnancy>,
    #[serde(default)]
    pub marriages: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pregnancy {
    pub mother: String,
    pub due_day: u32,
}
