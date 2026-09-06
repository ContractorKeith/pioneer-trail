use crate::art::{self, ColorMode, Pixel};
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
        for rock in self.rocks.iter_mut().filter(|_| self.ticks.is_multiple_of(3)) {
            rock.y += 1;
            if rock.y > 22 {
                rock.y = -rando(&mut self.rng);
                rock.x = self.rng.stream("rafting-rocks").gen_range(4..76);
                rock.charged = false
            }
            if !rock.charged
                && rock.y + 2 > 19
                && rock.y < 22
                && rock.x < self.raft_x + 6
                && rock.x + 6 > self.raft_x - 6
            {
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
            KeyCode::Left | KeyCode::Char('h') => self.raft_x = (self.raft_x - 2).max(6),
            KeyCode::Right | KeyCode::Char('l') => self.raft_x = (self.raft_x + 2).min(74),
            KeyCode::Esc => {
                self.finished = true;
                self.aborted = true;
            }
            _ => {}
        }
    }
    /// Drive one accessible, turn-based rafting action without rendering the river sprites.
    pub fn text_key(&mut self, key: KeyCode) {
        if self.finished {
            return;
        }
        match key {
            KeyCode::Char('1') => {
                self.raft_x = 14;
                self.advance_turn();
            }
            KeyCode::Char('2') => {
                self.raft_x = 40;
                self.advance_turn();
            }
            KeyCode::Char('3') => {
                self.raft_x = 66;
                self.advance_turn();
            }
            KeyCode::Char(' ') => self.advance_turn(),
            KeyCode::Esc => {
                self.finished = true;
                self.aborted = true;
            }
            _ => {}
        }
    }
    /// Screen-reader-friendly state for the turn-based raft. It deliberately contains no art.
    pub fn text_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "TURN-BASED RAFT".into(),
            format!(
                "Time {}s | Lane {} | Cargo lost {} | Deaths {}",
                30u16.saturating_sub(self.ticks / 30),
                lane(self.raft_x),
                self.cargo,
                self.casualties
            ),
            "Choose 1 Left, 2 Center, or 3 Right. Space holds course one second. Esc aborts."
                .into(),
        ];
        let mut rocks = self
            .rocks
            .iter()
            .filter(|rock| !rock.charged && rock.y >= -30 && rock.y < 22)
            .collect::<Vec<_>>();
        rocks.sort_by_key(|rock| std::cmp::Reverse(rock.y));
        for rock in rocks.into_iter().take(5) {
            let seconds = ((19 - rock.y).max(0) + 9) / 10;
            lines.push(format!(
                "Rock {} lane, row {}, about {}s to raft row.",
                lane(rock.x),
                rock.y,
                seconds
            ));
        }
        if !lines.iter().any(|line| line.starts_with("Rock ")) {
            lines.push("No rocks are near the raft row.".into());
        }
        lines
    }
    fn advance_turn(&mut self) {
        for _ in 0..30 {
            self.tick();
            if self.finished {
                break;
            }
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
                if !(2..78).contains(&x) {
                    b[(a.x + x, a.y + y)].set_symbol("░").set_fg(mode.color(Pixel::Blue));
                }
            }
        }
        for rock in &self.rocks {
            let image = art::embedded("rock_0.px").expect("rock sprite");
            art::render_at(
                image,
                b,
                Rect::new(a.x, a.y, a.width.min(80), a.height.min(23)),
                i32::from(a.x) + i32::from(rock.x),
                i32::from(a.y) + i32::from(rock.y),
                mode,
            );
        }
        if a.height > 21 && self.raft_x < a.width as i16 {
            let image = art::embedded("raft.px").expect("raft sprite");
            art::render_at(
                image,
                b,
                Rect::new(a.x, a.y, a.width.min(80), a.height.min(23)),
                i32::from(a.x) + i32::from(self.raft_x) - 6,
                i32::from(a.y) + 19,
                mode,
            );
        }
        if a.height > 23 {
            b.set_stringn(
                a.x,
                a.y + 23,
                format!(
                    " RAFT {:02}s · CARGO LOST {} · DEATHS {} · ←→ steer · Esc return",
                    30 - self.ticks / 30,
                    self.cargo,
                    self.casualties
                ),
                a.width as usize,
                Style::default().fg(Color::White),
            );
        }
    }
}
fn rando(rng: &mut SimRng) -> i16 {
    rng.stream("rafting-rocks").gen_range(20..70)
}
fn lane(x: i16) -> &'static str {
    if x < 27 {
        "left"
    } else if x > 53 {
        "right"
    } else {
        "center"
    }
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
        assert_eq!(g.raft_x, 6)
    }
    #[test]
    fn collision_once() {
        let mut g = RaftingGame::new(1);
        g.rocks = vec![Rock { x: 40, y: 20, charged: false }];
        g.ticks = 2;
        g.tick();
        let first = g.result();
        g.tick();
        assert_eq!(g.result(), first)
    }
    #[test]
    fn text_rafting_is_deterministic_and_printable() {
        let mut a = RaftingGame::new(4);
        let mut b = RaftingGame::new(4);
        for key in ['1', '2', '3', ' ', '2', '1'] {
            a.text_key(KeyCode::Char(key));
            b.text_key(KeyCode::Char(key));
        }
        assert_eq!(a.raft_x, 14);
        assert_eq!(a.result(), b.result());
        assert_eq!(a.ticks, 180);
        assert!(a
            .text_lines()
            .iter()
            .all(|line| line.bytes().all(|byte| byte == b' ' || byte.is_ascii_graphic())));
    }
    #[test]
    fn text_rafting_prioritizes_the_nearest_uncharged_rocks() {
        let mut g = RaftingGame::new(1);
        g.rocks = vec![
            Rock { x: 14, y: -6, charged: false },
            Rock { x: 40, y: -5, charged: false },
            Rock { x: 66, y: -4, charged: false },
            Rock { x: 14, y: -3, charged: false },
            Rock { x: 40, y: -2, charged: false },
            Rock { x: 66, y: -1, charged: false },
            Rock { x: 40, y: 21, charged: true },
            Rock { x: 14, y: 21, charged: false },
        ];
        let lines = g.text_lines();
        assert!(lines.iter().any(|line| line.contains("left lane, row 21")));
        assert!(!lines.iter().any(|line| line.contains("center lane, row 21")));
    }
}
