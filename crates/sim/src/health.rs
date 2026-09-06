use crate::party::{Relationships, Skills, Trait};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyMember {
    pub name: String,
    pub health: u8,
    pub morale: i16,
    pub ailments: Vec<String>,
    pub alive: bool,
    #[serde(default = "default_age")]
    pub age: u8,
    #[serde(default)]
    pub traits: Vec<Trait>,
    #[serde(default)]
    pub skills: Skills,
    #[serde(default)]
    pub relationships: Relationships,
    #[serde(default)]
    pub ailment_days: BTreeMap<String, u16>,
}

impl PartyMember {
    pub fn new(name: String) -> Self {
        Self {
            name,
            health: 100,
            morale: 50,
            ailments: Vec::new(),
            alive: true,
            age: default_age(),
            traits: Vec::new(),
            skills: Skills::default(),
            relationships: Relationships::default(),
            ailment_days: BTreeMap::new(),
        }
    }
}
const fn default_age() -> u8 {
    30
}

pub fn advance(member: &mut PartyMember, daily_damage: u8) -> bool {
    if !member.alive {
        return false;
    }
    member.health = member.health.saturating_sub(daily_damage);
    if member.health == 0 {
        member.alive = false;
        return true;
    }
    false
}
