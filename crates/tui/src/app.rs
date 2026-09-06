use crate::{
    input::{decode, Input},
    persist::{new_run_id, Settings, Storage},
    screens::Screen,
};
use crossterm::event::KeyEvent;
use crossterm::event::{self, Event};
use pioneer_sim::{Command, CrossMethod, GameContent, GameState, Outcome, Pace, RationLevel};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

/// UI-only draft; all rules are submitted to the simulation as commands.
#[derive(Debug, Clone)]
struct SetupDraft {
    trail: usize,
    era: usize,
    occupation: usize,
    names: [String; 5],
    month: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinigameRequest {
    Hunt,
    Raft,
}
impl Default for SetupDraft {
    fn default() -> Self {
        Self {
            trail: 0,
            era: 0,
            occupation: 0,
            names: [
                "Leader".into(),
                "Traveler 2".into(),
                "Traveler 3".into(),
                "Traveler 4".into(),
                "Traveler 5".into(),
            ],
            month: 4,
        }
    }
}

pub struct App {
    pub game: GameState,
    pub settings: Settings,
    pub screen: Screen,
    cursor: usize,
    draft: SetupDraft,
    log: Vec<String>,
    pending_event: Option<String>,
    quit: bool,
    storage: Option<Storage>,
    run_id: String,
    hall: Vec<String>,
    minigame_request: Option<MinigameRequest>,
}
impl App {
    pub fn new(content: GameContent, seed: u64, settings: Settings) -> Self {
        Self {
            game: GameState::with_content(seed, content),
            settings,
            screen: Screen::Title,
            cursor: 0,
            draft: SetupDraft::default(),
            log: vec!["Choose New Journey to begin.".into()],
            pending_event: None,
            quit: false,
            storage: None,
            run_id: new_run_id(),
            hall: Vec::new(),
            minigame_request: None,
        }
    }
    pub fn with_storage(mut self, storage: Storage) -> Self {
        self.storage = Some(storage);
        self
    }
    pub fn should_quit(&self) -> bool {
        self.quit
    }
    /// Host code takes this request and runs the real-time screen, then submits its result command.
    pub fn take_minigame_request(&mut self) -> Option<MinigameRequest> {
        self.minigame_request.take()
    }
    pub fn handle_key(&mut self, key: KeyEvent) {
        match decode(key) {
            Input::Quit => self.quit = true,
            Input::Up => self.cursor = self.cursor.saturating_sub(1),
            Input::Down => self.cursor = self.cursor.saturating_add(1),
            Input::Digit(n) if n > 0 => {
                self.cursor = usize::from(n - 1);
                self.select();
            }
            Input::Select => self.select(),
            Input::Back => self.back(),
            Input::Left => self.adjust(-1),
            Input::Right => self.adjust(1),
            Input::Character(c) => self.character(c),
            Input::None | Input::Digit(_) => {}
        }
    }
    fn select(&mut self) {
        match self.screen {
            Screen::Title => match self.cursor % 6 {
                0 => {
                    self.screen = Screen::SetupTrail;
                    self.cursor = 0;
                }
                1 => self.continue_saved(),
                2 => self.load_hall(),
                3 => self.screen = Screen::Seed,
                4 => self.screen = Screen::Settings,
                _ => self.quit = true,
            },
            Screen::SetupTrail => {
                self.draft.trail =
                    self.cursor.min(self.game.content.trails.len().saturating_sub(1));
                self.cursor = 0;
                self.screen = Screen::SetupOccupation;
            }
            Screen::SetupOccupation => {
                self.draft.occupation =
                    self.cursor.min(self.game.content.occupations.len().saturating_sub(1));
                self.cursor = 0;
                self.screen = Screen::SetupParty;
            }
            Screen::SetupParty => {
                if self.cursor >= 5 {
                    self.cursor = 0;
                    self.screen = Screen::SetupDeparture;
                } else {
                    self.cursor += 1;
                }
            }
            Screen::SetupDeparture => {
                let Some(trail) = self.game.content.trails.get(self.draft.trail) else {
                    self.note("No trail content loaded.");
                    return;
                };
                let Some(era) = self.game.content.eras.get(self.draft.era) else {
                    self.note("No era content loaded.");
                    return;
                };
                let Some(job) = self.game.content.occupations.get(self.draft.occupation) else {
                    self.note("No occupations loaded.");
                    return;
                };
                self.apply(Command::Configure {
                    trail_id: trail.id.clone(),
                    era_id: era.id.clone(),
                    occupation_id: job.id.clone(),
                    party: self.draft.names.to_vec(),
                    departure_month: self.draft.month,
                });
                self.screen = Screen::Store;
                self.cursor = 0;
            }
            Screen::Store => {
                if self.cursor < self.game.content.items.len() {
                    let id = self.game.content.items[self.cursor].id.clone();
                    self.apply(Command::Buy { item_id: id, quantity: 1 });
                } else {
                    self.apply(Command::Depart);
                    self.screen = Screen::Journey;
                }
            }
            Screen::Journey => match self.cursor % 9 {
                0 => self.apply(Command::Continue),
                1 => self.screen = Screen::Supplies,
                2 => self.screen = Screen::Map,
                3 => self.screen = Screen::Pace,
                4 => self.screen = Screen::Rations,
                5 => self.screen = Screen::Rest,
                6 => {
                    self.minigame_request = Some(MinigameRequest::Hunt);
                    self.screen = Screen::Minigame;
                }
                7 => self.screen = Screen::Talk,
                _ => {
                    if self.game.can_shop() {
                        self.screen = Screen::Store
                    } else {
                        self.note("There is no store here.")
                    }
                }
            },
            Screen::Pace => {
                let pace = [Pace::Steady, Pace::Strenuous, Pace::Grueling][self.cursor % 3];
                self.apply(Command::SetPace(pace));
                self.screen = Screen::Journey;
            }
            Screen::Rations => {
                let rations = [RationLevel::Filling, RationLevel::Meager, RationLevel::BareBones]
                    [self.cursor % 3];
                self.apply(Command::SetRations(rations));
                self.screen = Screen::Journey;
            }
            Screen::Rest => {
                self.apply(Command::Rest { days: (self.cursor + 1) as u32 });
                self.screen = Screen::Journey;
            }
            Screen::Talk => {
                self.apply(Command::Talk);
                self.screen = Screen::Journey;
            }
            Screen::Fork => {
                if let Some(node) = self.game.current_landmark() {
                    if let Some(route) = node.routes.get(self.cursor % node.routes.len().max(1)) {
                        self.apply(Command::ChooseRoute { route_id: route.id.clone() });
                        self.screen = Screen::Journey;
                    }
                }
            }
            Screen::River => {
                let method = [
                    CrossMethod::Ford,
                    CrossMethod::Caulk,
                    CrossMethod::Ferry,
                    CrossMethod::Wait,
                    CrossMethod::Guide,
                ][self.cursor % 5];
                self.apply(Command::CrossRiver { method });
                if !matches!(method, CrossMethod::Wait) {
                    self.screen = Screen::Journey;
                }
            }
            Screen::Event => {
                if let Some(id) = self.pending_event.clone() {
                    if let Some(event) =
                        self.game.content.events.iter().find(|event| event.id == id)
                    {
                        if let Some(choice) =
                            event.choices.get(self.cursor % event.choices.len().max(1))
                        {
                            self.apply(Command::Respond {
                                event_id: id,
                                choice_id: choice.id.clone(),
                            });
                            self.pending_event = None;
                            self.screen = Screen::Journey;
                        }
                    }
                }
            }
            Screen::Score
            | Screen::Hall
            | Screen::Settings
            | Screen::Seed
            | Screen::Supplies
            | Screen::Map
            | Screen::Treat
            | Screen::Minigame => self.back(),
        }
    }
    fn adjust(&mut self, delta: i8) {
        match self.screen {
            Screen::SetupTrail => {
                self.draft.era = cycle(self.draft.era, self.game.content.eras.len(), delta)
            }
            Screen::SetupDeparture => {
                self.draft.month = cycle(usize::from(self.draft.month - 3), 5, delta) as u8 + 3
            }
            Screen::Store if self.cursor < self.game.content.items.len() && delta > 0 => {
                let id = self.game.content.items[self.cursor].id.clone();
                self.apply(Command::Buy { item_id: id, quantity: 1 });
            }
            _ => {}
        }
    }
    fn character(&mut self, character: char) {
        match (self.screen, character) {
            (Screen::Journey, 's') => self.screen = Screen::Supplies,
            (Screen::Journey, 'm') => self.screen = Screen::Map,
            (Screen::Journey, 'p') => self.screen = Screen::Pace,
            (Screen::Journey, 'r') => self.screen = Screen::Rations,
            (Screen::Journey, 'x') => self.screen = Screen::Rest,
            (Screen::Journey, 't') => self.screen = Screen::Talk,
            (Screen::Journey, 'b') if self.game.can_shop() => self.screen = Screen::Store,
            (Screen::SetupParty, c) if self.cursor < 5 && !c.is_control() => {
                self.draft.names[self.cursor].push(c)
            }
            _ => {}
        }
    }
    fn back(&mut self) {
        self.cursor = 0;
        self.screen = match self.screen {
            Screen::Title => {
                self.quit = true;
                Screen::Title
            }
            Screen::SetupTrail | Screen::Hall | Screen::Settings | Screen::Seed => Screen::Title,
            Screen::SetupOccupation => Screen::SetupTrail,
            Screen::SetupParty => Screen::SetupOccupation,
            Screen::SetupDeparture => Screen::SetupParty,
            Screen::Store
            | Screen::Supplies
            | Screen::Map
            | Screen::Pace
            | Screen::Rations
            | Screen::Rest
            | Screen::Talk
            | Screen::Treat
            | Screen::Minigame => Screen::Journey,
            Screen::Fork | Screen::River | Screen::Event | Screen::Score => Screen::Journey,
            Screen::Journey => Screen::Title,
        }
    }
    fn apply(&mut self, command: Command) {
        for outcome in self.game.apply(command) {
            self.outcome(outcome);
        }
        self.autosave();
    }
    fn outcome(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::ForkAvailable { .. } => self.screen = Screen::Fork,
            Outcome::RiverCrossingRequired { .. } => self.screen = Screen::River,
            Outcome::Event { event_id, text } => {
                self.pending_event = Some(event_id);
                self.note(text);
                self.screen = Screen::Event;
            }
            Outcome::Score { points } => {
                self.note(format!("Final score: {points}"));
                self.screen = Screen::Score;
            }
            Outcome::Message(text) => self.note(text),
            Outcome::Rejected(error) => self.note(error.to_string()),
            other => self.note(format!("{other:?}")),
        }
    }
    fn note(&mut self, text: impl Into<String>) {
        self.log.push(text.into());
        if self.log.len() > 4 {
            self.log.remove(0);
        }
    }
    fn autosave(&mut self) {
        if let Some(storage) = &self.storage {
            if let Err(error) = storage.save_session(&self.run_id, &self.game) {
                self.note(format!("Save failed: {error}"));
            }
        }
    }
    fn continue_saved(&mut self) {
        let Some(storage) = &self.storage else {
            self.note("No local storage is attached.");
            return;
        };
        match storage.load_session() {
            Ok(Some((run_id, game))) => match game.validate() {
                Ok(()) => {
                    self.run_id = run_id;
                    self.game = game;
                    self.screen = Screen::Journey;
                }
                Err(error) => self.note(format!("Saved journey is invalid: {error}")),
            },
            Ok(None) => self.note("No saved journey found."),
            Err(error) => self.note(format!("Could not read save: {error}")),
        }
    }
    fn load_hall(&mut self) {
        self.hall.clear();
        if let Some(storage) = &self.storage {
            match storage.load_history() {
                Ok(history) => {
                    self.hall = history
                        .leaders()
                        .iter()
                        .map(|entry| format!("{}  {} points", entry.leader, entry.score))
                        .collect()
                }
                Err(error) => self.note(format!("Could not read hall: {error}")),
            }
        } else {
            self.note("No local storage is attached.");
        }
        self.screen = Screen::Hall;
    }
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let outer = Block::default().borders(Borders::ALL).title(self.title());
        frame.render_widget(outer, area);
        let inner = area.inner(Margin { horizontal: 2, vertical: 1 });
        let chunks = Layout::vertical([Constraint::Min(5), Constraint::Length(5)]).split(inner);
        let lines = self.body();
        frame.render_widget(
            Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false }),
            chunks[0],
        );
        let log = self.log.iter().rev().map(|line| ListItem::new(line.clone())).collect::<Vec<_>>();
        frame.render_widget(
            List::new(log).block(Block::default().borders(Borders::TOP).title("TRAIL LOG")),
            chunks[1],
        );
    }
    fn title(&self) -> &'static str {
        match self.screen {
            Screen::Title => "PIONEER TRAIL",
            Screen::Journey => "ON THE TRAIL",
            Screen::Store => "GENERAL STORE",
            Screen::Score => "JOURNEY COMPLETE",
            _ => "PIONEER TRAIL",
        }
    }
    fn body(&self) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from(self.heading())];
        match self.screen {
            Screen::Title => lines.extend(menu(
                &["New Journey", "Continue", "Hall of Fame", "Seed", "Settings", "Quit"],
                self.cursor,
            )),
            Screen::SetupTrail => {
                lines.push(Line::from("Choose Trail (Up/Down) · Left/Right era · Enter"));
                lines.extend(menu(
                    &self
                        .game
                        .content
                        .trails
                        .iter()
                        .map(|trail| trail.name.as_str())
                        .collect::<Vec<_>>(),
                    self.cursor,
                ));
            }
            Screen::SetupOccupation => lines.extend(menu(
                &self
                    .game
                    .content
                    .occupations
                    .iter()
                    .map(|job| job.name.as_str())
                    .collect::<Vec<_>>(),
                self.cursor,
            )),
            Screen::SetupParty => {
                lines.push(Line::from("Enter advances. Type a name; Esc goes back."));
                for (index, name) in self.draft.names.iter().enumerate() {
                    lines.push(Line::from(format!("{} {}", marker(index == self.cursor), name)));
                }
                lines.push(Line::from("► Done"));
            }
            Screen::SetupDeparture => lines.push(Line::from(format!(
                "Departure month: {}  [Left/Right]  Enter to outfit",
                self.draft.month
            ))),
            Screen::Store => {
                for (index, item) in self.game.content.items.iter().enumerate() {
                    lines.push(Line::from(format!(
                        "{} {}  ${:.2}  owned {}",
                        marker(index == self.cursor),
                        item.name,
                        item.price_cents as f64 / 100.0,
                        self.game.inventory.get(&item.id)
                    )));
                }
                lines.push(Line::from(format!(
                    "{} Depart  Cash ${:.2}  Weight {} lb",
                    marker(self.cursor >= self.game.content.items.len()),
                    self.game.cash_cents as f64 / 100.0,
                    self.game.weight()
                )));
            }
            Screen::Journey => {
                lines.push(Line::from(format!(
                    "Day {} · {} miles · Food {} lb · Cash ${:.2}",
                    self.game.day,
                    self.game.miles,
                    self.game.inventory.get("food"),
                    self.game.cash_cents as f64 / 100.0
                )));
                lines.extend(menu(
                    &[
                        "Continue", "Supplies", "Map", "Pace", "Rations", "Rest", "Hunt", "Talk",
                        "Buy",
                    ],
                    self.cursor,
                ));
            }
            Screen::Supplies => {
                for (id, quantity) in &self.game.inventory.quantities {
                    lines.push(Line::from(format!("{id}: {quantity}")));
                }
            }
            Screen::Map => lines.push(Line::from(format!(
                "Position: {} miles. Next landmark is shown on arrival.",
                self.game.miles
            ))),
            Screen::Pace => lines.extend(menu(&["Steady", "Strenuous", "Grueling"], self.cursor)),
            Screen::Rations => {
                lines.extend(menu(&["Filling", "Meager", "Bare Bones"], self.cursor))
            }
            Screen::Rest => {
                lines.extend(menu(&["Rest 1 day", "Rest 2 days", "Rest 3 days"], self.cursor))
            }
            Screen::Talk => lines.push(Line::from("Talk to people: Enter to listen.")),
            Screen::Fork => {
                lines.push(Line::from("Choose a route:"));
                if let Some(node) = self.game.current_landmark() {
                    lines.extend(menu(
                        &node.routes.iter().map(|route| route.label.as_str()).collect::<Vec<_>>(),
                        self.cursor,
                    ));
                }
            }
            Screen::River => lines
                .extend(menu(&["Ford", "Caulk wagon", "Ferry", "Wait", "Hire guide"], self.cursor)),
            Screen::Event => {
                lines.push(Line::from("A decision is required:"));
                if let Some(id) = &self.pending_event {
                    if let Some(event) =
                        self.game.content.events.iter().find(|event| &event.id == id)
                    {
                        lines.push(Line::from(event.text.clone()));
                        lines.extend(menu(
                            &event
                                .choices
                                .iter()
                                .map(|choice| choice.label.as_str())
                                .collect::<Vec<_>>(),
                            self.cursor,
                        ));
                    }
                }
            }
            Screen::Score => lines.push(Line::from(format!("Score: {}", self.game.score()))),
            Screen::Hall => {
                if self.hall.is_empty() {
                    lines.push(Line::from("No completed journeys yet."));
                } else {
                    for entry in &self.hall {
                        lines.push(Line::from(entry.clone()));
                    }
                }
            }
            Screen::Settings => {
                lines.push(Line::from("Settings are saved by the application host."))
            }
            Screen::Seed => lines.push(Line::from(format!("Seed: {}", self.game.rng.seed()))),
            Screen::Treat => {
                lines.push(Line::from("Treatment is available when a party member is ill."))
            }
            Screen::Minigame => {
                lines.push(Line::from("A real-time trail action has been requested."))
            }
        }
        lines.push(Line::from("Esc: Back · ↑↓/jk: Move · Enter/Space: Select · Ctrl-Q: Quit"));
        lines
    }
    fn heading(&self) -> String {
        self.title().to_string()
    }
}
/// Runs the keyboard application. The caller owns save/load policy and supplies
/// a fully constructed app, keeping terminal concerns out of persistence.
pub fn run(mut app: App) -> anyhow::Result<()> {
    let guard = crate::terminal::TerminalGuard::enter()?;
    let backend = ratatui::backend::CrosstermBackend::new(std::io::stdout());
    let mut terminal = ratatui::Terminal::new(backend)?;
    while !app.should_quit() {
        terminal.draw(|frame| app.render(frame))?;
        if let Event::Key(key) = event::read()? {
            app.handle_key(key);
        }
    }
    drop(terminal);
    drop(guard);
    Ok(())
}
fn cycle(current: usize, len: usize, delta: i8) -> usize {
    if len == 0 {
        0
    } else {
        (current + len + (delta.rem_euclid(len as i8) as usize)) % len
    }
}
fn marker(selected: bool) -> &'static str {
    if selected {
        "►"
    } else {
        " "
    }
}
fn menu(items: &[&str], cursor: usize) -> Vec<Line<'static>> {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            Line::from(format!(
                "{} {}. {}",
                marker(index == cursor % items.len().max(1)),
                index + 1,
                item
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn render(screen: Screen, width: u16, height: u16) -> String {
        let mut app = App::new(GameContent::starter(), 7, Settings::default());
        app.screen = screen;
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let buffer = terminal.backend().buffer();
        let mut output = String::new();
        for y in 0..height {
            for x in 0..width {
                output.push_str(buffer[(x, y)].symbol());
            }
            output.push('\n');
        }
        output
    }
    fn all_screens(width: u16, height: u16) -> String {
        [
            Screen::Title,
            Screen::SetupTrail,
            Screen::SetupOccupation,
            Screen::SetupParty,
            Screen::SetupDeparture,
            Screen::Store,
            Screen::Journey,
            Screen::Supplies,
            Screen::Map,
            Screen::Pace,
            Screen::Rations,
            Screen::Rest,
            Screen::Talk,
            Screen::Treat,
            Screen::Fork,
            Screen::River,
            Screen::Event,
            Screen::Score,
            Screen::Hall,
            Screen::Settings,
            Screen::Seed,
            Screen::Minigame,
        ]
        .into_iter()
        .map(|screen| format!("\n=== {screen:?} ===\n{}", render(screen, width, height)))
        .collect()
    }
    #[test]
    fn snapshots_at_80x24() {
        insta::assert_snapshot!("screens_80x24", all_screens(80, 24));
    }
    #[test]
    fn snapshots_at_120x40() {
        insta::assert_snapshot!("screens_120x40", all_screens(120, 40));
    }
    #[test]
    fn title_menu_starts_a_setup_flow() {
        let mut app = App::new(GameContent::starter(), 1, Settings::default());
        app.handle_key(KeyEvent::from(crossterm::event::KeyCode::Enter));
        assert_eq!(app.screen, Screen::SetupTrail);
    }
}
