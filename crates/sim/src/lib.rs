//! Pioneer Trail simulation core.
//!
//! Rules: no I/O, no terminal, fully deterministic given a seed.
//! The UI drives the sim exclusively through [`GameState::apply`].

pub mod calendar;
pub mod content;
pub mod economy;
pub mod health;
pub mod party;
pub mod rng;
pub mod score;
pub mod state;
pub mod weather;

pub use calendar::{days_in_month, is_leap_year, CalendarDate};
pub use content::*;
pub use economy::Market;
pub use party::{Relationships, Skills, Trait};
pub use state::{
    Command, CommandError, CrossMethod, GameState, Outcome, Pace, RationLevel, RunStatus,
};
pub use weather::{Terrain, WeatherState};
