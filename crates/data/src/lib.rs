//! Typed, build-embedded content for Pioneer Trail.

use include_dir::{include_dir, Dir};
use pioneer_sim::{Condition, Effect, GameContent, LandmarkKind, TrailDefinition};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

pub static ART: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/art");
static EVENT_BATCHES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/events");
static QUOTE_BATCHES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/quotes");
const CONTENT: &str = include_str!("../content.ron");

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("could not parse embedded content: {0}")]
    Parse(#[from] ron::error::SpannedError),
    #[error("content validation failed: {0}")]
    Validation(String),
}

/// Parse the RON included in the release binary and reject broken references.
pub fn load() -> Result<GameContent, ContentError> {
    let mut content: GameContent = ron::from_str(CONTENT)?;
    for file in
        EVENT_BATCHES.files().filter(|file| file.path().extension().is_some_and(|ext| ext == "ron"))
    {
        let mut events: Vec<pioneer_sim::EventDefinition> = ron::de::from_bytes(file.contents())?;
        content.events.append(&mut events);
    }
    for file in
        QUOTE_BATCHES.files().filter(|file| file.path().extension().is_some_and(|ext| ext == "ron"))
    {
        let mut quotes: Vec<pioneer_sim::QuoteDefinition> = ron::de::from_bytes(file.contents())?;
        content.quotes.append(&mut quotes);
    }
    validate(&content)?;
    Ok(content)
}

