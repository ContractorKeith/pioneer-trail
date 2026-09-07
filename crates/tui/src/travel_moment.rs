//! Presentation-only pauses between travel days. These never touch simulation state or RNG.

use pioneer_sim::{Terrain, WeatherKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TravelMoment {
    pub title: &'static str,
    pub text: &'static str,
    pub wildlife_sprite: Option<&'static str>,
}

/// Pick an infrequent, deterministic observation from already-resolved travel state.
///
/// This is deliberately arithmetic rather than random: opening or skipping a moment cannot
/// alter the simulation's named random streams or consume another day.
pub fn after_travel_day(
    day: u32,
    weather: WeatherKind,
    terrain: Terrain,
    last_moment_day: Option<u32>,
) -> Option<TravelMoment> {
    if day == 0
        || !day.is_multiple_of(6)
        || last_moment_day.is_some_and(|last| day.saturating_sub(last) < 6)
    {
        return None;
    }
    let moment = match (weather, terrain) {
        (WeatherKind::Rain | WeatherKind::Storm, _) => TravelMoment {
            title: "RAIN ON THE CANVAS",
            text: "Rain beads on the wagon cover. The team keeps its patient pace.",
            wildlife_sprite: None,
        },
        (WeatherKind::Snow | WeatherKind::Cold, _) => TravelMoment {
            title: "COLD MORNING",
            text: "Frost whitens the ruts before the first wheel turns.",
            wildlife_sprite: None,
        },
        (WeatherKind::Hot, Terrain::Desert) => TravelMoment {
            title: "HEAT SHIMMER",
            text: "The far road wavers in the heat. Your wagon holds its line.",
            wildlife_sprite: None,
        },
        (_, Terrain::Forest) => TravelMoment {
            title: "DEER IN THE TIMBER",
            text: "A deer slips between the pines and is gone before anyone speaks.",
            wildlife_sprite: Some("deer_0.px"),
        },
        (_, Terrain::Mountains | Terrain::Hills) => TravelMoment {
            title: "HIGH GROUND",
            text: "Loose stone clicks beneath the wheels. The ridge opens ahead.",
            wildlife_sprite: None,
        },
        _ => TravelMoment {
            title: "SQUIRREL IN THE GRASS",
            text: "A squirrel darts across the wheel ruts and vanishes into the grass.",
            wildlife_sprite: Some("squirrel_0.px"),
        },
    };
    Some(moment)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moments_are_spaced_after_resuming_and_do_not_need_rng() {
        assert!(after_travel_day(3, WeatherKind::Clear, Terrain::Plains, None).is_none());
        assert!(after_travel_day(6, WeatherKind::Clear, Terrain::Plains, Some(6)).is_none());
        assert!(after_travel_day(7, WeatherKind::Clear, Terrain::Plains, None).is_none());
        assert!(after_travel_day(11, WeatherKind::Clear, Terrain::Plains, None).is_none());
        assert_eq!(
            after_travel_day(12, WeatherKind::Rain, Terrain::Plains, Some(6)).unwrap().title,
            "RAIN ON THE CANVAS"
        );
    }
}
