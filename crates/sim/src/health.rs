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
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AilmentStage {
    Symptoms,
    Acute,
    Recovering,
}
pub fn stage(days: u16, severity: u8) -> AilmentStage {
    if days <= 1 {
        AilmentStage::Symptoms
    } else if days < u16::from(severity.max(2)) * 2 {
        AilmentStage::Acute
    } else {
        AilmentStage::Recovering
    }
}
pub fn contagious(id: &str) -> bool {
    matches!(id, "measles" | "cholera" | "whooping_cough")
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stages_progress() {
        assert_eq!(stage(1, 4), AilmentStage::Symptoms);
        assert_eq!(stage(3, 4), AilmentStage::Acute);
        assert_eq!(stage(8, 4), AilmentStage::Recovering);
    }
    #[test]
    fn only_named_diseases_are_contagious() {
        assert!(contagious("measles"));
        assert!(contagious("cholera"));
        assert!(contagious("whooping_cough"));
        assert!(!contagious("broken_leg"));
        assert!(!contagious("snakebite"));
    }
    #[test]
    fn dead_member_never_recovers() {
        let mut member = PartyMember::new("A".into());
        assert!(advance(&mut member, 100));
        assert!(!member.alive);
        assert!(!advance(&mut member, 0));
        assert_eq!(member.health, 0);
    }
}
