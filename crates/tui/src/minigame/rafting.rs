use crate::art::{self, ColorMode, PxImage};
use crossterm::event::KeyCode;
use pioneer_sim::rng::SimRng;
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaftingResult {
    pub cargo_lost_lbs: u16,
    pub casualties: u8,
    pub completed: bool,
    pub aborted: bool,
}
#[derive(Clone, Copy, Debug)]
struct Rock {
    x: i16,
    y: i16,
    charged: bool,
}
/// A deterministic 30fps downstream course. The raft can be steered with arrows or hjkl.
pub struct RaftingGame {
    rng: SimRng,
    rocks: Vec<Rock>,
    raft_x: i16,
    ticks: u16,
    cargo: u16,
    casualties: u8,
    finished: bool,
    aborted: bool,
}
impl RaftingGame {
    pub fn new(seed: u64) -> Self {
        let mut game = Self {
            rng: SimRng::new(seed),
            rocks: Vec::new(),
            raft_x: 40,
            ticks: 0,
            cargo: 0,
            casualties: 0,
            finished: false,
            aborted: false,
        };
        game.spawn();
        game
    }
    fn spawn(&mut self) {
        let r = self.rng.stream("rafting-rocks");
        for _ in 0..8 {
            self.rocks.push(Rock { x: r.gen_range(4..76), y: r.gen_range(-70..0), charged: false })
        }
    }
    pub fn tick(&mut self) {
        if self.finished {
            return;
        }
        self.ticks += 1;
        for rock in &mut self.rocks {
            rock.y += 1;
            if rock.y > 22 {
                rock.y = -rando(&mut self.rng);
                rock.x = self.rng.stream("rafting-rocks").gen_range(4..76);
                rock.charged = false
            }
            if !rock.charged && rock.y >= 20 && (rock.x - self.raft_x).abs() <= 3 {
                rock.charged = true;
                self.cargo = (self.cargo + 15).min(120);
                if self.cargo.is_multiple_of(45) {
                    self.casualties = (self.casualties + 1).min(4)
                }
            }
        }
        if self.ticks >= 900 {
            self.finished = true
        }
    }
    pub fn handle_key(&mut self, key: KeyCode) {
        if self.finished {
            return;
        }
        match key {
            KeyCode::Left | KeyCode::Char('h') => self.raft_x = (self.raft_x - 2).max(3),
            KeyCode::Right | KeyCode::Char('l') => self.raft_x = (self.raft_x + 2).min(76),
            KeyCode::Esc => {
                self.finished = true;
                self.aborted = true;
            }
            _ => {}
        }
    }
    pub fn is_finished(&self) -> bool {
        self.finished
    }
    pub fn result(&self) -> RaftingResult {
        RaftingResult {
            cargo_lost_lbs: self.cargo,
            casualties: self.casualties,
            completed: self.finished && !self.aborted && self.ticks >= 900,
            aborted: self.aborted,
        }
    }
    pub fn render(&self, b: &mut Buffer, area: Rect, mode: ColorMode) {
        let a = area.intersection(b.area);
        for y in 0..a.height.min(24) {
            for x in 0..a.width.min(80) {
                b.cell_mut((a.x + x, a.y + y)).unwrap().set_symbol(" ").set_bg(Color::Black);
            }
        }
        for rock in &self.rocks {
            if rock.y >= 0 && rock.y < a.height as i16 && rock.x < a.width as i16 {
                render_asset(
                    "rock_0.px",
                    b,
                    Rect::new(a.x + rock.x as u16, a.y + rock.y as u16, 6, 2),
                    mode,
                );
            }
        }
        if a.height > 21 && self.raft_x < a.width as i16 {
            render_asset("raft.px", b, Rect::new(a.x + self.raft_x as u16, a.y + 21, 12, 3), mode);
        }
        if a.height > 23 {
            b.set_stringn(
                a.x,
                a.y + 23,
                format!(" RAFT  CARGO LOST {}  CASUALTIES {} ", self.cargo, self.casualties),
                a.width as usize,
                Style::default().fg(Color::White),
            );
        }
    }
}
fn render_asset(name: &str, b: &mut Buffer, a: Rect, mode: ColorMode) {
    if let Some(file) = pioneer_data::ART.get_file(name) {
        if let Ok(image) = PxImage::parse(file.contents_utf8().unwrap()) {
            art::render(&image, b, a, mode)
        }
    }
}
fn rando(rng: &mut SimRng) -> i16 {
    rng.stream("rafting-rocks").gen_range(20..70)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic() {
        let mut a = RaftingGame::new(4);
        let mut b = RaftingGame::new(4);
        for _ in 0..901 {
            a.tick();
            b.tick()
        }
        assert_eq!(a.result(), b.result());
        assert!(a.is_finished())
    }
    #[test]
    fn bounds() {
        let mut g = RaftingGame::new(1);
        for _ in 0..99 {
            g.handle_key(KeyCode::Left)
        }
        assert_eq!(g.raft_x, 3)
    }
    #[test]
    fn collision_once() {
        let mut g = RaftingGame::new(1);
        g.rocks = vec![Rock { x: 40, y: 20, charged: false }];
        g.tick();
        let first = g.result();
        g.tick();
        assert_eq!(g.result(), first)
    }
}
