//! Typed, build-embedded content for Pioneer Trail.

use include_dir::{include_dir, Dir};
use pioneer_sim::{Condition, Effect, GameContent, LandmarkKind, TrailDefinition};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

pub static ART: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/art");
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
    let content = ron::from_str(CONTENT)?;
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
    let ailment_ids =
        content.ailments.iter().map(|ailment| ailment.id.as_str()).collect::<HashSet<_>>();
    let event_ids = content.events.iter().map(|event| event.id.as_str()).collect::<HashSet<_>>();
    let landmark_ids = content
        .trails
        .iter()
        .flat_map(|trail| trail.nodes.iter())
        .map(|node| node.id.as_str())
        .collect::<HashSet<_>>();

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
            expect(
                node.name.len() <= 60 && !node.name.is_empty(),
                &format!("landmark {} has invalid name", node.id),
            )?;
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
    for event in &content.events {
        expect(
            event.text.len() <= 240 && !event.text.is_empty(),
            &format!("event {} text is invalid", event.id),
        )?;
        for condition in &event.conditions {
            validate_condition(
                condition,
                &trail_ids,
                &era_ids,
                &occupation_ids,
                &ailment_ids,
                &landmark_ids,
            )?;
        }
        validate_effects(&event.effects, &ailment_ids, &event_ids)?;
        for choice in &event.choices {
            expect(
                !choice.id.is_empty() && !choice.label.is_empty(),
                &format!("event {} has an invalid choice", event.id),
            )?;
            validate_effects(&choice.effects, &ailment_ids, &event_ids)?;
        }
        unique(event.choices.iter().map(|choice| choice.id.as_str()), "event choice")?;
    }
    for quote in &content.quotes {
        expect(
            quote.text.len() <= 240 && quote.text.contains(':'),
            &format!("quote {} must be named and concise", quote.id),
        )?;
        expect(
            !quote.seasons.is_empty(),
            &format!("quote {} needs season and state tags", quote.id),
        )?;
        if let Some(id) = &quote.landmark_id {
            expect(
                landmark_ids.contains(id.as_str()),
                &format!("quote {} names unknown landmark {}", quote.id, id),
            )?;
        }
    }
    Ok(())
}

fn validate_condition(
    condition: &Condition,
    trails: &HashSet<&str>,
    eras: &HashSet<&str>,
    occupations: &HashSet<&str>,
    ailments: &HashSet<&str>,
    landmarks: &HashSet<&str>,
) -> Result<(), ContentError> {
    match condition {
        Condition::All(items) | Condition::Any(items) => {
            expect(!items.is_empty(), "compound condition is empty")?;
            for item in items {
                validate_condition(item, trails, eras, occupations, ailments, landmarks)?;
            }
        }
        Condition::Not(item) => {
            validate_condition(item, trails, eras, occupations, ailments, landmarks)?
        }
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
        _ => {}
    }
    Ok(())
}

fn validate_effects(
    effects: &[Effect],
    ailments: &HashSet<&str>,
    events: &HashSet<&str>,
) -> Result<(), ContentError> {
    for effect in effects {
        match effect {
            Effect::InflictAilment(id) | Effect::HealAilment(id) => {
                expect(ailments.contains(id.as_str()), &format!("unknown ailment effect {id}"))?
            }
            Effect::Schedule { event_id, .. } => expect(
                events.contains(event_id.as_str()),
                &format!("scheduled event {event_id} does not exist"),
            )?,
            _ => {}
        }
    }
    Ok(())
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
    fn v1_catalogue_inventory_is_present() {
        let content = load().unwrap();
        assert_eq!(content.eras.len(), 4);
        assert_eq!(content.trails.len(), 3);
        assert!(content.occupations.len() >= 9);
        assert!(content.items.len() >= 10);
        assert!(content.ailments.len() >= 20);
        assert_eq!(content.events.len(), 26);
        assert_eq!(content.quotes.len(), 25);
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
}