pub fn validate(content: &GameContent) -> Result<(), ContentError> {
    unique(content.eras.iter().map(|value| value.id.as_str()), "era")?;
    unique(content.trails.iter().map(|value| value.id.as_str()), "trail")?;
    unique(content.occupations.iter().map(|value| value.id.as_str()), "occupation")?;
    unique(content.items.iter().map(|value| value.id.as_str()), "item")?;
    unique(content.ailments.iter().map(|value| value.id.as_str()), "ailment")?;
    unique(content.events.iter().map(|value| value.id.as_str()), "event")?;
    unique(content.quotes.iter().map(|value| value.id.as_str()), "quote")?;
    unique(content.letters.iter().map(|value| value.id.as_str()), "letter")?;
    unique(content.speakers.iter().map(|value| value.id.as_str()), "speaker")?;
    unique(content.events.iter().map(|value| value.text.as_str()), "event text")?;
    unique(
        content.quotes.iter().map(|value| {
            value.text.split_once(':').map_or(value.text.as_str(), |(_, body)| body.trim())
        }),
        "quote body",
    )?;

    let trail_ids = content.trails.iter().map(|trail| trail.id.as_str()).collect::<HashSet<_>>();
    let era_ids = content.eras.iter().map(|era| era.id.as_str()).collect::<HashSet<_>>();
    let occupation_ids =
        content.occupations.iter().map(|occupation| occupation.id.as_str()).collect::<HashSet<_>>();
    let item_ids = content.items.iter().map(|item| item.id.as_str()).collect::<HashSet<_>>();
    let ailment_ids =
        content.ailments.iter().map(|ailment| ailment.id.as_str()).collect::<HashSet<_>>();
    let event_ids = content.events.iter().map(|event| event.id.as_str()).collect::<HashSet<_>>();
    let landmark_ids = content
        .trails
        .iter()
        .flat_map(|trail| trail.nodes.iter())
        .map(|node| node.id.as_str())
        .collect::<HashSet<_>>();
    let fort_landmark_ids = content
        .trails
        .iter()
        .flat_map(|trail| trail.nodes.iter())
        .filter(|node| node.kind == LandmarkKind::Fort)
        .map(|node| node.id.as_str())
        .collect::<HashSet<_>>();
    let store_ids = content
        .trails
        .iter()
        .flat_map(|trail| trail.nodes.iter())
        .filter(|node| node.store)
        .map(|node| node.id.as_str())
        .collect::<HashSet<_>>();
    for letter in &content.letters {
        expect(
            valid_text(&letter.recipient, 80) && valid_text(&letter.text, 240),
            "invalid letter text",
        )?;
        expect((100..=3_000).contains(&letter.reward_cents), "letter reward is out of range")?;
        let trail =
            content.trails.iter().find(|trail| trail.id == letter.trail_id).ok_or_else(|| {
                ContentError::Validation(format!("letter {} names an unknown trail", letter.id))
            })?;
        let origin =
            trail.nodes.iter().find(|node| node.id == letter.origin_id).ok_or_else(|| {
                ContentError::Validation(format!("letter {} names an unknown origin", letter.id))
            })?;
        expect(origin.store, &format!("letter {} origin is not a supply stop", letter.id))?;
        expect(
            trail.nodes.iter().any(|node| node.id == letter.destination_id && node.store),
            &format!("letter {} destination is not a supply stop", letter.id),
        )?;
        expect(
            letter.origin_id != letter.destination_id,
            &format!("letter {} does not travel later", letter.id),
        )?;
        expect(
            letter_destination_reachable(trail, &letter.origin_id, &letter.destination_id),
            &format!("letter {} destination is unreachable", letter.id),
        )?;
        expect(
            letter_destination_unavoidable(trail, &letter.origin_id, &letter.destination_id),
            &format!("letter {} destination can be bypassed", letter.id),
        )?;
        expect(
            content.eras.iter().any(|era| {
                !content.era_rules.get(&era.id).is_some_and(|rules| {
                    rules
                        .unavailable_stores
                        .iter()
                        .any(|store| store == &letter.origin_id || store == &letter.destination_id)
                })
            }),
            &format!("letter {} destination is unavailable in every era", letter.id),
        )?;
    }
    let produced_flags = content
        .events
        .iter()
        .flat_map(|event| {
            event.effects.iter().chain(event.choices.iter().flat_map(|choice| &choice.effects))
        })
        .filter_map(|effect| match effect {
            Effect::SetFlag(flag) => Some(flag.as_str()),
            _ => None,
        })
        .collect::<HashSet<_>>();
    let scheduled_events = content
        .events
        .iter()
        .flat_map(|event| {
            event.effects.iter().chain(event.choices.iter().flat_map(|choice| &choice.effects))
        })
        .filter_map(|effect| match effect {
            Effect::Schedule { event_id, .. } => Some(event_id.as_str()),
            _ => None,
        })
        .collect::<HashSet<_>>();

    for (era_id, rules) in &content.era_rules {
        expect(era_ids.contains(era_id.as_str()), &format!("era rules name unknown era {era_id}"))?;
        expect(
            (50..=200).contains(&rules.price_percent),
            &format!("era rules {era_id} have invalid price percent"),
        )?;
        expect(
            (50..=200).contains(&rules.ferry_fee_percent),
            &format!("era rules {era_id} have invalid ferry percent"),
        )?;
        expect(
            (1..=6).contains(&rules.guide_cost_clothing),
            &format!("era rules {era_id} have invalid guide clothing cost"),
        )?;
        for store_id in &rules.unavailable_stores {
            expect(
                store_ids.contains(store_id.as_str()),
                &format!("era rules {era_id} name unknown store {store_id}"),
            )?;
        }
        for (event_id, percent) in &rules.event_weight_percent {
            expect(
                event_ids.contains(event_id.as_str()),
                &format!("era rules {era_id} name unknown event {event_id}"),
            )?;
            expect(
                (25..=300).contains(percent),
                &format!("era rules {era_id} have invalid event percent"),
            )?;
        }
    }
    if !content.era_rules.is_empty() {
        for era_id in &era_ids {
            expect(content.era_rules.contains_key(*era_id), &format!("era {era_id} has no rules"))?;
        }
    }
    for region_id in content.regions.keys() {
        expect(
            landmark_ids.contains(region_id.as_str()),
            &format!("region names unknown landmark {region_id}"),
        )?;
    }
    if !content.regions.is_empty() {
        for landmark_id in &landmark_ids {
            expect(
                content.regions.contains_key(*landmark_id),
                &format!("landmark {landmark_id} has no region"),
            )?;
        }
    }

    for trail in &content.trails {
        let own_nodes = trail.nodes.iter().map(|node| node.id.as_str()).collect::<HashSet<_>>();
        unique(trail.nodes.iter().map(|node| node.id.as_str()), "landmark")?;
        expect(
            own_nodes.contains(trail.start_node_id.as_str()),
            &format!("trail {} has unknown start", trail.id),
        )?;
        expect(
            own_nodes.contains(trail.goal_node_id.as_str()),
            &format!("trail {} has unknown goal", trail.id),
        )?;
        for node in &trail.nodes {
            expect(valid_text(&node.name, 60), &format!("landmark {} has invalid name", node.id))?;
            unique(node.routes.iter().map(|route| route.id.as_str()), "route")?;
            for route in &node.routes {
                expect(route.distance_miles > 0, &format!("route {} has zero distance", route.id))?;
                expect(
                    own_nodes.contains(route.target_id.as_str()),
                    &format!("route {} has unknown target {}", route.id, route.target_id),
                )?;
            }
            if let Some(river) = &node.river {
                expect(
                    river.width_feet > 0 && river.depth_feet > 0,
                    &format!("river {} has invalid dimensions", node.id),
                )?;
            }
            expect(
                !matches!(node.kind, LandmarkKind::River) || node.river.is_some(),
                &format!("river {} is missing crossing data", node.id),
            )?;
            expect(
                node.river.is_none()
                    || matches!(node.kind, LandmarkKind::River | LandmarkKind::Finale),
                &format!("landmark {} has river data but is not a crossing", node.id),
            )?;
            if matches!(node.kind, LandmarkKind::Fork) {
                expect(node.routes.len() >= 2, &format!("fork {} needs two routes", node.id))?;
            }
            if node.id == trail.goal_node_id {
                expect(
                    node.routes.is_empty(),
                    &format!("finale {} must not have routes", node.id),
                )?;
            } else {
                expect(!node.routes.is_empty(), &format!("non-finale {} is a trap", node.id))?;
            }
        }
        validate_trail_graph(trail, &own_nodes)?;
    }
    for occupation in &content.occupations {
        expect(
            occupation.starting_cash_cents >= 0,
            &format!("occupation {} has negative cash", occupation.id),
        )?;
        expect(
            occupation.score_multiplier.is_finite() && occupation.score_multiplier > 0.0,
            &format!("occupation {} has invalid multiplier", occupation.id),
        )?;
    }
    for ailment in &content.ailments {
        expect(
            ailment.severity > 0 && ailment.daily_damage > 0 && ailment.mortality_per_mille <= 1000,
            &format!("ailment {} has invalid ranges", ailment.id),
        )?;
    }
    for item in &content.items {
        expect(item.limit > 0, &format!("item {} has zero limit", item.id))?;
        expect(item.price_cents >= 0, &format!("item {} has negative price", item.id))?;
    }
    let refs = ConditionRefs {
        trails: &trail_ids,
        eras: &era_ids,
        occupations: &occupation_ids,
        items: &item_ids,
        ailments: &ailment_ids,
        landmarks: &landmark_ids,
        flags: &produced_flags,
    };
    for event in &content.events {
        expect(
            event.weight != 0 || scheduled_events.contains(event.id.as_str()),
            &format!("weight-zero event {} is not scheduled", event.id),
        )?;
        expect(valid_text(&event.text, 240), &format!("event {} text is invalid", event.id))?;
        for condition in &event.conditions {
            validate_condition(condition, &refs)?;
        }
        validate_event_landmarks(&event.conditions, &content.trails)?;
        validate_effects(&event.effects, &item_ids, &ailment_ids, &event_ids)?;
        for choice in &event.choices {
            expect(
                valid_text(&choice.id, 80) && valid_text(&choice.label, 80),
                &format!("event {} has an invalid choice", event.id),
            )?;
            for condition in &choice.conditions {
                validate_condition(condition, &refs)?;
            }
            validate_event_landmarks(&choice.conditions, &content.trails)?;
            validate_effects(&choice.effects, &item_ids, &ailment_ids, &event_ids)?;
        }
        unique(event.choices.iter().map(|choice| choice.id.as_str()), "event choice")?;
        expect(
            event.choices.is_empty() || event.choices.iter().any(has_unconditional_fallback),
            &format!("event {} has no unconditional fallback", event.id),
        )?;
    }
    for quote in &content.quotes {
        expect(
            valid_text(&quote.text, 240) && quote.text.contains(':'),
            &format!("quote {} must be named and concise", quote.id),
        )?;
        expect(
            !quote.seasons.is_empty(),
            &format!("quote {} needs at least one season", quote.id),
        )?;
        if let Some(id) = &quote.landmark_id {
            expect(
                landmark_ids.contains(id.as_str()),
                &format!("quote {} names unknown landmark {}", quote.id, id),
            )?;
        }
        expect(
            quote.state_tags.iter().all(|tag| {
                matches!(
                    tag.as_str(),
                    "hungry"
                        | "sick"
                        | "wealthy"
                        | "late"
                        | "cold"
                        | "weary"
                        | "homesick"
                        | "grieving"
                        | "hopeful"
                        | "thirsty"
                )
            }),
            &format!("quote {} has an unsupported state tag", quote.id),
        )?;
    }
    for speaker in &content.speakers {
        expect(
            fort_landmark_ids.contains(speaker.landmark_id.as_str()),
            &format!("speaker {} names a non-fort landmark {}", speaker.id, speaker.landmark_id),
        )?;
        expect(
            valid_text(&speaker.name, 60) && !speaker.name.is_empty(),
            &format!("speaker {} needs a name", speaker.id),
        )?;
        expect(
            valid_text(&speaker.greeting, 240)
                && valid_text(&speaker.returning_greeting, 240)
                && speaker.greeting != speaker.returning_greeting,
            &format!("speaker {} needs distinct first-meeting and returning greetings", speaker.id),
        )?;
        expect(
            speaker.favor_food_lbs > 0 && speaker.favor_food_lbs <= 50,
            &format!("speaker {} favor_food_lbs must stay a small, bounded amount", speaker.id),
        )?;
    }
    Ok(())
}

