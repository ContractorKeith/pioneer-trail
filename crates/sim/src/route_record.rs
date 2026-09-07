//! Factual route history and read-only supply-stop lookup.
use crate::GameState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Visit {
    pub landmark_id: String,
    pub day: u32,
    pub mile: u32,
}

impl GameState {
    /// Closest reachable store along the chosen leg, then any still-open forks.
    pub fn next_supply_stop(&self) -> Option<(&str, u32)> {
        let trail = self.content.trails.iter().find(|t| Some(&t.id) == self.trail_id.as_ref())?;
        let current = self.current_node_id.as_deref()?;
        let mut distances = BTreeMap::new();
        if let Some(target) = &self.target_node_id {
            distances.insert(target.as_str(), self.route_miles_remaining);
        } else {
            for route in &trail.nodes.iter().find(|n| n.id == current)?.routes {
                distances.insert(route.target_id.as_str(), route.distance_miles);
            }
        }
        let mut done = std::collections::BTreeSet::new();
        while let Some((id, distance)) = distances
            .iter()
            .filter(|(id, _)| !done.contains(**id))
            .min_by_key(|(_, d)| **d)
            .map(|(id, d)| (*id, *d))
        {
            done.insert(id);
            let node = trail.nodes.iter().find(|n| n.id == id)?;
            let unavailable = self
                .era_id
                .as_ref()
                .and_then(|id| self.content.era_rules.get(id))
                .is_some_and(|era| era.unavailable_stores.contains(&node.id));
            if node.store && !unavailable && id != current {
                return Some((&node.name, distance));
            }
            for route in &node.routes {
                let distance = distance.saturating_add(route.distance_miles);
                distances
                    .entry(route.target_id.as_str())
                    .and_modify(|d| *d = (*d).min(distance))
                    .or_insert(distance);
            }
        }
        None
    }
}
