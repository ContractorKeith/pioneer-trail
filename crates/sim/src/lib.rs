//! Pioneer Trail simulation core.
//!
//! Rules: no I/O, no terminal, fully deterministic given a seed.
//! The UI drives the sim exclusively through [`GameState::apply`].

pub mod rng;
pub mod state;

pub use state::{Command, GameState, Outcome};