struct ConditionRefs<'a> {
    trails: &'a HashSet<&'a str>,
    eras: &'a HashSet<&'a str>,
    occupations: &'a HashSet<&'a str>,
    items: &'a HashSet<&'a str>,
    ailments: &'a HashSet<&'a str>,
    landmarks: &'a HashSet<&'a str>,
    flags: &'a HashSet<&'a str>,
}
fn validate_condition(condition: &Condition, refs: &ConditionRefs<'_>) -> Result<(), ContentError> {
    let ConditionRefs { trails, eras, occupations, items, ailments, landmarks, flags } = refs;
    match condition {
        Condition::All(children) | Condition::Any(children) => {
            expect(!children.is_empty(), "compound condition is empty")?;
            for child in children {
                validate_condition(child, refs)?;
            }
        }
        Condition::Not(item) => validate_condition(item, refs)?,
        Condition::Trail(id) => {
            expect(trails.contains(id.as_str()), &format!("unknown trail condition {id}"))?
        }
        Condition::Era(id) => {
            expect(eras.contains(id.as_str()), &format!("unknown era condition {id}"))?
        }
        Condition::Occupation(id) => expect(
            occupations.contains(id.as_str()),
            &format!("unknown occupation condition {id}"),
        )?,
        Condition::HasAilment(id) => {
            expect(ailments.contains(id.as_str()), &format!("unknown ailment condition {id}"))?
        }
        Condition::AtLandmark(id) => {
            expect(landmarks.contains(id.as_str()), &format!("unknown landmark condition {id}"))?
        }
        Condition::InventoryAtLeast { item_id, quantity } => {
            expect(items.contains(item_id.as_str()), &format!("unknown item condition {item_id}"))?;
            expect(*quantity > 0, &format!("item condition {item_id} has zero quantity"))?
        }
        Condition::CashAtLeast(cash) => expect(*cash >= 0, "cash condition is negative")?,
        Condition::MoraleBelow(morale) => {
            expect((0..=100).contains(morale), "morale condition is out of range")?
        }
        Condition::RelationshipAtLeast(affinity) => {
            expect((-100..=100).contains(affinity), "relationship condition is out of range")?
        }
        Condition::PartySizeBelow(size) => {
            expect((1..=12).contains(size), "party-size condition is out of range")?
        }
        Condition::Flag(flag) => expect(
            flags.contains(flag.as_str()),
            &format!("flag condition {flag} is never produced"),
        )?,
        _ => {}
    }
    Ok(())
}

