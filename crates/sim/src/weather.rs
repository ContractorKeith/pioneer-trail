use crate::rng::SimRng;
use crate::{Season, WeatherKind};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Terrain {
    Plains,
    Hills,
    Mountains,
    Desert,
    RiverValley,
    Forest,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClimateZone {
    Temperate,
    Mountain,
    Arid,
    Pacific,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct WeatherState {
    pub kind: WeatherKind,
    pub days_remaining: u8,
    pub river_depth_bonus: i8,
}
impl Default for WeatherState {
    fn default() -> Self {
        Self { kind: WeatherKind::Clear, days_remaining: 1, river_depth_bonus: 0 }
    }
}
impl WeatherState {
    pub fn advance(&mut self, rng: &mut SimRng, season: Season) {
        self.advance_in(rng, season, ClimateZone::Temperate);
    }

    /// Advance weather using a regional climate while retaining 1–5 day weather states.
    pub fn advance_in(&mut self, rng: &mut SimRng, season: Season, climate: ClimateZone) {
        if self.days_remaining > 1 {
            self.days_remaining -= 1;
            return;
        }
        let roll = rng.stream("weather").gen_range(0..100);
        self.kind = match (climate, season, roll) {
            (ClimateZone::Mountain, Season::Winter, 0..55)
            | (ClimateZone::Mountain, Season::Spring, 0..30)
            | (ClimateZone::Mountain, Season::Autumn, 0..25) => WeatherKind::Snow,
            (ClimateZone::Mountain, _, 0..55) => WeatherKind::Cold,
            (ClimateZone::Arid, Season::Summer, 0..50) => WeatherKind::Hot,
            (ClimateZone::Arid, Season::Spring | Season::Autumn, 0..28) => WeatherKind::Hot,
            (ClimateZone::Arid, _, 0..10) => WeatherKind::Storm,
            (ClimateZone::Arid, _, 10..18) => WeatherKind::Rain,
            (ClimateZone::Arid, _, 18..28) => WeatherKind::Cold,
            (ClimateZone::Arid, _, 28..52) => WeatherKind::Warm,
            (ClimateZone::Pacific, Season::Winter, 0..20) => WeatherKind::Snow,
            (ClimateZone::Pacific, _, 0..26) => WeatherKind::Storm,
            (ClimateZone::Pacific, _, 26..60) => WeatherKind::Rain,
            (ClimateZone::Pacific, _, 60..70) => WeatherKind::Cold,
            (ClimateZone::Pacific, _, 70..85) => WeatherKind::Warm,
            (_, Season::Winter, 0..35) => WeatherKind::Snow,
            (_, Season::Summer, 0..20) => WeatherKind::Hot,
            (_, _, 0..18) => WeatherKind::Storm,
            (_, _, 18..42) => WeatherKind::Rain,
            (_, _, 42..56) => WeatherKind::Cold,
            (_, _, 56..70) => WeatherKind::Warm,
            _ => WeatherKind::Clear,
        };
        self.days_remaining = rng.stream("weather").gen_range(1..=5);
        self.river_depth_bonus = match self.kind {
            WeatherKind::Storm => 2,
            WeatherKind::Rain => 1,
            WeatherKind::Snow => 1,
            _ => 0,
        };
    }
    pub fn travel_penalty(self, terrain: Terrain) -> u32 {
        let weather = match self.kind {
            WeatherKind::Storm | WeatherKind::Snow => 7,
            WeatherKind::Rain | WeatherKind::Cold => 3,
            WeatherKind::Hot => 2,
            _ => 0,
        };
        weather
            + match terrain {
                Terrain::Mountains => 5,
                Terrain::Hills | Terrain::Forest => 2,
                Terrain::Desert => 3,
                _ => 0,
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regional_weather_changes_seasonal_odds_without_changing_the_legacy_wrapper() {
        for seed in 0..32 {
            let mut legacy = WeatherState::default();
            let mut temperate = WeatherState::default();
            legacy.advance(&mut SimRng::new(seed), Season::Summer);
            temperate.advance_in(&mut SimRng::new(seed), Season::Summer, ClimateZone::Temperate);
            assert_eq!(legacy, temperate);
        }

        let mut mountain_cold = false;
        let mut arid_hot = false;
        for seed in 0..64 {
            let mut mountain = WeatherState::default();
            let mut arid = WeatherState::default();
            mountain.advance_in(&mut SimRng::new(seed), Season::Summer, ClimateZone::Mountain);
            arid.advance_in(&mut SimRng::new(seed), Season::Summer, ClimateZone::Arid);
            mountain_cold |= matches!(mountain.kind, WeatherKind::Cold | WeatherKind::Snow);
            arid_hot |= arid.kind == WeatherKind::Hot;
        }
        assert!(mountain_cold, "mountain summers retain cold-weather outcomes");
        assert!(arid_hot, "arid summers retain hot-weather outcomes");
    }
}
