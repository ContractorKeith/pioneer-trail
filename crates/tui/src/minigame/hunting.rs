use crossterm::event::KeyCode;
use pioneer_sim::rng::SimRng;
use rand::Rng;
use ratatui::{buffer::Buffer, layout::Rect, style::Color};

pub const TICKS_PER_SECOND: u16 = 30;
const WIDTH: i16 = 80;
const HEIGHT: i16 = 23;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Biome {
    Plains,
    Forest,
    Mountains,
    Desert,
    RiverValley,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HuntingResult {
    pub food_lbs: u16,
    pub shots: u16,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Animal {
    Buffalo,
    Deer,
    Bear,
    Rabbit,
    Squirrel,
}
impl Animal {
    fn food(self) -> u16 {
        match self {
            Self::Buffalo => 100,
            Self::Deer => 55,
            Self::Bear => 65,
            Self::Rabbit => 8,
            Self::Squirrel => 3,
        }
    }
    fn glyph(self) -> &'static str {
        match self {
            Self::Buffalo => "B",
            Self::Deer => "D",
            Self::Bear => "A",
            Self::Rabbit => "R",
            Self::Squirrel => "S",
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct Target {
    kind: Animal,
    x: i16,
    y: i16,
    dx: i16,
    alive: bool,
}

/// Top-down 80x23 hunting field. `tick` is called once per UI 30fps tick.
pub struct HuntingGame {
    rng: SimRng,
    biome: Biome,
    targets: Vec<Target>,
    crosshair: (i16, i16),
    ticks: u16,
    shots: u16,
    ammo: u16,
    food: u16,
    capacity: u16,
    finished: bool,
}
impl HuntingGame {
    pub fn new(seed: u64, biome: Biome, hunter: bool) -> Self {
        let mut game = Self {
            rng: SimRng::new(seed),
            biome,
            targets: Vec::new(),
            crosshair: (40, 11),
            ticks: 0,
            shots: 0,
            ammo: 20,
            food: 0,
            capacity: if hunter { 200 } else { 150 },
            finished: false,
        };
        game.spawn();
        game
    }
    fn spawn(&mut self) {
        let stream = self.rng.stream("hunting-spawn");
        let count = 3 + stream.gen_range(0..3);
        for _ in 0..count {
            let kind = match self.biome {
                Biome::Plains => {
                    if stream.gen_bool(0.35) {
                        Animal::Buffalo
                    } else {
                        Animal::Deer
                    }
                }
                Biome::Forest => {
                    if stream.gen_bool(0.4) {
                        Animal::Deer
                    } else {
                        Animal::Bear
                    }
                }
                Biome::Mountains => Animal::Deer,
                Biome::Desert => Animal::Rabbit,
                Biome::RiverValley => Animal::Squirrel,
            };
            self.targets.push(Target {
                kind,
                x: stream.gen_range(3..77),
                y: stream.gen_range(2..21),
                dx: if stream.gen_bool(0.5) { 1 } else { -1 },
                alive: true,
            });
        }
    }
    pub fn tick(&mut self) {
        if self.finished {
            return;
        }
        self.ticks = self.ticks.saturating_add(1);
        if self.ticks.is_multiple_of(6) {
            for target in &mut self.targets {
                if target.alive {
                    target.x += target.dx;
                    if target.x <= 1 || target.x >= 78 {
                        target.dx = -target.dx
                    }
                }
            }
        }
        if self.ticks >= 30 * TICKS_PER_SECOND {
            self.finished = true
        }
    }
    pub fn handle_key(&mut self, key: KeyCode) {
        if self.finished {
            return;
        }
        match key {
            KeyCode::Left | KeyCode::Char('h') => self.crosshair.0 = (self.crosshair.0 - 1).max(0),
            KeyCode::Right | KeyCode::Char('l') => {
                self.crosshair.0 = (self.crosshair.0 + 1).min(WIDTH - 1)
            }
            KeyCode::Up | KeyCode::Char('k') => self.crosshair.1 = (self.crosshair.1 - 1).max(0),
            KeyCode::Down | KeyCode::Char('j') => {
                self.crosshair.1 = (self.crosshair.1 + 1).min(HEIGHT - 1)
            }
            KeyCode::Char(' ') => self.shoot(),
            KeyCode::Esc => self.finished = true,
            _ => {}
        }
    }
    fn shoot(&mut self) {
        if self.ammo == 0 {
            return;
        }
        self.ammo -= 1;
        self.shots = self.shots.saturating_add(1);
        for target in &mut self.targets {
            if target.alive
                && (target.x - self.crosshair.0).abs() <= 1
                && (target.y - self.crosshair.1).abs() <= 1
            {
                target.alive = false;
                self.food = (self.food + target.kind.food()).min(self.capacity);
                break;
            }
        }
        if self.food >= self.capacity {
            self.finished = true
        }
    }
    pub fn is_finished(&self) -> bool {
        self.finished
    }
    pub fn result(&self) -> HuntingResult {
        HuntingResult { food_lbs: self.food, shots: self.shots }
    }
    pub fn render(&self, buffer: &mut Buffer, area: Rect) {
        let area = area.intersection(buffer.area);
        for y in 0..area.height.min(23) {
            for x in 0..area.width.min(80) {
                let cell = buffer.cell_mut((area.x + x, area.y + y)).unwrap();
                cell.set_symbol(if y == 22 { "▄" } else { " " })
                    .set_fg(Color::Green)
                    .set_bg(Color::Black);
            }
        }
        for target in self.targets.iter().filter(|t| t.alive) {
            if target.x >= 0
                && target.y >= 0
                && target.x < area.width as i16
                && target.y < area.height as i16
            {
                buffer
                    .cell_mut((area.x + target.x as u16, area.y + target.y as u16))
                    .unwrap()
                    .set_symbol(target.kind.glyph())
                    .set_fg(Color::LightRed);
            }
        }
        if self.crosshair.0 < area.width as i16 && self.crosshair.1 < area.height as i16 {
            buffer
                .cell_mut((area.x + self.crosshair.0 as u16, area.y + self.crosshair.1 as u16))
                .unwrap()
                .set_symbol("+")
                .set_fg(Color::White);
        }
        if area.height > 23 {
            buffer
                .cell_mut((area.x, area.y + 23))
                .unwrap()
                .set_symbol(&format!(
                    " HUNT  {:02}s  FOOD {}/{}  AMMO {} ",
                    30 - self.ticks / TICKS_PER_SECOND,
                    self.food,
                    self.capacity,
                    self.ammo
                ))
                .set_fg(Color::White);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_world() {
        let mut a = HuntingGame::new(7, Biome::Plains, false);
        let mut b = HuntingGame::new(7, Biome::Plains, false);
        for _ in 0..100 {
            a.tick();
            b.tick()
        }
        assert_eq!(a.result(), b.result())
    }
    #[test]
    fn timer_and_escape_keep_earned_result() {
        let mut g = HuntingGame::new(1, Biome::Desert, false);
        for _ in 0..900 {
            g.tick()
        }
        assert!(g.is_finished());
        assert_eq!(g.result().food_lbs, 0)
    }
    #[test]
    fn ammo_does_not_underflow() {
        let mut g = HuntingGame::new(1, Biome::Desert, false);
        for _ in 0..1000 {
            g.handle_key(KeyCode::Char(' '))
        }
        assert_eq!(g.result().shots, 20)
    }
}
