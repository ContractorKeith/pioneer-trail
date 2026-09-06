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
        if self.days_remaining > 1 {
            self.days_remaining -= 1;
            return;
        }
        let roll = rng.stream("weather").gen_range(0..100);
        self.kind = match season {
            Season::Winter if roll < 35 => WeatherKind::Snow,
            Season::Summer if roll < 20 => WeatherKind::Hot,
            _ if roll < 18 => WeatherKind::Storm,
            _ if roll < 42 => WeatherKind::Rain,
            _ if roll < 56 => WeatherKind::Cold,
            _ if roll < 70 => WeatherKind::Warm,
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
