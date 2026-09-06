use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartyMember {
    pub name: String,
    pub health: u8,
    pub morale: i16,
    pub ailments: Vec<String>,
    pub alive: bool,
}

impl PartyMember {
    pub fn new(name: String) -> Self {
        Self { name, health: 100, morale: 50, ailments: Vec::new(), alive: true }
    }
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
