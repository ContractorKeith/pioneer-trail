use crate::art::{self, ColorMode};
use crossterm::event::KeyCode;
use pioneer_sim::rng::SimRng;
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};

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
    fn name(self) -> &'static str {
        match self {
            Self::Buffalo => "Buffalo",
            Self::Deer => "Deer",
            Self::Bear => "Bear",
            Self::Rabbit => "Rabbit",
            Self::Squirrel => "Squirrel",
        }
    }
    fn food(self) -> u16 {
        match self {
            Self::Buffalo => 100,
            Self::Deer => 55,
            Self::Bear => 65,
            Self::Rabbit => 8,
            Self::Squirrel => 3,
        }
    }
    fn sprite(self, frame: u16) -> String {
        let name = match self {
            Self::Buffalo => "buffalo",
            Self::Deer => "deer",
            Self::Bear => "bear",
            Self::Rabbit => "rabbit",
            Self::Squirrel => "squirrel",
        };
        format!("{name}_{}.px", frame % 2)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    pub fn new(seed: u64, biome: Biome, hunter: bool, ammo_available: u16) -> Self {
        let mut game = Self {
            rng: SimRng::new(seed),
            biome,
            targets: Vec::new(),
            crosshair: (40, 11),
            ticks: 0,
            shots: 0,
            ammo: ammo_available.min(20),
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
    /// Drive one accessible, turn-based action without rendering the pixel field.
    ///
    /// A numbered target is aimed at a visible cell in its existing sprite, then uses `shoot`
    /// so text mode shares normal ammo, corpse, bag-cap, and hit accounting.
    pub fn text_key(&mut self, key: KeyCode) {
        if self.finished {
            return;
        }
        match key {
            KeyCode::Char(number @ '1'..='5') => self.text_shoot((number as u8 - b'0') as usize),
            KeyCode::Char(' ') => self.advance_turn(),
            KeyCode::Esc => self.finished = true,
            _ => {}
        }
    }
    /// Screen-reader-friendly state for the turn-based hunt. It deliberately contains no art.
    pub fn text_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "TURN-BASED HUNT".into(),
            format!(
                "Time {}s | Shots {} | Ammo {} | Bag {}/{}",
                30u16.saturating_sub(self.ticks / TICKS_PER_SECOND),
                self.shots,
                self.ammo,
                self.food,
                self.capacity
            ),
            "Choose 1-5 to shoot a living animal. Space waits one second. Esc finishes.".into(),
        ];
        for (index, target) in self.targets.iter().filter(|target| target.alive).take(5).enumerate()
        {
            lines.push(format!(
                "{}. {}: {} lb meat",
                index + 1,
                target.kind.name(),
                target.kind.food()
            ));
        }
        if !self.targets.iter().any(|target| target.alive) {
            lines.push("No living animals remain.".into());
        }
        lines
    }
    fn text_shoot(&mut self, number: usize) {
        if self.ammo == 0 {
            return;
        }
        let Some(target) =
            self.targets.iter().filter(|target| target.alive).nth(number - 1).copied()
        else {
            return;
        };
        let Some(aim) = self.visible_cell(target) else {
            return;
        };
        self.crosshair = aim;
        self.shoot();
        self.advance_turn();
    }
    fn visible_cell(&self, target: Target) -> Option<(i16, i16)> {
        let image = art::embedded(&target.kind.sprite(self.ticks / 5)).expect("animal sprite");
        for y in 0..image.height() {
            for x in 0..image.width() {
                if image.pixel(x, y).is_none() {
                    continue;
                }
                let aim = (target.x - 8 + x as i16, target.y - 2 + (y / 2) as i16);
                if (0..WIDTH).contains(&aim.0) && (0..HEIGHT).contains(&aim.1) {
                    return Some(aim);
                }
            }
        }
        None
    }
    fn advance_turn(&mut self) {
        for _ in 0..TICKS_PER_SECOND {
            self.tick();
            if self.finished {
                break;
            }
        }
    }
    fn shoot(&mut self) {
        if self.ammo == 0 {
            return;
        }
        self.ammo -= 1;
        self.shots = self.shots.saturating_add(1);
        for target in &mut self.targets {
            let image = art::embedded(&target.kind.sprite(self.ticks / 5)).expect("animal sprite");
            let x = self.crosshair.0 - (target.x - 8);
            let y = self.crosshair.1 - (target.y - 2);
            if target.alive
                && x >= 0
                && y >= 0
                && (image.pixel(x as u16, y as u16 * 2).is_some()
                    || image.pixel(x as u16, y as u16 * 2 + 1).is_some())
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
    pub fn render(&self, buffer: &mut Buffer, area: Rect, mode: ColorMode) {
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
                let image =
                    art::embedded(&target.kind.sprite(self.ticks / 5)).expect("animal sprite");
                art::render_at(
                    image,
                    buffer,
                    Rect::new(area.x, area.y, area.width.min(80), area.height.min(23)),
                    i32::from(area.x) + i32::from(target.x) - 8,
                    i32::from(area.y) + i32::from(target.y) - 2,
                    mode,
                );
            }
        }
        if self.crosshair.0 < area.width as i16 && self.crosshair.1 < area.height as i16 {
            // The center marker is exactly the cell used by hit testing.
            buffer[(area.x + self.crosshair.0 as u16, area.y + self.crosshair.1 as u16)]
                .set_symbol("+")
                .set_fg(Color::White)
                .set_bg(Color::Black);
        }
        if area.height > 23 {
            buffer.set_stringn(
                area.x,
                area.y + 23,
                format!(
                    " HUNT {:02}s FOOD {}/{} AMMO {} · arrows: aim Space: fire Esc: return",
                    30 - self.ticks / TICKS_PER_SECOND,
                    self.food,
                    self.capacity,
                    self.ammo
                ),
                area.width as usize,
                Style::default().fg(Color::White),
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_world() {
        let mut a = HuntingGame::new(7, Biome::Plains, false, 20);
        let mut b = HuntingGame::new(7, Biome::Plains, false, 20);
        for _ in 0..100 {
            a.tick();
            b.tick()
        }
        assert_eq!(a.targets, b.targets);
        let mut left = Buffer::empty(Rect::new(0, 0, 80, 24));
        let mut right = Buffer::empty(Rect::new(0, 0, 80, 24));
        a.render(&mut left, Rect::new(0, 0, 80, 24), ColorMode::Ansi16);
        b.render(&mut right, Rect::new(0, 0, 80, 24), ColorMode::Ansi16);
        assert_eq!(left, right);
    }
    #[test]
    fn timer_and_escape_keep_earned_result() {
        let mut g = HuntingGame::new(1, Biome::Desert, false, 20);
        for _ in 0..900 {
            g.tick()
        }
        assert!(g.is_finished());
        assert_eq!(g.result().food_lbs, 0)
    }
    #[test]
    fn ammo_does_not_underflow() {
        let mut g = HuntingGame::new(1, Biome::Desert, false, 20);
        for _ in 0..1000 {
            g.handle_key(KeyCode::Char(' '))
        }
        assert_eq!(g.result().shots, 20)
    }
    #[test]
    fn a_dead_target_cannot_be_credited_twice() {
        let mut g = HuntingGame::new(1, Biome::Desert, false, 20);
        g.targets = vec![Target { kind: Animal::Rabbit, x: 40, y: 11, dx: 1, alive: true }];
        g.handle_key(KeyCode::Char(' '));
        g.handle_key(KeyCode::Char(' '));
        assert_eq!(g.result().food_lbs, 8);
    }
    #[test]
    fn text_controls_use_normal_shots_and_cannot_farm_corpses() {
        let mut g = HuntingGame::new(1, Biome::Desert, false, 20);
        g.targets = vec![Target { kind: Animal::Rabbit, x: 40, y: 11, dx: 1, alive: true }];
        g.text_key(KeyCode::Char('1'));
        assert_eq!(g.result(), HuntingResult { food_lbs: 8, shots: 1 });
        assert_eq!(g.ticks, 30);
        g.text_key(KeyCode::Char('1'));
        assert_eq!(g.result(), HuntingResult { food_lbs: 8, shots: 1 });
        assert_eq!(g.ticks, 30);
    }
    #[test]
    fn text_hunt_time_and_ammo_are_finite_and_printable() {
        let mut g = HuntingGame::new(1, Biome::Desert, false, 1);
        g.targets = vec![Target { kind: Animal::Rabbit, x: 40, y: 11, dx: 1, alive: true }];
        g.text_key(KeyCode::Char('1'));
        for _ in 0..30 {
            g.text_key(KeyCode::Char(' '));
        }
        assert!(g.is_finished());
        assert_eq!(g.result().shots, 1);
        assert!(g
            .text_lines()
            .iter()
            .all(|line| line.bytes().all(|byte| byte == b' ' || byte.is_ascii_graphic())));
    }
}
