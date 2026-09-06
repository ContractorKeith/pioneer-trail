//! Typed, build-embedded content for Pioneer Trail.

use include_dir::{include_dir, Dir};
use pioneer_sim::{Condition, Effect, GameContent};
use std::collections::HashSet;
use thiserror::Error;

pub static ROOT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR");
const CONTENT_FILE: &str = "content.ron";

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("embedded {CONTENT_FILE} is missing")]
    Missing,
    #[error("could not parse {CONTENT_FILE}: {0}")]
    Parse(#[from] ron::error::SpannedError),
    #[error("content validation failed: {0}")]
    Validation(String),
}

/// Parse the RON included in the release binary and reject broken references.
pub fn load() -> Result<GameContent, ContentError> {
    let file = ROOT.get_file(CONTENT_FILE).ok_or(ContentError::Missing)?;
    let content = ron::from_str(file.contents_utf8().ok_or(ContentError::Missing)?)?;
    validate(&content)?;
    Ok(content)
}

/// Content is a single catalogue so every selected trail and era is embedded.
pub fn trail_files() -> Vec<&'static str> {
    vec![CONTENT_FILE]
}

pub fn validate(content: &GameContent) -> Result<(), ContentError> {
    expect(content.eras.len() == 4, "expected exactly four eras")?;
    expect(content.trails.len() == 3, "expected exactly three trails")?;
    expect(content.occupations.len() >= 9, "expected at least nine occupations")?;
    expect(content.items.len() >= 10, "expected at least ten store items")?;
    expect(content.ailments.len() >= 20, "expected at least twenty ailments")?;
    expect(content.events.len() >= 150, "expected at least 150 events")?;
    expect(content.quotes.len() >= 200, "expected at least 200 quotes")?;

    unique(content.eras.iter().map(|value| value.id.as_str()), "era")?;
    unique(content.trails.iter().map(|value| value.id.as_str()), "trail")?;
    unique(content.occupations.iter().map(|value| value.id.as_str()), "occupation")?;
    unique(content.items.iter().map(|value| value.id.as_str()), "item")?;
    unique(content.ailments.iter().map(|value| value.id.as_str()), "ailment")?;
    unique(content.events.iter().map(|value| value.id.as_str()), "event")?;
    unique(content.quotes.iter().map(|value| value.id.as_str()), "quote")?;

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
        }
    }
    for item in &content.items {
        expect(item.limit > 0, &format!("item {} has zero limit", item.id))?;
        expect(item.price_cents >= 0, &format!("item {} has negative price", item.id))?;
    }
    for event in &content.events {
        expect(event.weight > 0, &format!("event {} has zero weight", event.id))?;
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
    }
    for quote in &content.quotes {
        expect(
            quote.text.len() <= 240 && quote.text.contains(':'),
            &format!("quote {} must be named and concise", quote.id),
        )?;
        expect(
            !quote.seasons.is_empty() && !quote.state_tags.is_empty(),
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
}
