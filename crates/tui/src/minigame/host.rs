//! UI worlds consume input; only validated result commands change the journey.
use super::{
    hunting::{Biome, HuntingGame},
    rafting::RaftingGame,
};
use crate::art::ColorMode;
use crossterm::event::KeyCode;
use pioneer_sim::{Command, GameState, MinigameKind, Terrain};
use ratatui::{buffer::Buffer, layout::Rect};

pub enum LiveMinigame {
    Hunt(HuntingGame),
    Raft(RaftingGame),
}
impl LiveMinigame {
    pub fn from_session(game: &GameState) -> Option<Self> {
        let session = game.active_minigame.as_ref()?;
        Some(match session.kind {
            MinigameKind::Hunt => {
                let biome = match game.terrain() {
                    Terrain::Mountains => Biome::Mountains,
                    Terrain::Forest | Terrain::Hills => Biome::Forest,
                    Terrain::Desert => Biome::Desert,
                    Terrain::RiverValley => Biome::RiverValley,
                    Terrain::Plains => Biome::Plains,
                };
                Self::Hunt(HuntingGame::new(
                    session.seed,
                    biome,
                    game.occupation_id.as_deref() == Some("hunter"),
                    session.ammo_available.min(20) as u16,
                ))
            }
            MinigameKind::Raft => Self::Raft(RaftingGame::new(session.seed)),
        })
    }
    pub fn key(&mut self, key: KeyCode) {
        match self {
            Self::Hunt(g) => g.handle_key(key),
            Self::Raft(g) => g.handle_key(key),
        }
    }
    pub fn tick(&mut self) {
        match self {
            Self::Hunt(g) => g.tick(),
            Self::Raft(g) => g.tick(),
        }
    }
    pub fn result(&self) -> Option<Command> {
        match self {
            Self::Hunt(g) if g.is_finished() => {
                let result = g.result();
                Some(Command::HuntResult {
                    food_lbs: result.food_lbs.into(),
                    shots: result.shots.into(),
                })
            }
            Self::Raft(g) if g.is_finished() => {
                let result = g.result();
                Some(Command::RaftResult {
                    cargo_lost_lbs: result.cargo_lost_lbs.into(),
                    casualties: result.casualties,
                    completed: result.completed,
                })
            }
            _ => None,
        }
    }
    pub fn render(&self, buffer: &mut Buffer, area: Rect, mode: ColorMode) {
        match self {
            Self::Hunt(g) => g.render(buffer, area, mode),
            Self::Raft(g) => g.render(buffer, area, mode),
        }
    }
}
