use crate::{
    health::{PartyMember, Sex},
    rng::SimRng,
};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trait {
    Hardy,
    Sickly,
    Cheerful,
    Grumbler,
    Sharpshooter,
    Herbalist,
    Devout,
    Restless,
    Practical,
    Gentle,
    Stoic,
    Sociable,
    Cautious,
    Daring,
    Patient,
    Impulsive,
    Handy,
    Animalwise,
    Meticulous,
    Adaptable,
}

impl Trait {
    pub const ALL: [Self; 20] = [
        Self::Hardy,
        Self::Sickly,
        Self::Cheerful,
        Self::Grumbler,
        Self::Sharpshooter,
        Self::Herbalist,
        Self::Devout,
        Self::Restless,
        Self::Practical,
        Self::Gentle,
        Self::Stoic,
        Self::Sociable,
        Self::Cautious,
        Self::Daring,
        Self::Patient,
        Self::Impulsive,
        Self::Handy,
        Self::Animalwise,
        Self::Meticulous,
        Self::Adaptable,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Skills {
    pub hunting: u8,
    pub medicine: u8,
    pub repair: u8,
    pub animals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Relationships {
    pub affinity: BTreeMap<String, i16>,
}

/// Assign a reproducible, varied profile to a newly configured party.
///
/// The leader receives the occupation's strongest skill. Companions receive only the smaller
/// skill bonuses implied by their individual traits.
pub fn initialize(party: &mut [PartyMember], occupation: &str, rng: &mut SimRng) {
    if party.is_empty() {
        return;
    }

    let leader_traits = occupation_traits(occupation);
    let mut available = Trait::ALL.to_vec();
    available.retain(|trait_| !leader_traits.contains(trait_));
    available.shuffle(rng.stream("party-profile"));

    for (index, member) in party.iter_mut().enumerate() {
        member.age = rng.stream("party-profile").gen_range(18..=65);
        member.sex = if rng.stream("family").gen_bool(0.5) { Sex::Female } else { Sex::Male };
        member.traits =
            if index == 0 { leader_traits.to_vec() } else { next_pair(&mut available, rng) };
        member.skills = Skills::default();
        member.relationships = Relationships::default();
        for trait_ in &member.traits {
            add_skills(&mut member.skills, trait_skills(*trait_));
        }
        if index == 0 {
            add_skills(&mut member.skills, occupation_skills(occupation));
        }
        let morale = 52 + member.traits.iter().map(|trait_| starting_morale(*trait_)).sum::<i16>();
        member.morale = morale.clamp(35, 70);
    }

    initialize_relationships(party, rng);
}

/// Apply daily morale and rare social consequences. Messages are emitted only for social events.
pub fn daily(
    party: &mut [PartyMember],
    resting: bool,
    filling: bool,
    varied_food: bool,
    rng: &mut SimRng,
) -> Vec<String> {
    for member in party.iter_mut().filter(|member| member.alive) {
        let mut change = if filling { 1 } else { -2 };
        if varied_food {
            change += 1;
        } else if !filling {
            change -= 1;
        }
        if resting {
            change += 1;
        }
        for trait_ in &member.traits {
            change += daily_morale(*trait_, resting, filling, varied_food);
        }
        member.morale = (member.morale + change).clamp(0, 100);
    }

    let mut messages = Vec::new();
    for left in 0..party.len() {
        for right in (left + 1)..party.len() {
            if !party[left].alive || !party[right].alive || party[left].name == party[right].name {
                continue;
            }
            let goodwill = social_weight(&party[left].traits) + social_weight(&party[right].traits);
            let support_chance = (2 + goodwill.max(0) / 4).clamp(1, 8) as u32;
            let argument_chance = (2 + (-goodwill).max(0) / 4).clamp(1, 8) as u32;
            let roll = rng.stream("party-daily").gen_range(0..100);
            if roll < support_chance {
                let change = rng.stream("party-daily").gen_range(1..=3);
                adjust_affinity(party, left, right, change);
                party[left].morale = (party[left].morale + 1).clamp(0, 100);
                party[right].morale = (party[right].morale + 1).clamp(0, 100);
                messages.push(format!(
                    "{} checks on {}; both take heart.",
                    party[left].name, party[right].name
                ));
            } else if roll < support_chance + argument_chance {
                let change = -rng.stream("party-daily").gen_range(1..=3);
                adjust_affinity(party, left, right, change);
                party[left].morale = (party[left].morale - 1).clamp(0, 100);
                party[right].morale = (party[right].morale - 1).clamp(0, 100);
                messages.push(format!(
                    "{} and {} quarrel over the day's work.",
                    party[left].name, party[right].name
                ));
            }
        }
    }
    messages
}

/// Apply grief for newly reported deaths. The caller supplies each death once.
pub fn mourn(party: &mut [PartyMember], deceased: &[String]) -> Vec<String> {
    let deceased = deceased.iter().collect::<BTreeSet<_>>();
    let mut messages = Vec::new();
    for member in party.iter_mut().filter(|member| member.alive) {
        for name in &deceased {
            if member.name == name.as_str() {
                continue;
            }
            let affinity = member.relationships.affinity.get(name.as_str()).copied().unwrap_or(0);
            member.morale = (member.morale + grief(affinity)).clamp(0, 100);
            messages.push(format!("{} mourns {}.", member.name, name));
        }
    }
    messages
}

pub fn grief(affinity: i16) -> i16 {
    -10 - affinity.max(0) / 4
}

fn occupation_traits(occupation: &str) -> [Trait; 2] {
    match occupation {
        "doctor" => [Trait::Herbalist, Trait::Patient],
        "hunter" => [Trait::Sharpshooter, Trait::Animalwise],
        "preacher" => [Trait::Devout, Trait::Gentle],
        "blacksmith" => [Trait::Handy, Trait::Practical],
        "carpenter" => [Trait::Meticulous, Trait::Handy],
        "farmer" => [Trait::Animalwise, Trait::Hardy],
        "soldier" => [Trait::Cautious, Trait::Stoic],
        "merchant" => [Trait::Sociable, Trait::Practical],
        _ => [Trait::Practical, Trait::Adaptable],
    }
}

fn occupation_skills(occupation: &str) -> Skills {
    match occupation {
        "doctor" => Skills { medicine: 4, ..Skills::default() },
        "hunter" => Skills { hunting: 4, ..Skills::default() },
        "blacksmith" => Skills { repair: 4, ..Skills::default() },
        "carpenter" => Skills { repair: 3, ..Skills::default() },
        "farmer" => Skills { animals: 4, ..Skills::default() },
        "soldier" => Skills { hunting: 2, repair: 2, ..Skills::default() },
        "preacher" => Skills { medicine: 2, ..Skills::default() },
        "merchant" => Skills { animals: 1, ..Skills::default() },
        _ => Skills { repair: 1, ..Skills::default() },
    }
}

fn next_pair(available: &mut Vec<Trait>, rng: &mut SimRng) -> Vec<Trait> {
    if available.len() < 2 {
        *available = Trait::ALL.to_vec();
        available.shuffle(rng.stream("party-profile"));
    }
    vec![available.pop().unwrap(), available.pop().unwrap()]
}

fn trait_skills(trait_: Trait) -> Skills {
    match trait_ {
        Trait::Hardy => Skills { animals: 1, ..Skills::default() },
        Trait::Sharpshooter => Skills { hunting: 3, ..Skills::default() },
        Trait::Herbalist => Skills { medicine: 3, ..Skills::default() },
        Trait::Devout | Trait::Gentle | Trait::Patient => {
            Skills { medicine: 1, ..Skills::default() }
        }
        Trait::Restless | Trait::Daring => Skills { hunting: 1, ..Skills::default() },
        Trait::Practical | Trait::Stoic | Trait::Cautious | Trait::Meticulous => {
            Skills { repair: 1, ..Skills::default() }
        }
        Trait::Handy => Skills { repair: 3, ..Skills::default() },
        Trait::Animalwise | Trait::Sociable => Skills { animals: 2, ..Skills::default() },
        Trait::Adaptable => Skills { hunting: 1, medicine: 1, repair: 1, animals: 1 },
        Trait::Sickly | Trait::Cheerful | Trait::Grumbler | Trait::Impulsive => Skills::default(),
    }
}

fn add_skills(target: &mut Skills, bonus: Skills) {
    target.hunting = target.hunting.saturating_add(bonus.hunting);
    target.medicine = target.medicine.saturating_add(bonus.medicine);
    target.repair = target.repair.saturating_add(bonus.repair);
    target.animals = target.animals.saturating_add(bonus.animals);
}

fn starting_morale(trait_: Trait) -> i16 {
    match trait_ {
        Trait::Cheerful | Trait::Devout | Trait::Gentle | Trait::Patient => 3,
        Trait::Grumbler | Trait::Sickly | Trait::Impulsive => -3,
        Trait::Stoic => 1,
        _ => 0,
    }
}

fn daily_morale(trait_: Trait, resting: bool, filling: bool, varied_food: bool) -> i16 {
    match trait_ {
        Trait::Hardy if !filling => 1,
        Trait::Sickly if !filling => -1,
        Trait::Cheerful if filling => 1,
        Trait::Grumbler if !varied_food => -1,
        Trait::Devout if resting => 1,
        Trait::Restless if resting => -1,
        Trait::Restless => 1,
        Trait::Stoic if !filling => 1,
        Trait::Sociable if varied_food => 1,
        Trait::Cautious if !resting && !filling => -1,
        Trait::Daring if !resting => 1,
        Trait::Patient if resting => 1,
        Trait::Impulsive if !varied_food => -1,
        Trait::Adaptable if varied_food => 1,
        _ => 0,
    }
}

fn social_weight(traits: &[Trait]) -> i16 {
    traits
        .iter()
        .map(|trait_| match trait_ {
            Trait::Cheerful | Trait::Gentle => 3,
            Trait::Devout | Trait::Sociable | Trait::Patient => 2,
            Trait::Stoic | Trait::Cautious => 1,
            Trait::Grumbler => -3,
            Trait::Impulsive => -2,
            Trait::Restless | Trait::Daring => -1,
            _ => 0,
        })
        .sum()
}

fn initialize_relationships(party: &mut [PartyMember], rng: &mut SimRng) {
    for left in 0..party.len() {
        for right in (left + 1)..party.len() {
            if party[left].name == party[right].name {
                continue;
            }
            let random: i16 = rng.stream("party-relationships").gen_range(-6..=12);
            let affinity =
                (random + social_weight(&party[left].traits) + social_weight(&party[right].traits))
                    .clamp(-10, 30);
            let left_name = party[left].name.clone();
            let right_name = party[right].name.clone();
            party[left].relationships.affinity.insert(right_name, affinity);
            party[right].relationships.affinity.insert(left_name, affinity);
        }
    }
}

fn adjust_affinity(party: &mut [PartyMember], left: usize, right: usize, change: i16) {
    let left_name = party[left].name.clone();
    let right_name = party[right].name.clone();
    let current = party[left].relationships.affinity.get(&right_name).copied().unwrap_or(0);
    let affinity = (current + change).clamp(-10, 30);
    party[left].relationships.affinity.insert(right_name, affinity);
    party[right].relationships.affinity.insert(left_name, affinity);
}

#[cfg(test)]
fn trait_has_effect(trait_: Trait) -> bool {
    trait_skills(trait_) != Skills::default()
        || starting_morale(trait_) != 0
        || daily_morale(trait_, true, true, true) != 0
        || daily_morale(trait_, false, false, false) != 0
        || social_weight(&[trait_]) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn party() -> Vec<PartyMember> {
        ["Ada", "Bert", "Clara", "Dora", "Eli"].map(|name| PartyMember::new(name.into())).to_vec()
    }

    #[test]
    fn initialization_is_seeded_varied_and_social() {
        let mut left = party();
        let mut right = party();
        let mut different = party();
        initialize(&mut left, "doctor", &mut SimRng::new(91));
        initialize(&mut right, "doctor", &mut SimRng::new(91));
        initialize(&mut different, "doctor", &mut SimRng::new(92));
        assert_eq!(left, right);
        assert_ne!(left, different, "a different seed should produce a different profile");
        assert!(left.iter().all(|member| (18..=65).contains(&member.age)));
        assert!(left.iter().all(|member| member.traits.len() == 2));
        let traits = left.iter().flat_map(|member| member.traits.iter()).collect::<BTreeSet<_>>();
        assert_eq!(traits.len(), 10, "a five-person party receives ten varied traits");
        assert!(left[0].skills.medicine >= 7, "doctor leader gets the profession bonus");
        assert!(left[1..].iter().all(|member| member.skills.medicine <= 1));
        for member in &left {
            assert!(!member.relationships.affinity.contains_key(&member.name));
            for (name, affinity) in &member.relationships.affinity {
                let other = left.iter().find(|other| other.name == *name).unwrap();
                assert_eq!(other.relationships.affinity[&member.name], *affinity);
                assert!((-10..=30).contains(affinity));
            }
        }
    }

    #[test]
    fn every_trait_changes_a_profile_or_daily_behavior() {
        assert_eq!(Trait::ALL.len(), 20);
        for trait_ in Trait::ALL {
            assert!(trait_has_effect(trait_), "{trait_:?} has no modeled effect");
        }
    }

    #[test]
    fn daily_stays_bounded_and_ignores_dead_members() {
        let mut members = party();
        initialize(&mut members, "merchant", &mut SimRng::new(7));
        members[4].alive = false;
        members[4].morale = 37;
        let dead_relationships = members[4].relationships.clone();
        let mut rng = SimRng::new(12);
        let mut messages = Vec::new();
        for _ in 0..300 {
            messages.extend(daily(&mut members, false, true, true, &mut rng));
        }
        assert!(members[..4].iter().all(|member| (0..=100).contains(&member.morale)));
        assert_eq!(members[4].morale, 37);
        assert_eq!(members[4].relationships, dead_relationships);
        assert!(!messages.is_empty());
        assert!(messages
            .iter()
            .all(|message| message.contains(" and ") || message.contains(" checks on ")));
        for member in &members[..4] {
            for (name, affinity) in &member.relationships.affinity {
                let other = members.iter().find(|other| other.name == *name).unwrap();
                assert_eq!(other.relationships.affinity[&member.name], *affinity);
                assert!((-10..=30).contains(affinity));
            }
        }
    }

    #[test]
    fn mourning_uses_affinity_clamps_morale_and_skips_the_dead() {
        let mut members = party();
        initialize(&mut members, "farmer", &mut SimRng::new(4));
        members[0].morale = 8;
        members[0].relationships.affinity.insert("Bert".into(), 30);
        members[1].alive = false;
        let messages = mourn(&mut members, &["Bert".into()]);
        assert_eq!(members[0].morale, 0, "close friends cause deeper grief");
        assert_eq!(members[1].morale, 52);
        assert!(messages.iter().any(|message| message == "Ada mourns Bert."));
        assert!(!messages.iter().any(|message| message.starts_with("Bert ")));
    }
}