fn validate_effects(
    effects: &[Effect],
    items: &HashSet<&str>,
    ailments: &HashSet<&str>,
    events: &HashSet<&str>,
) -> Result<(), ContentError> {
    for effect in effects {
        match effect {
            Effect::Message(message) => {
                expect(valid_text(message, 240), "message effect has invalid text")?
            }
            Effect::InflictAilment(id) | Effect::HealAilment(id) => {
                expect(ailments.contains(id.as_str()), &format!("unknown ailment effect {id}"))?
            }
            Effect::AdjustItem { item_id, quantity } => {
                expect(
                    items.contains(item_id.as_str()),
                    &format!("unknown item effect {item_id}"),
                )?;
                expect(*quantity != 0, &format!("item effect {item_id} has zero quantity"))?
            }
            Effect::AdjustCash(cash) => expect(*cash != 0, "cash effect is zero")?,
            Effect::AdjustMorale(morale) => expect(
                *morale != 0 && (-100..=100).contains(morale),
                "morale effect is out of range",
            )?,
            Effect::AdjustRelationship(affinity) => expect(
                *affinity != 0 && (-100..=100).contains(affinity),
                "relationship effect is out of range",
            )?,
            Effect::AddMember { name, age } => {
                expect(valid_text(name, 24), "added member has an invalid name")?;
                expect(*age <= 100, "added member age is out of range")?
            }
            Effect::Schedule { event_id, days } => {
                expect(
                    events.contains(event_id.as_str()),
                    &format!("scheduled event {event_id} does not exist"),
                )?;
                expect(
                    (1..=365).contains(days),
                    &format!("scheduled event {event_id} has invalid days"),
                )?
            }
            Effect::SetFlag(flag) | Effect::ClearFlag(flag) => {
                expect(valid_text(flag, 80), "flag effect has an invalid id")?
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_event_landmarks(
    conditions: &[Condition],
    trails: &[TrailDefinition],
) -> Result<(), ContentError> {
    let mut landmarks = Vec::new();
    for condition in conditions {
        collect_landmarks(condition, &mut landmarks);
    }
    for landmark in &landmarks {
        expect(
            trails.iter().any(|trail| {
                trail.nodes.iter().any(|node| node.id == *landmark)
                    && trail.goal_node_id != *landmark
            }),
            &format!("event landmark {landmark} is terminal-only"),
        )?;
    }

    let mut required_trails = Vec::new();
    let mut required_landmarks = Vec::new();
    for condition in conditions {
        collect_all_requirements(condition, &mut required_trails, &mut required_landmarks);
    }
    for trail_id in required_trails {
        let trail = trails.iter().find(|trail| trail.id == trail_id).unwrap();
        for landmark in &required_landmarks {
            expect(
                trail.nodes.iter().any(|node| node.id == *landmark),
                &format!("trail condition {trail_id} cannot reach landmark {landmark}"),
            )?;
        }
    }
    Ok(())
}

fn collect_landmarks<'a>(condition: &'a Condition, landmarks: &mut Vec<&'a str>) {
    match condition {
        Condition::AtLandmark(id) => landmarks.push(id),
        Condition::All(children) | Condition::Any(children) => {
            for child in children {
                collect_landmarks(child, landmarks);
            }
        }
        // A negated terminal landmark is reachable everywhere before arrival.
        Condition::Not(_) => {}
        _ => {}
    }
}

fn collect_all_requirements<'a>(
    condition: &'a Condition,
    trails: &mut Vec<&'a str>,
    landmarks: &mut Vec<&'a str>,
) {
    match condition {
        Condition::Trail(id) => trails.push(id),
        Condition::AtLandmark(id) => landmarks.push(id),
        Condition::All(children) => {
            for child in children {
                collect_all_requirements(child, trails, landmarks);
            }
        }
        _ => {}
    }
}

fn has_unconditional_fallback(choice: &pioneer_sim::EventChoice) -> bool {
    choice.conditions.is_empty()
        && choice.effects.iter().all(|effect| {
            !matches!(effect, Effect::AdjustCash(amount) if *amount < 0)
                && !matches!(effect, Effect::AdjustItem { quantity, .. } if *quantity < 0)
        })
}

fn valid_text(text: &str, max_len: usize) -> bool {
    !text.is_empty() && text.len() <= max_len && !text.chars().any(char::is_control)
}

fn letter_destination_reachable(trail: &TrailDefinition, origin: &str, destination: &str) -> bool {
    let mut reached = HashSet::new();
    let mut queue = VecDeque::from([origin]);
    while let Some(node) = queue.pop_front() {
        if !reached.insert(node) {
            continue;
        }
        if node == destination {
            return true;
        }
        if let Some(definition) = trail.nodes.iter().find(|candidate| candidate.id == node) {
            queue.extend(definition.routes.iter().map(|route| route.target_id.as_str()));
        }
    }
    false
}

fn letter_destination_unavoidable(
    trail: &TrailDefinition,
    origin: &str,
    destination: &str,
) -> bool {
    let mut reached = HashSet::new();
    let mut queue = VecDeque::from([origin]);
    while let Some(node) = queue.pop_front() {
        if node == destination || !reached.insert(node) {
            continue;
        }
        if node == trail.goal_node_id {
            return false;
        }
        if let Some(definition) = trail.nodes.iter().find(|candidate| candidate.id == node) {
            queue.extend(definition.routes.iter().map(|route| route.target_id.as_str()));
        }
    }
    true
}

fn validate_trail_graph(
    trail: &TrailDefinition,
    nodes: &HashSet<&str>,
) -> Result<(), ContentError> {
    let edges = trail
        .nodes
        .iter()
        .map(|node| {
            (
                node.id.as_str(),
                node.routes.iter().map(|route| route.target_id.as_str()).collect::<Vec<_>>(),
            )
        })
        .collect::<HashMap<_, _>>();
    let mut reached = HashSet::new();
    let mut queue = VecDeque::from([trail.start_node_id.as_str()]);
    while let Some(node) = queue.pop_front() {
        if reached.insert(node) {
            for target in &edges[node] {
                queue.push_back(target);
            }
        }
    }
    expect(reached.len() == nodes.len(), &format!("trail {} has unreachable nodes", trail.id))?;
    let mut reverse: HashMap<&str, Vec<&str>> = nodes.iter().map(|id| (*id, Vec::new())).collect();
    for (node, targets) in &edges {
        for target in targets {
            reverse.get_mut(target).unwrap().push(node);
        }
    }
    let mut can_finish = HashSet::new();
    let mut queue = VecDeque::from([trail.goal_node_id.as_str()]);
    while let Some(node) = queue.pop_front() {
        if can_finish.insert(node) {
            for prior in &reverse[node] {
                queue.push_back(prior);
            }
        }
    }
    expect(
        can_finish.len() == nodes.len(),
        &format!("trail {} has a route that cannot reach its goal", trail.id),
    )
}

fn unique<'a>(ids: impl Iterator<Item = &'a str>, kind: &str) -> Result<(), ContentError> {
    let mut seen = HashSet::new();
    for id in ids {
        expect(!id.is_empty() && !id.chars().any(char::is_control), &format!("invalid {kind} id"))?;
        expect(seen.insert(id), &format!("duplicate {kind} id {id}"))?;
    }
    Ok(())
}

fn expect(condition: bool, message: &str) -> Result<(), ContentError> {
    condition.then_some(()).ok_or_else(|| ContentError::Validation(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_content_parses_and_validates() {
        assert!(load().is_ok());
    }

    #[test]
    fn family_events_load_and_invalid_family_dsl_bounds_are_rejected() {
        let content = load().unwrap();
        for id in ["wedding_offer", "family_feud", "party_departure", "relative_joins"] {
            assert!(content.events.iter().any(|event| event.id == id), "missing {id}");
        }
        let mut invalid = content.clone();
        invalid.events[0].conditions = vec![Condition::RelationshipAtLeast(101)];
        assert!(validate(&invalid).unwrap_err().to_string().contains("relationship condition"));
        invalid.events[0].conditions = vec![Condition::Always];
        invalid.events[0].effects = vec![Effect::AddMember { name: "A".into(), age: 101 }];
        assert!(validate(&invalid).unwrap_err().to_string().contains("member age"));
    }

    #[test]
    fn v1_catalogue_inventory_is_present() {
        let content = load().unwrap();
        assert_eq!(content.eras.len(), 4);
        assert_eq!(content.trails.len(), 3);
        assert!(content.occupations.len() >= 9);
        assert!(content.items.len() >= 10);
        assert!(content.ailments.len() >= 20);
        assert!(content.events.len() >= 150);
        assert!(content.quotes.len() >= 200);
        assert_eq!(content.letters.len(), 3);
        assert_eq!(
            content.letters.iter().map(|letter| letter.trail_id.as_str()).collect::<HashSet<_>>(),
            HashSet::from(["oregon", "california", "mormon"])
        );
        assert!(content.speakers.len() >= 10);
        let fort_ids = content
            .trails
            .iter()
            .flat_map(|trail| trail.nodes.iter())
            .filter(|node| node.kind == pioneer_sim::LandmarkKind::Fort)
            .map(|node| node.id.as_str())
            .collect::<HashSet<_>>();
        for id in &fort_ids {
            assert!(
                content.speakers.iter().any(|speaker| speaker.landmark_id == *id),
                "fort {id} has no named speaker"
            );
        }
    }

    #[test]
    fn speaker_landmark_must_be_a_fort() {
        let content = load().unwrap();
        let mut invalid = content;
        invalid.speakers[0].landmark_id = "independence".into();
        assert!(validate(&invalid).unwrap_err().to_string().contains("non-fort landmark"));
    }

    #[test]
    fn speaker_favor_food_must_stay_small_and_bounded() {
        let content = load().unwrap();
        let mut too_much = content.clone();
        too_much.speakers[0].favor_food_lbs = 5_000;
        assert!(validate(&too_much).unwrap_err().to_string().contains("bounded amount"));

        let mut zero = content;
        zero.speakers[0].favor_food_lbs = 0;
        assert!(validate(&zero).unwrap_err().to_string().contains("bounded amount"));
    }

    #[test]
    fn letters_reject_a_destination_that_a_fork_can_bypass() {
        let mut content = load().unwrap();
        let letter = content.letters.iter_mut().find(|letter| letter.id == "sierra_note").unwrap();
        letter.destination_id = "soda_springs_ca".into();
        content
            .trails
            .iter_mut()
            .find(|trail| trail.id == "california")
            .unwrap()
            .nodes
            .iter_mut()
            .find(|node| node.id == "soda_springs_ca")
            .unwrap()
            .store = true;
        assert!(validate(&content).unwrap_err().to_string().contains("can be bypassed"));
    }

    #[test]
    fn era_rules_and_regions_reject_unknown_ids_and_invalid_ranges() {
        let content = load().unwrap();

        let mut unknown_era = content.clone();
        let rules = unknown_era.era_rules["1848"].clone();
        unknown_era.era_rules.insert("1900".into(), rules);
        assert!(validate(&unknown_era).unwrap_err().to_string().contains("unknown era"));

        let mut missing_region = content.clone();
        missing_region.regions.remove("independence");
        assert!(validate(&missing_region).unwrap_err().to_string().contains("has no region"));

        let mut unknown_region = content.clone();
        let region = unknown_region.regions["independence"];
        unknown_region.regions.insert("nowhere".into(), region);
        assert!(validate(&unknown_region).unwrap_err().to_string().contains("unknown landmark"));

        let mut invalid_price = content.clone();
        invalid_price.era_rules.get_mut("1848").unwrap().price_percent = 49;
        assert!(validate(&invalid_price).unwrap_err().to_string().contains("invalid price"));

        let mut unknown_event = content;
        unknown_event
            .era_rules
            .get_mut("1852")
            .unwrap()
            .event_weight_percent
            .insert("unknown_event".into(), 100);
        assert!(validate(&unknown_event).unwrap_err().to_string().contains("unknown event"));
    }

    #[test]
    fn missing_route_target_is_rejected() {
        let mut content = load().unwrap();
        content.trails[0].nodes[0].routes[0].target_id = "nowhere".into();
        assert!(validate(&content).unwrap_err().to_string().contains("unknown target"));
    }

    #[test]
    fn missing_ailment_reference_is_rejected() {
        let mut content = load().unwrap();
        content.events[0].effects = vec![Effect::InflictAilment("ghost_ague".into())];
        assert!(validate(&content).unwrap_err().to_string().contains("unknown ailment"));
    }

    #[test]
    fn disconnected_route_is_rejected() {
        let mut content = load().unwrap();
        content.trails[0].nodes[0].routes[0].target_id = content.trails[0].goal_node_id.clone();
        assert!(validate(&content).unwrap_err().to_string().contains("unreachable"));
    }

    #[test]
    fn california_uses_the_shared_spine_through_fort_hall() {
        let content = load().unwrap();
        let california = content.trails.iter().find(|trail| trail.id == "california").unwrap();
        for id in [
            "kansas_river_ca",
            "big_blue_ca",
            "fort_kearney_ca",
            "chimney_rock_ca",
            "independence_rock_ca",
            "south_pass_ca",
            "soda_springs_ca",
            "fort_hall_ca",
        ] {
            assert!(california.nodes.iter().any(|node| node.id == id));
        }
        let bridger = california.nodes.iter().find(|node| node.id == "fort_bridger_ca").unwrap();
        assert_eq!(
            bridger.routes.iter().find(|route| route.id == "main").unwrap().target_id,
            "soda_springs_ca"
        );
        let hall = california.nodes.iter().find(|node| node.id == "fort_hall_ca").unwrap();
        assert_eq!(hall.routes[0].target_id, "humboldt");
    }

    #[test]
    fn choice_conditions_are_validated_recursively() {
        let mut content = load().unwrap();
        content.events[0].choices[1].conditions =
            vec![Condition::All(vec![Condition::InventoryAtLeast {
                item_id: "ghost".into(),
                quantity: 1,
            }])];
        assert!(validate(&content).unwrap_err().to_string().contains("unknown item condition"));
    }

    #[test]
    fn invalid_item_and_schedule_effects_are_rejected() {
        let mut content = load().unwrap();
        content.events[0].effects =
            vec![Effect::AdjustItem { item_id: "ghost".into(), quantity: 1 }];
        assert!(validate(&content).unwrap_err().to_string().contains("unknown item effect"));

        let mut content = load().unwrap();
        content.events[0].effects = vec![Effect::Schedule { event_id: "wheel".into(), days: 0 }];
        assert!(validate(&content).unwrap_err().to_string().contains("invalid days"));
    }

    #[test]
    fn invalid_numeric_conditions_and_effects_are_rejected() {
        let mut content = load().unwrap();
        content.events[0].conditions = vec![Condition::MoraleBelow(101)];
        assert!(validate(&content).unwrap_err().to_string().contains("morale condition"));

        let mut content = load().unwrap();
        content.events[0].conditions =
            vec![Condition::InventoryAtLeast { item_id: "food".into(), quantity: 0 }];
        assert!(validate(&content).unwrap_err().to_string().contains("zero quantity"));

        let mut content = load().unwrap();
        content.events[0].conditions = vec![Condition::CashAtLeast(-1)];
        assert!(validate(&content).unwrap_err().to_string().contains("cash condition"));

        let mut content = load().unwrap();
        content.events[0].effects = vec![Effect::AdjustCash(0)];
        assert!(validate(&content).unwrap_err().to_string().contains("cash effect"));

        let mut content = load().unwrap();
        content.events[0].effects =
            vec![Effect::AdjustItem { item_id: "food".into(), quantity: 0 }];
        assert!(validate(&content).unwrap_err().to_string().contains("zero quantity"));
    }

    #[test]
    fn unproduced_flags_and_terminal_events_are_rejected() {
        let mut content = load().unwrap();
        content.events[0].conditions = vec![Condition::Flag("ghost_flag".into())];
        assert!(validate(&content).unwrap_err().to_string().contains("never produced"));

        let mut content = load().unwrap();
        content.events[0].conditions = vec![Condition::AtLandmark("willamette".into())];
        assert!(validate(&content).unwrap_err().to_string().contains("terminal-only"));

        let mut content = load().unwrap();
        content.events[0].conditions = vec![
            Condition::Trail("california".into()),
            Condition::AtLandmark("fort_laramie".into()),
        ];
        assert!(validate(&content).unwrap_err().to_string().contains("cannot reach landmark"));
    }

    #[test]
    fn fallbacks_weight_zero_events_and_quote_tags_are_checked() {
        let mut content = load().unwrap();
        content.events[0].choices[1].conditions = vec![Condition::CashAtLeast(1)];
        assert!(validate(&content).unwrap_err().to_string().contains("no unconditional fallback"));

        let mut content = load().unwrap();
        content.events[0].weight = 0;
        assert!(validate(&content).unwrap_err().to_string().contains("not scheduled"));

        let mut content = load().unwrap();
        content.quotes[0].state_tags = vec!["starving".into()];
        assert!(validate(&content).unwrap_err().to_string().contains("unsupported state tag"));
    }

    #[test]
    fn control_text_is_rejected() {
        let mut content = load().unwrap();
        content.events[0].text = "bad\ntext".into();
        assert!(validate(&content).unwrap_err().to_string().contains("invalid event text"));

        let mut content = load().unwrap();
        content.events[0].id.clear();
        assert!(validate(&content).unwrap_err().to_string().contains("invalid event id"));
    }
}
