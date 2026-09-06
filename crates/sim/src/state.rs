//! Game state and the single mutation entry point.
//! Phase 1 fills this in; the skeleton exists so the workspace compiles.

use crate::rng::SimRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub rng: SimRng,
    pub miles: u32,
    pub day: u32,
}

/// Every player action. The UI never mutates state directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Continue,
}

/// What the UI should show after a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Message(String),
}

impl GameState {
    pub fn new(seed: u64) -> Self {
        Self { rng: SimRng::new(seed), miles: 0, day: 0 }
    }

    pub fn apply(&mut self, cmd: Command) -> Vec<Outcome> {
        match cmd {
            Command::Continue => self.tick_day(),
        }
    }

    /// Advance one day. Placeholder until Phase 1 travel/supplies/health land.
    pub fn tick_day(&mut self) -> Vec<Outcome> {
        self.day += 1;
        self.miles += 15;
        vec![Outcome::Message(format!("Day {}: {} miles", self.day, self.miles))]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn miles_monotonic() {
        let mut g = GameState::new(1);
        let before = g.miles;
        g.apply(Command::Continue);
        assert!(g.miles >= before);
    }
}
