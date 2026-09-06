use crate::{
    input::{decode, Input},
    persist::{new_run_id, RunRecord, Settings, Storage},
    screens::Screen,
    seed::WorldSeed,
};
use crossterm::event::{self, Event};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
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
    edited: [bool; 5],
    month: u8,
}
#[derive(Default)]
struct TradeDraft {
    npc: usize,
    offered: usize,
    wanted: usize,
    offered_quantity: u32,
    wanted_quantity: u32,
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
            edited: [false; 5],
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
    minigame: Option<crate::minigame::host::LiveMinigame>,
    store_quantity: u32,
    seed_text: String,
    recorded: bool,
    animation_tick: u64,
    epitaph: String,
    trade: TradeDraft,
    graves: Vec<crate::legacy::Grave>,
    auto_travel: bool,
    bell_pending: bool,
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
            minigame: None,
            store_quantity: 1,
            seed_text: String::new(),
            recorded: false,
            animation_tick: 0,
            epitaph: String::new(),
            trade: TradeDraft {
                offered: 1,
                wanted: 2,
                offered_quantity: 50,
                wanted_quantity: 1,
                ..TradeDraft::default()
            },
            graves: Vec::new(),
            auto_travel: false,
            bell_pending: false,
        }
    }
    pub fn with_storage(mut self, storage: Storage) -> Self {
        self.storage = Some(storage);
        self
    }
    pub fn should_quit(&self) -> bool {
        self.quit
    }
    pub fn with_defaults(mut self, config: &crate::headless::RunConfig) -> Self {
        self.draft.trail =
            self.game.content.trails.iter().position(|t| t.id == config.trail).unwrap_or(0);
        self.draft.era =
            self.game.content.eras.iter().position(|e| e.id == config.era).unwrap_or(0);
        self.draft.occupation = self
            .game
            .content
            .occupations
            .iter()
            .position(|o| o.id == config.occupation)
            .unwrap_or(0);
        self.draft.month = config.month;
        self
    }
    pub fn resume(&mut self) {
        self.continue_saved();
    }
    fn menu_len(&self) -> usize {
        match self.screen {
            Screen::Title | Screen::SetupParty => 6,
            Screen::SetupTrail => self.game.content.trails.len(),
            Screen::SetupOccupation => self.game.content.occupations.len(),
            Screen::Store => self.game.content.items.len() + 1,
            Screen::Journey => 9,
            Screen::Map => self
                .game
                .content
                .trails
                .iter()
                .find(|t| Some(&t.id) == self.game.trail_id.as_ref())
                .map_or(1, |t| t.nodes.len() + self.graves.len()),
            Screen::Pace | Screen::Rations | Screen::Rest => 3,
            Screen::Treat | Screen::Party => self.game.party.len(),
            Screen::Trade => 9,
            Screen::River => 5,
            Screen::Fork => self.game.current_landmark().map_or(0, |n| n.routes.len()),
            Screen::Event => self
                .game
                .content
                .events
                .iter()
                .find(|e| Some(&e.id) == self.game.pending_event.as_ref())
                .map_or(0, |e| e.choices.len()),
            _ => 1,
        }
    }
    /// One fixed 30 Hz step. The runtime pauses these steps below the minimum terminal size.
    pub fn tick_minigame(&mut self) {
        if let Some(game) = &mut self.minigame {
            game.tick();
        }
        self.finish_minigame();
    }
    fn finish_minigame(&mut self) {
        if let Some(result) = self.minigame.as_ref().and_then(|game| game.result()) {
            self.minigame = None;
            self.apply(result);
        }
    }
    pub fn handle_key(&mut self, key: KeyEvent) {
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return;
        }
        if key.code != KeyCode::Char('a') {
            self.auto_travel = false;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
            self.quit = true;
            return;
        }
        if let Some(game) = &mut self.minigame {
            if self.settings.no_art {
                game.text_key(key.code);
            } else {
                game.key(key.code);
            }
            self.finish_minigame();
            return;
        }
        if self.screen == Screen::Epitaph {
            match key.code {
                KeyCode::Char(c) if !c.is_control() && self.epitaph.chars().count() < 80 => {
                    self.epitaph.push(c);
                    return;
                }
                KeyCode::Backspace => {
                    self.epitaph.pop();
                    return;
                }
                _ => {}
            }
        }
        if self.screen == Screen::SetupParty && self.cursor < 5 {
            match key.code {
                KeyCode::Backspace | KeyCode::Delete => {
                    self.draft.names[self.cursor].pop();
                    return;
                }
                KeyCode::Char(c)
                    if !c.is_control() && self.draft.names[self.cursor].chars().count() < 24 =>
                {
                    if !self.draft.edited[self.cursor] {
                        self.draft.names[self.cursor].clear();
                        self.draft.edited[self.cursor] = true;
                    }
                    self.draft.names[self.cursor].push(c);
                    return;
                }
                _ => {}
            }
        }
        if self.screen == Screen::Seed {
            match key.code {
                KeyCode::Backspace => {
                    self.seed_text.pop();
                    return;
                }
                KeyCode::Char(c) if c.is_ascii_alphanumeric() || c == '-' => {
                    if self.seed_text.len() >= 18 {
                        return;
                    }
                    self.seed_text.push(c);
                    return;
                }
                _ => {}
            }
        }
        let previous_cursor = self.cursor;
        match decode(key) {
            Input::Quit => self.quit = true,
            Input::Up => self.cursor = self.cursor.saturating_sub(1),
            Input::Down => self.cursor = (self.cursor + 1).min(self.menu_len().saturating_sub(1)),
            Input::Digit(n) if n > 0 => {
                if usize::from(n) <= self.menu_len() {
                    if self.screen == Screen::Store && self.cursor != usize::from(n - 1) {
                        self.store_quantity = 1;
                    }
                    self.cursor = usize::from(n - 1);
                    self.select();
                }
            }
            Input::Select => self.select(),
            Input::Back => self.back(),
            Input::Left => self.adjust(-1),
            Input::Right => self.adjust(1),
            Input::Character(c) => self.character(c),
            Input::None | Input::Digit(_) => {}
        }
        if self.screen == Screen::Store && self.cursor != previous_cursor {
            self.store_quantity = 1;
        }
    }
    fn select(&mut self) {
        match self.screen {
            Screen::Title => match self.cursor % 6 {
                0 => {
                    self.game =
                        GameState::with_content(self.game.rng.seed(), self.game.content.clone());
                    self.run_id = new_run_id();
                    self.recorded = false;
                    self.pending_event = None;
                    self.minigame = None;
                    self.log.clear();
                    self.epitaph.clear();
                    self.screen = Screen::SetupTrail;
                    self.cursor = self.draft.trail;
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
                self.cursor = self.draft.occupation;
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
                self.cursor = 0;
            }
            Screen::Store => {
                if self.cursor < self.game.content.items.len() {
                    let id = self.game.content.items[self.cursor].id.clone();
                    self.apply(Command::Buy { item_id: id, quantity: self.store_quantity });
                } else {
                    if matches!(self.game.status, pioneer_sim::RunStatus::Outfitting) {
                        self.apply(Command::Depart);
                    } else {
                        self.apply(Command::Continue);
                    }
                }
            }
            Screen::Journey => match self.cursor % 9 {
                0 => self.apply(Command::Continue),
                1 => self.screen = Screen::Supplies,
                2 => self.screen = Screen::Map,
                3 => self.screen = Screen::Pace,
                4 => self.screen = Screen::Rations,
                5 => self.screen = Screen::Rest,
                6 => self.apply(Command::BeginHunt),
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
            }
            Screen::Rations => {
                let rations = [RationLevel::Filling, RationLevel::Meager, RationLevel::BareBones]
                    [self.cursor % 3];
                self.apply(Command::SetRations(rations));
            }
            Screen::Rest => {
                self.apply(Command::Rest { days: (self.cursor + 1) as u32 });
            }
            Screen::Talk => {
                self.apply(Command::Talk);
            }
            Screen::Treat => {
                if let Some(ailment_id) = self
                    .game
                    .party
                    .get(self.cursor)
                    .filter(|m| m.alive)
                    .and_then(|m| m.ailments.first())
                    .cloned()
                {
                    self.apply(Command::Treat { member_index: self.cursor, ailment_id });
                } else {
                    self.note("This traveler has no ailment to treat.");
                }
            }
            Screen::Trade => {
                let Some(npc) = self.game.npcs.get(self.trade.npc) else {
                    return;
                };
                let npc_id = npc.id.clone();
                let offered = &self.game.content.items[self.trade.offered];
                let wanted = &self.game.content.items[self.trade.wanted];
                match self.cursor {
                    5 => self.apply(Command::Barter {
                        npc_id,
                        offered_item: offered.id.clone(),
                        offered_quantity: self.trade.offered_quantity,
                        wanted_item: wanted.id.clone(),
                        wanted_quantity: self.trade.wanted_quantity,
                    }),
                    6 => {
                        if let Some(offer) = self.game.pending_counteroffer.clone() {
                            self.apply(Command::AcceptCounteroffer {
                                npc_id: offer.npc_id,
                                offered_item: offer.offered_item,
                                offered_quantity: offer.offered_quantity,
                                wanted_item: offer.wanted_item,
                                wanted_quantity: offer.wanted_quantity,
                            });
                        } else {
                            self.note("There is no open counteroffer.");
                        }
                    }
                    7 => self.apply(Command::InviteNpc { npc_id }),
                    8 => self.apply(Command::DismissNpc { npc_id }),
                    _ => self.adjust(1),
                }
            }
            Screen::Fork => {
                if let Some(node) = self.game.current_landmark() {
                    if let Some(route) = node.routes.get(self.cursor % node.routes.len().max(1)) {
                        self.apply(Command::ChooseRoute { route_id: route.id.clone() });
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
                        }
                    }
                }
            }
            Screen::Seed => match WorldSeed::parse(&self.seed_text) {
                Ok(world) => {
                    self.game = GameState::with_content(world.seed, self.game.content.clone());
                    self.run_id = new_run_id();
                    self.recorded = false;
                    self.pending_event = None;
                    if let Some(index) =
                        self.game.content.trails.iter().position(|trail| trail.id == world.trail)
                    {
                        self.draft.trail = index;
                    }
                    if let Some(index) =
                        self.game.content.eras.iter().position(|era| era.id == world.era)
                    {
                        self.draft.era = index;
                    }
                    self.draft = SetupDraft {
                        trail: self.draft.trail,
                        era: self.draft.era,
                        ..SetupDraft::default()
                    };
                    self.screen = Screen::SetupTrail;
                    self.cursor = self.draft.trail;
                }
                Err(error) => self.note(error.to_string()),
            },
            Screen::Epitaph => {
                if let Some(storage) = &self.storage {
                    if let Err(error) = storage.set_epitaph(&self.run_id, &self.epitaph) {
                        self.note(format!("Could not save epitaph: {error}"));
                        return;
                    }
                }
                self.screen = Screen::Score;
            }
            Screen::Score
            | Screen::Party
            | Screen::Hall
            | Screen::Settings
            | Screen::Supplies
            | Screen::Map
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
            Screen::Store if self.cursor < self.game.content.items.len() => {
                let step = if self.game.content.items[self.cursor].id == "food" { 100 } else { 1 };
                self.store_quantity = if delta > 0 {
                    if step == 100 && self.store_quantity == 1 {
                        100
                    } else {
                        self.store_quantity.saturating_add(step)
                    }
                } else {
                    self.store_quantity.saturating_sub(step).max(1)
                };
            }
            Screen::Trade => match self.cursor {
                0 => self.trade.npc = cycle(self.trade.npc, self.game.npcs.len(), delta),
                1 => {
                    self.trade.offered =
                        cycle(self.trade.offered, self.game.content.items.len(), delta)
                }
                3 => {
                    self.trade.wanted =
                        cycle(self.trade.wanted, self.game.content.items.len(), delta)
                }
                2 | 4 => {
                    let quantity = if self.cursor == 2 {
                        &mut self.trade.offered_quantity
                    } else {
                        &mut self.trade.wanted_quantity
                    };
                    *quantity = quantity.saturating_add_signed(i32::from(delta)).clamp(1, 2000);
                }
                _ => {}
            },
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
            (Screen::Journey, 'i') => self.screen = Screen::Treat,
            (Screen::Journey, 'u') => {
                self.screen = Screen::Trade;
                self.cursor = 0;
            }
            (Screen::Journey, 'v') => {
                self.screen = Screen::Party;
                self.cursor = 0;
            }
            (Screen::Journey, 'f') => self.apply(Command::Forage),
            (Screen::Journey, 'g') => self.apply(Command::Fish),
            (Screen::Journey, 'a') => {
                if matches!(self.game.status, pioneer_sim::RunStatus::AtLandmark(_)) {
                    self.apply(Command::Continue);
                }
                self.auto_travel = !self.auto_travel;
            }
            (Screen::SetupTrail, 'd') => {
                use pioneer_sim::state::Difficulty;
                let next = match self.game.difficulty {
                    Difficulty::Easy => Difficulty::Normal,
                    Difficulty::Normal => Difficulty::Hard,
                    Difficulty::Hard => Difficulty::Easy,
                };
                self.apply(Command::SetDifficulty(next));
            }
            (Screen::Store, 's') if self.cursor < self.game.content.items.len() => {
                self.apply(Command::Sell {
                    item_id: self.game.content.items[self.cursor].id.clone(),
                    quantity: self.store_quantity,
                });
            }
            (Screen::Journey, 'b') if self.game.can_shop() => self.screen = Screen::Store,
            (Screen::Score, 'e') => self.screen = Screen::Epitaph,
            (Screen::Settings, 's') => {
                use crate::persist::Speed;
                self.settings.speed = match self.settings.speed {
                    Speed::Instant => Speed::Fast,
                    Speed::Fast => Speed::Normal,
                    Speed::Normal => Speed::Slow,
                    Speed::Slow => Speed::Instant,
                };
                self.save_settings();
            }
            (Screen::Settings, 'c') => {
                self.settings.color = match self.settings.color {
                    crate::persist::ColorMode::Truecolor => crate::persist::ColorMode::Indexed,
                    crate::persist::ColorMode::Indexed => crate::persist::ColorMode::Basic,
                    crate::persist::ColorMode::Basic => crate::persist::ColorMode::Mono,
                    crate::persist::ColorMode::Mono => crate::persist::ColorMode::Truecolor,
                };
                self.save_settings();
            }
            (Screen::Settings, 'b') => {
                self.settings.bell = !self.settings.bell;
                self.save_settings();
            }
            (Screen::Settings, 'a') => {
                self.settings.no_art = !self.settings.no_art;
                self.save_settings();
            }
            _ => {}
        }
    }
    fn back(&mut self) {
        if matches!(self.screen, Screen::Fork | Screen::River | Screen::Event) {
            self.note("A decision is required before continuing.");
            return;
        }
        if self.screen == Screen::Store && self.game.status == pioneer_sim::RunStatus::Outfitting {
            self.note("Buy oxen and supplies, then choose Depart.");
            return;
        }
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
            | Screen::Trade
            | Screen::Party
            | Screen::Minigame => Screen::Journey,
            Screen::Score => Screen::Title,
            Screen::Epitaph => Screen::Score,
            Screen::Fork | Screen::River | Screen::Event => Screen::Journey,
            Screen::Journey => Screen::Title,
        }
    }
    fn apply(&mut self, command: Command) {
        let previous_screen = self.screen;
        let previous_miles = self.game.miles;
        let configuring = matches!(&command, Command::Configure { .. });
        let retain_screen = matches!(
            &command,
            Command::Buy { .. }
                | Command::Sell { .. }
                | Command::Barter { .. }
                | Command::AcceptCounteroffer { .. }
                | Command::InviteNpc { .. }
                | Command::DismissNpc { .. }
                | Command::Talk
                | Command::Treat { .. }
        );
        let outcomes = self.game.apply(command);
        let rejected = !outcomes.is_empty()
            && outcomes.iter().all(|outcome| matches!(outcome, Outcome::Rejected(_)));
        for outcome in outcomes {
            self.outcome(outcome);
        }
        if configuring && !rejected {
            self.load_graves();
        }
        let passed = self
            .graves
            .iter()
            .filter(|grave| grave.mile > previous_miles && grave.mile <= self.game.miles)
            .map(|grave| {
                format!("Grave at mile {}: {} — {}", grave.mile, grave.leader, grave.epitaph)
            })
            .collect::<Vec<_>>();
        for note in passed {
            self.note(note);
        }
        self.autosave();
        self.sync_screen();
        if rejected
            || (retain_screen
                && self.game.pending_event.is_none()
                && matches!(
                    self.game.status,
                    pioneer_sim::RunStatus::Travelling
                        | pioneer_sim::RunStatus::AtLandmark(_)
                        | pioneer_sim::RunStatus::Outfitting
                ))
        {
            self.screen = previous_screen;
        }
        if self.screen != previous_screen {
            self.cursor = 0;
        }
        if self.screen != Screen::Journey || self.game.status != pioneer_sim::RunStatus::Travelling
        {
            self.auto_travel = false;
        }
    }
    fn sync_screen(&mut self) {
        if self.game.active_minigame.is_some() {
            if self.minigame.is_none() {
                self.minigame = crate::minigame::host::LiveMinigame::from_session(&self.game);
            }
            self.screen = Screen::Minigame;
            return;
        }
        self.minigame = None;
        if self.game.pending_event.is_some() {
            self.pending_event = self.game.pending_event.clone();
            self.screen = Screen::Event;
            return;
        }
        self.pending_event = None;
        self.screen = match self.game.status {
            pioneer_sim::RunStatus::Setup => Screen::SetupTrail,
            pioneer_sim::RunStatus::Outfitting => Screen::Store,
            pioneer_sim::RunStatus::Travelling | pioneer_sim::RunStatus::AtLandmark(_) => {
                Screen::Journey
            }
            pioneer_sim::RunStatus::AwaitingFork(_) => Screen::Fork,
            pioneer_sim::RunStatus::AwaitingRiver(_) => Screen::River,
            pioneer_sim::RunStatus::Arrived | pioneer_sim::RunStatus::Failed => Screen::Score,
        };
        if matches!(
            self.game.status,
            pioneer_sim::RunStatus::Arrived | pioneer_sim::RunStatus::Failed
        ) {
            self.record_run();
        }
    }
    fn outcome(&mut self, outcome: Outcome) {
        if matches!(
            &outcome,
            Outcome::Event { .. } | Outcome::MemberDied { .. } | Outcome::Score { .. }
        ) {
            self.bell_pending = true;
        }
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
            Outcome::Quote { text, .. } => self.note(text),
            Outcome::ArrivedAt { .. } => {
                if let Some(node) = self.game.current_landmark() {
                    self.note(format!("Reached {}.", node.name));
                }
            }
            Outcome::MemberDied { name } => self.note(format!("{name} has died.")),
            _ => {}
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
    fn save_settings(&mut self) {
        if let Some(storage) = &self.storage {
            if let Err(error) = storage.save_settings(&self.settings) {
                self.note(format!("Settings save failed: {error}"));
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
                    self.recorded = false;
                    self.minigame = None;
                    self.load_graves();
                    self.sync_screen();
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
    fn load_graves(&mut self) {
        let history = match self.storage.as_ref().map(Storage::load_history).transpose() {
            Ok(history) => history.unwrap_or_default(),
            Err(error) => {
                self.note(format!("Could not read local graves: {error}"));
                crate::persist::History::default()
            }
        };
        self.graves = self
            .game
            .content
            .trails
            .iter()
            .find(|trail| Some(&trail.id) == self.game.trail_id.as_ref())
            .map(|trail| {
                crate::legacy::graves(
                    &history,
                    self.game.rng.seed(),
                    &trail.id,
                    self.game.date().0 as u16,
                    crate::legacy::trail_extent(trail),
                )
            })
            .unwrap_or_default();
    }
    fn record_run(&mut self) {
        if self.recorded {
            return;
        }
        if let Some(storage) = &self.storage {
            let arrived = matches!(self.game.status, pioneer_sim::RunStatus::Arrived);
            let record = RunRecord {
                run_id: self.run_id.clone(),
                leader: self
                    .game
                    .party
                    .first()
                    .map_or_else(|| "Unknown".into(), |member| member.name.clone()),
                seed: self.game.rng.seed(),
                trail: self.game.trail_id.clone().unwrap_or_default(),
                era: self
                    .game
                    .era_id
                    .as_deref()
                    .and_then(|era| era.parse().ok())
                    .unwrap_or_default(),
                occupation: self.game.occupation_id.clone().unwrap_or_default(),
                score: self.game.score(),
                survivors: self.game.party.iter().filter(|member| member.alive).count(),
                days: self.game.day,
                miles: self.game.miles,
                arrived,
                epitaph: String::new(),
                cause: if arrived { "Arrived".into() } else { "Trail ended".into() },
            };
            match storage.record_run(record) {
                Ok(_) => self.recorded = true,
                Err(error) => self.note(format!("Hall save failed: {error}")),
            }
        }
    }
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        if area.width < 80 || area.height < 24 {
            frame.render_widget(
                Paragraph::new(format!(
                    "PIONEER TRAIL requires 80×24. Current size: {}×{}.",
                    area.width, area.height
                ))
                .block(Block::default().borders(Borders::ALL).title("RESIZE TERMINAL")),
                area,
            );
            return;
        }
        if !self.settings.no_art && matches!(self.screen, Screen::Title | Screen::Journey) {
            self.render_scene(frame);
            return;
        }
        if let Some(game) = &self.minigame {
            let canvas =
                Rect::new(area.x + (area.width - 80) / 2, area.y + (area.height - 24) / 2, 80, 24);
            if self.settings.no_art {
                frame.render_widget(
                    Paragraph::new(game.text_lines().join("\n\n"))
                        .wrap(ratatui::widgets::Wrap { trim: true }),
                    canvas.inner(Margin { horizontal: 1, vertical: 1 }),
                );
            } else {
                game.render(frame.buffer_mut(), canvas, self.color_mode());
            }
            return;
        }
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
    fn color_mode(&self) -> crate::art::ColorMode {
        match self.settings.color {
            crate::persist::ColorMode::Truecolor => crate::art::ColorMode::TrueColor,
            crate::persist::ColorMode::Indexed => crate::art::ColorMode::Ansi256,
            crate::persist::ColorMode::Basic => crate::art::ColorMode::Ansi16,
            crate::persist::ColorMode::Mono => crate::art::ColorMode::Mono,
        }
    }
    fn render_scene(&self, frame: &mut Frame) {
        use crate::art;
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black).fg(Color::White)),
            area,
        );
        let canvas = Rect::new(area.x + (area.width - 80) / 2, area.y, 80, 16);
        let mode = self.color_mode();
        if self.screen == Screen::Title {
            art::render(
                art::embedded("title.px").expect("title art"),
                frame.buffer_mut(),
                canvas,
                mode,
            );
            let labels = ["New Journey", "Continue", "Hall of Fame", "Seed", "Settings", "Quit"];
            for (i, label) in labels.iter().enumerate() {
                frame.render_widget(
                    Paragraph::new(format!("{} {}. {label}", marker(i == self.cursor), i + 1)),
                    Rect::new(
                        canvas.x + 10 + (i % 2) as u16 * 35,
                        canvas.y + 17 + (i / 2) as u16,
                        34,
                        1,
                    ),
                );
            }
            frame.render_widget(
                Paragraph::new("↑↓ choose · Enter begin · 1–6 select · Ctrl-Q quit"),
                Rect::new(canvas.x + 8, canvas.y + 21, 72, 1),
            );
            if let Some(last) = self.log.last() {
                frame.render_widget(
                    Paragraph::new(last.clone()).wrap(ratatui::widgets::Wrap { trim: true }),
                    Rect::new(canvas.x + 2, canvas.y + 22, 76, 2),
                );
            }
            return;
        }
        let node = self.game.current_landmark();
        let name = node.map_or("Open trail", |n| n.name.as_str());
        let landmark = node.map(|n| format!("{}.px", n.id));
        let background = if matches!(self.game.status, pioneer_sim::RunStatus::AtLandmark(_)) {
            landmark.as_deref().and_then(art::embedded)
        } else {
            None
        };
        let background = background.unwrap_or_else(|| {
            let id = self.game.target_node_id.as_deref().unwrap_or("");
            let file = if id.contains("pass") || id.contains("sierra") || id.contains("mountain") {
                "terrain_mountains.px"
            } else if id.contains("river") || id == "humboldt" {
                "terrain_river_valley.px"
            } else if id.contains("desert") {
                "terrain_desert.px"
            } else if self.game.miles > 1500 {
                "terrain_forest.px"
            } else {
                "terrain_plains.px"
            };
            art::embedded(file).expect("terrain art")
        });
        art::render(background, frame.buffer_mut(), canvas, mode);
        let moving = matches!(self.game.status, pioneer_sim::RunStatus::Travelling);
        let phase = if moving { self.animation_tick % 4 } else { 0 };
        for (file, x, y) in
            [(format!("ox_{}.px", phase % 2), 10, 10), (format!("wagon_{phase}.px"), 32, 5)]
        {
            art::render(
                art::embedded(&file).expect("wagon animation"),
                frame.buffer_mut(),
                Rect::new(canvas.x + x, canvas.y + y, 80 - x, 16 - y),
                mode,
            );
        }
        let (year, month, day) = self.game.date();
        frame.render_widget(
            Paragraph::new(format!(
                "{month:02}/{day:02}/{year} · {} mi · {name} · {:?}",
                self.game.miles, self.game.weather
            )),
            Rect::new(canvas.x, canvas.y + 16, 80, 1),
        );
        frame.render_widget(
            Paragraph::new(format!(
                "Food {} lb · ${:.2} · {:?}/{:?} · {} alive",
                self.game.inventory.get("food"),
                self.game.cash_cents as f64 / 100.,
                self.game.pace,
                self.game.rations,
                self.game.party.iter().filter(|m| m.alive).count()
            )),
            Rect::new(canvas.x, canvas.y + 17, 80, 1),
        );
        for (i, label) in
            ["Continue", "Supplies", "Map", "Pace", "Rations", "Rest", "Hunt", "Talk", "Buy"]
                .iter()
                .enumerate()
        {
            frame.render_widget(
                Paragraph::new(format!("{} {} {label}", marker(i == self.cursor), i + 1)),
                Rect::new(canvas.x + (i % 3) as u16 * 26, canvas.y + 18 + (i / 3) as u16, 26, 1),
            );
        }
        frame.render_widget(
            Paragraph::new(
                "a travel · i treat · u trade · f forage · g fish · v party · Esc title",
            ),
            Rect::new(canvas.x, canvas.y + 21, 80, 1),
        );
        if let Some(last) = self.log.last() {
            frame.render_widget(
                Paragraph::new(last.clone()).wrap(ratatui::widgets::Wrap { trim: true }),
                Rect::new(canvas.x, canvas.y + 22, 80, 2),
            );
        }
        if area.height >= 30 {
            let party = self
                .game
                .party
                .iter()
                .map(|m| {
                    Line::from(format!(
                        "{} · Health {} · Morale {} · {}",
                        m.name,
                        m.health,
                        m.morale,
                        if m.alive { m.ailments.join(", ") } else { "deceased".into() }
                    ))
                })
                .collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(party)
                    .block(Block::default().borders(Borders::TOP).title("YOUR PARTY")),
                Rect::new(canvas.x, canvas.y + 25, 80, area.height - 25),
            );
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
                if let Some(era) = self.game.content.eras.get(self.draft.era) {
                    lines.push(Line::from(format!("Era: {}", era.name)));
                }
                lines.push(Line::from(format!("Difficulty: {:?} [D]", self.game.difficulty)));
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
            Screen::SetupOccupation => {
                lines.extend(menu(
                    &self
                        .game
                        .content
                        .occupations
                        .iter()
                        .map(|job| job.name.as_str())
                        .collect::<Vec<_>>(),
                    self.cursor,
                ));
                if let Some(job) = self.game.content.occupations.get(self.cursor) {
                    lines.push(Line::from(format!(
                        "${:.0} · score ×{} · {}",
                        job.starting_cash_cents as f64 / 100.,
                        job.score_multiplier,
                        job.perk
                    )));
                }
            }
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
                        self.game.price_cents(&item.id).unwrap_or(item.price_cents) as f64 / 100.0,
                        self.game.inventory.get(&item.id)
                    )));
                }
                lines.push(Line::from(format!(
                    "{} {}  Qty {} [←/→]  Cash ${:.2}  Weight {} lb",
                    marker(self.cursor >= self.game.content.items.len()),
                    if matches!(self.game.status, pioneer_sim::RunStatus::Outfitting) {
                        "Depart"
                    } else {
                        "Leave"
                    },
                    self.store_quantity,
                    self.game.cash_cents as f64 / 100.0,
                    self.game.weight()
                )));
                lines.push(Line::from(
                    "Enter buys · S sells selected quantity at half the quoted price",
                ));
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
                lines.push(Line::from(
                    "A auto travel · I treat · U trade · F forage · G fish · V party",
                ));
            }
            Screen::Supplies => {
                for (id, quantity) in &self.game.inventory.quantities {
                    lines.push(Line::from(format!("{id}: {quantity}")));
                }
            }
            Screen::Map => {
                lines.push(Line::from(format!(
                    "{} miles · {:?} · next {} ({} mi)",
                    self.game.miles,
                    self.game.weather,
                    self.game.target_node_id.as_deref().unwrap_or("camp"),
                    self.game.route_miles_remaining
                )));
                if let Some(trail) = self
                    .game
                    .content
                    .trails
                    .iter()
                    .find(|trail| Some(&trail.id) == self.game.trail_id.as_ref())
                {
                    let mut entries = trail
                        .nodes
                        .iter()
                        .map(|node| {
                            format!(
                                "{} {}: {} miles",
                                if Some(&node.id) == self.game.current_node_id.as_ref() {
                                    "►"
                                } else {
                                    " "
                                },
                                node.name,
                                node.mile
                            )
                        })
                        .collect::<Vec<_>>();
                    entries.extend(self.graves.iter().map(|grave| {
                        format!(
                            "† Mile {} · {} · {}{}",
                            grave.mile,
                            grave.leader,
                            grave.date,
                            if grave.local { " [local]" } else { "" }
                        )
                    }));
                    lines.extend(entries.into_iter().skip(self.cursor).take(11).map(Line::from));
                    if let Some(grave) = self
                        .cursor
                        .checked_sub(trail.nodes.len())
                        .and_then(|index| self.graves.get(index))
                    {
                        lines.push(Line::from(format!("{}: {}", grave.cause, grave.epitaph)));
                    }
                    lines.push(Line::from("↑↓ scroll landmarks and graves"));
                }
            }
            Screen::Pace => lines.extend(menu(&["Steady", "Strenuous", "Grueling"], self.cursor)),
            Screen::Rations => {
                lines.extend(menu(&["Filling", "Meager", "Bare Bones"], self.cursor))
            }
            Screen::Rest => {
                lines.extend(menu(&["Rest 1 day", "Rest 2 days", "Rest 3 days"], self.cursor))
            }
            Screen::Talk => {
                lines.push(Line::from("Talk to people: Enter to listen. Esc returns to camp."));
                if let Some(last) = self.log.last() {
                    lines.push(Line::from(last.clone()));
                }
            }
            Screen::Trade => {
                lines.push(Line::from("↑↓ select · ←→ change · Enter act · Esc camp"));
                let npc = self.game.npcs.get(self.trade.npc);
                let offered = &self.game.content.items[self.trade.offered];
                let wanted = &self.game.content.items[self.trade.wanted];
                let rows = [
                    format!(
                        "Neighbor: {} (goodwill {})",
                        npc.map_or("No neighbors", |n| n.name.as_str()),
                        npc.map_or(0, |n| n.reputation)
                    ),
                    format!(
                        "Offer: {} (own {})",
                        offered.name,
                        self.game.inventory.get(&offered.id)
                    ),
                    format!("Offer quantity: {}", self.trade.offered_quantity),
                    format!(
                        "Request: {} (neighbor has {})",
                        wanted.name,
                        npc.and_then(|n| n.inventory.get(&wanted.id)).copied().unwrap_or(0)
                    ),
                    format!("Request quantity: {}", self.trade.wanted_quantity),
                    "Propose barter".into(),
                    "Accept pending counteroffer".into(),
                    "Invite neighbor to party".into(),
                    "Dismiss joined neighbor".into(),
                ];
                for (index, row) in rows.iter().enumerate() {
                    lines.push(Line::from(format!("{} {row}", marker(index == self.cursor))));
                }
                if let Some(offer) = &self.game.pending_counteroffer {
                    lines.push(Line::from(format!(
                        "Counteroffer: {} {} for {} {}",
                        offer.offered_quantity,
                        offer.offered_item,
                        offer.wanted_quantity,
                        offer.wanted_item
                    )));
                }
            }
            Screen::Party => {
                for (index, member) in
                    self.game.party.iter().enumerate().skip(self.cursor.saturating_sub(3)).take(7)
                {
                    lines.push(Line::from(format!(
                        "{} {} · age {} · health {} · morale {}{}",
                        marker(index == self.cursor),
                        member.name,
                        member.age,
                        member.health,
                        member.morale,
                        if member.alive { "" } else { " · deceased" }
                    )));
                }
                if let Some(member) = self.game.party.get(self.cursor) {
                    lines.push(Line::from(format!("Traits: {:?}", member.traits)));
                    lines.push(Line::from(format!(
                        "Skills: hunt {} · medicine {} · repair {} · animals {}",
                        member.skills.hunting,
                        member.skills.medicine,
                        member.skills.repair,
                        member.skills.animals
                    )));
                    lines.push(Line::from(format!(
                        "Ailments: {}",
                        if member.ailments.is_empty() {
                            "none".into()
                        } else {
                            member.ailments.join(", ")
                        }
                    )));
                    lines.push(Line::from(format!(
                        "Bonds: {}",
                        member
                            .relationships
                            .affinity
                            .iter()
                            .map(|(name, value)| format!("{name} {value:+}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )));
                }
            }
            Screen::Fork => {
                lines.push(Line::from("Choose a route:"));
                if let Some(node) = self.game.current_landmark() {
                    lines.extend(menu(
                        &node.routes.iter().map(|route| route.label.as_str()).collect::<Vec<_>>(),
                        self.cursor,
                    ));
                }
            }
            Screen::River => {
                if let Some(node) = self.game.current_landmark() {
                    if let Some(river) = &node.river {
                        lines.push(Line::from(format!(
                            "{} · width {} ft · depth {} ft · ferry {}",
                            node.name,
                            river.width_feet,
                            river.depth_feet,
                            river.ferry_cost_cents.map_or("unavailable".into(), |v| format!(
                                "${:.2}",
                                v as f64 / 100.
                            ))
                        )));
                    }
                }
                lines.extend(menu(
                    &[
                        "Ford (risk cargo and lives)",
                        "Caulk wagon and float",
                        "Ferry (if available)",
                        "Wait one day",
                        "Hire guide (Snake River: 3 clothing sets)",
                    ],
                    self.cursor,
                ));
            }
            Screen::Event => {
                lines.push(Line::from("A decision is required:"));
                if let Some(id) = &self.pending_event {
                    if let Some(event) =
                        self.game.content.events.iter().find(|event| &event.id == id)
                    {
                        lines.push(Line::from(event.text.clone()));
                        for (i, choice) in event.choices.iter().enumerate() {
                            lines.push(Line::from(format!(
                                "{} {}. {}{}",
                                marker(i == self.cursor),
                                i + 1,
                                choice.label,
                                if self.game.choice_available(choice) {
                                    ""
                                } else {
                                    " [unavailable]"
                                }
                            )));
                        }
                    }
                }
            }
            Screen::Score => {
                lines.push(Line::from(format!(
                    "{} · Score {} · {} days · {} miles",
                    if matches!(self.game.status, pioneer_sim::RunStatus::Arrived) {
                        "Arrived!"
                    } else {
                        "The journey has ended."
                    },
                    self.game.score(),
                    self.game.day,
                    self.game.miles
                )));
                for member in &self.game.party {
                    lines.push(Line::from(format!(
                        "{}: {}",
                        member.name,
                        if member.alive {
                            format!("survived, health {}", member.health)
                        } else {
                            "died on the trail".into()
                        }
                    )));
                }
                if let (Some(trail), Some(era)) = (&self.game.trail_id, &self.game.era_id) {
                    if let Ok(code) = (WorldSeed {
                        seed: self.game.rng.seed(),
                        trail: trail.clone(),
                        era: era.clone(),
                    })
                    .code()
                    {
                        lines.push(Line::from(format!("Share this world: {code}")));
                    }
                }
                lines.push(Line::from("[E] Write an epitaph · Enter returns to title."));
            }
            Screen::Epitaph => {
                lines.push(Line::from("Write up to 80 characters. Enter saves; Esc cancels."));
                lines.push(Line::from(format!("{}▏", self.epitaph)));
            }
            Screen::Hall => {
                if self.hall.is_empty() {
                    lines.push(Line::from("No completed journeys yet."));
                } else {
                    for entry in &self.hall {
                        lines.push(Line::from(entry.clone()));
                    }
                }
            }
            Screen::Settings => lines.push(Line::from(format!(
                "[C]olor {:?}  [B]ell {}  [A]rt text-only {}  [S]peed {:?}",
                self.settings.color, self.settings.bell, self.settings.no_art, self.settings.speed
            ))),
            Screen::Seed => lines.push(Line::from(format!("Enter world code: {}", self.seed_text))),
            Screen::Treat => {
                for (index, member) in self.game.party.iter().enumerate() {
                    lines.push(Line::from(format!(
                        "{} {}: health {} morale {} · {}",
                        marker(index == self.cursor),
                        member.name,
                        member.health,
                        member.morale,
                        if member.alive { member.ailments.join(", ") } else { "deceased".into() }
                    )));
                }
                lines.push(Line::from("Enter treats the selected traveler's first ailment."));
            }
            Screen::Minigame => lines.push(Line::from("No active hunt or river passage.")),
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
    let started = std::time::Instant::now();
    let step = std::time::Duration::from_nanos(1_000_000_000 / 30);
    let mut next_tick = std::time::Instant::now() + step;
    let mut next_day = std::time::Instant::now();
    while !app.should_quit() {
        if app.bell_pending {
            if app.settings.bell {
                use std::io::Write;
                std::io::stdout().write_all(b"\x07")?;
                std::io::stdout().flush()?;
            }
            app.bell_pending = false;
        }
        terminal.draw(|frame| app.render(frame))?;
        let timeout = if app.minigame.is_some() && !app.settings.no_art {
            next_tick.saturating_duration_since(std::time::Instant::now())
        } else {
            std::time::Duration::from_millis(100)
        };
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                let size = terminal.size()?;
                if (size.width >= 80 && size.height >= 24)
                    || key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    app.handle_key(key);
                }
            }
        }
        let now = std::time::Instant::now();
        let size = terminal.size()?;
        if app.auto_travel && now >= next_day && size.width >= 80 && size.height >= 24 {
            app.apply(Command::Continue);
            next_day =
                now + std::time::Duration::from_millis(app.settings.speed.milliseconds().max(30));
        }
        if app.minigame.is_some() && !app.settings.no_art && size.width >= 80 && size.height >= 24 {
            // Bound catch-up after a stalled terminal; never fast-forward a whole hunt.
            for _ in 0..3 {
                if now < next_tick {
                    break;
                }
                app.tick_minigame();
                next_tick += step;
            }
            if next_tick <= now {
                next_tick = now + step;
            }
        } else {
            next_tick = now + step;
        }
        app.animation_tick = (started.elapsed().as_millis() / 250) as u64;
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
        let mut app = App::new(pioneer_data::load().unwrap(), 7, Settings::default());
        if !matches!(
            screen,
            Screen::Title
                | Screen::SetupTrail
                | Screen::SetupOccupation
                | Screen::SetupParty
                | Screen::SetupDeparture
                | Screen::Settings
                | Screen::Seed
        ) {
            app.game.apply(Command::Configure {
                trail_id: "oregon".into(),
                era_id: "1848".into(),
                occupation_id: "banker".into(),
                party: ["Ada", "James", "Ruth", "Thomas", "Clara"].map(str::to_owned).to_vec(),
                departure_month: 3,
            });
            for (item, quantity) in
                [("oxen", 3), ("food", 1500), ("clothing", 5), ("ammunition", 10), ("medicine", 2)]
            {
                app.game.apply(Command::Buy { item_id: item.into(), quantity });
            }
            if screen != Screen::Store {
                app.game.apply(Command::Depart);
            }
            app.log = vec!["The wagon is ready. Five travelers set out from Independence.".into()];
            match screen {
                Screen::River => {
                    app.game.current_node_id = Some("kansas_river".into());
                    app.game.target_node_id = None;
                    app.game.route_miles_remaining = 0;
                    app.game.miles = 102;
                    app.game.day = 9;
                    app.game.status = pioneer_sim::RunStatus::AwaitingRiver("kansas_river".into());
                }
                Screen::Fork => {
                    app.game.current_node_id = Some("south_pass".into());
                    app.game.status = pioneer_sim::RunStatus::AwaitingFork("south_pass".into());
                }
                Screen::Event => {
                    app.pending_event = Some("wheel".into());
                    app.game.pending_event = app.pending_event.clone();
                }
                Screen::Score | Screen::Epitaph => {
                    app.game.status = pioneer_sim::RunStatus::Arrived;
                    app.game.miles = 1885;
                    app.game.day = 155;
                }
                Screen::Hall => {
                    app.hall = vec!["Ada  3200 points".into(), "Clara  2800 points".into()];
                }
                Screen::Treat => {
                    app.game.party[1].ailments = vec!["fever".into()];
                    app.game.party[1].health = 68;
                }
                Screen::Minigame => {
                    app.game.apply(Command::Depart);
                    app.apply(Command::BeginHunt);
                }
                _ => {}
            }
        }
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
            Screen::Trade,
            Screen::Party,
            Screen::Fork,
            Screen::River,
            Screen::Event,
            Screen::Score,
            Screen::Epitaph,
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
    #[test]
    fn rejected_event_choice_remains_visible_and_resolvable() {
        let mut app = App::new(pioneer_data::load().unwrap(), 7, Settings::default());
        app.game.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "banker".into(),
            party: vec!["A".into(); 5],
            departure_month: 3,
        });
        app.game.pending_event = Some("wheel".into());
        app.sync_screen();
        app.select(); // No spare wheel: rejected.
        assert_eq!(app.pending_event.as_deref(), Some("wheel"));
        assert_eq!(app.screen, Screen::Event);
        assert!(app.body().iter().any(|line| format!("{line:?}").contains("unavailable")));
        app.cursor = 1;
        app.select();
        assert!(app.game.pending_event.is_none());
    }
    #[test]
    fn name_key_release_is_ignored_and_control_q_quits() {
        let mut app = App::new(GameContent::starter(), 1, Settings::default());
        app.screen = Screen::SetupParty;
        app.handle_key(KeyEvent::new_with_kind(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
            KeyEventKind::Release,
        ));
        assert_eq!(app.draft.names[0], "Leader");
        app.handle_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL));
        assert!(app.should_quit());
    }
    #[test]
    fn party_name_input_keeps_navigation_letters_and_spaces() {
        let mut app = App::new(GameContent::starter(), 1, Settings::default());
        app.screen = Screen::SetupParty;
        for key in ['h', 'j', 'k', 'l', ' ', '7'] {
            app.handle_key(KeyEvent::from(KeyCode::Char(key)));
        }
        assert_eq!(app.draft.names[0], "hjkl 7");
        app.handle_key(KeyEvent::from(KeyCode::Backspace));
        assert_eq!(app.draft.names[0], "hjkl ");
    }
    #[test]
    fn store_quantity_can_buy_bulk_food() {
        let mut app = App::new(GameContent::starter(), 2, Settings::default());
        app.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "farmer".into(),
            party: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
            departure_month: 4,
        });
        app.screen = Screen::Store;
        app.cursor = 1;
        app.store_quantity = 2_000;
        app.select();
        assert_eq!(app.game.inventory.get("food"), 2_000);
        assert_eq!(app.screen, Screen::Store);
    }
    #[test]
    fn loaded_content_journey_keeps_mandatory_phases_after_resume() {
        let content = pioneer_data::load().unwrap();
        let mut app = App::new(content, 41, Settings::default());
        app.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "banker".into(),
            party: vec!["A".into(), "B".into(), "C".into(), "D".into(), "E".into()],
            departure_month: 4,
        });
        app.screen = Screen::Store;
        app.cursor = app.game.content.items.iter().position(|item| item.id == "oxen").unwrap();
        app.store_quantity = 3;
        app.select();
        app.cursor = app.game.content.items.iter().position(|item| item.id == "food").unwrap();
        app.store_quantity = 2_000;
        app.select();
        app.cursor = app.game.content.items.len();
        app.select();
        let mut saw_event = false;
        for _ in 0..100 {
            match app.screen {
                Screen::Event => {
                    saw_event = true;
                    let event = app
                        .game
                        .content
                        .events
                        .iter()
                        .find(|e| Some(&e.id) == app.game.pending_event.as_ref())
                        .unwrap();
                    app.cursor = event
                        .choices
                        .iter()
                        .position(|choice| app.game.choice_available(choice))
                        .unwrap();
                    app.select()
                }
                Screen::River => break,
                _ => app.apply(Command::Continue),
            }
        }
        assert!(saw_event, "seeded loaded-content journey should exercise an event choice");
        assert_eq!(app.screen, Screen::River);
        let root = std::env::temp_dir().join(format!("pioneer-tui-phase-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&root);
        let storage = Storage::at(&root);
        storage.save_session("phase", &app.game).unwrap();
        let mut resumed =
            App::new(app.game.content.clone(), 99, Settings::default()).with_storage(storage);
        resumed.cursor = 1;
        resumed.select();
        assert_eq!(resumed.screen, Screen::River);
        resumed.back();
        assert_eq!(resumed.screen, Screen::River);
        let _ = std::fs::remove_dir_all(root);
    }

    fn outfitted_app() -> App {
        let mut app = App::new(pioneer_data::load().unwrap(), 23, Settings::default());
        app.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "banker".into(),
            party: ["Ada", "Ben", "Clara", "Dan", "Eve"].map(str::to_owned).to_vec(),
            departure_month: 3,
        });
        for (id, qty) in [("oxen", 3), ("food", 500), ("ammunition", 5)] {
            app.apply(Command::Buy { item_id: id.into(), quantity: qty });
        }
        app.apply(Command::Depart);
        app
    }
    #[test]
    fn live_hunt_keys_consume_exact_shots_and_commit_once_on_escape() {
        let mut app = outfitted_app();
        let day = app.game.day;
        app.handle_key(KeyEvent::from(KeyCode::Char('7')));
        assert_eq!(app.screen, Screen::Minigame);
        app.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        assert_eq!(app.game.inventory.get("ammunition"), 5, "world not committed yet");
        app.handle_key(KeyEvent::from(KeyCode::Esc));
        assert_eq!(app.game.inventory.get("ammunition"), 4);
        assert_eq!(app.game.loose_bullets, 19);
        assert_eq!(app.game.day, day + 1);
        assert!(app.game.active_minigame.is_none());
        for _ in 0..1000 {
            app.tick_minigame();
        }
        assert_eq!(app.game.day, day + 1, "result must not be applied twice");
    }
    #[test]
    fn text_minigame_renders_without_pixels_and_advances_only_on_commands() {
        let mut app = outfitted_app();
        app.settings.no_art = true;
        app.handle_key(KeyEvent::from(KeyCode::Char('7')));
        let before = app.minigame.as_ref().unwrap().text_lines();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let text = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(text.contains("TURN-BASED HUNT"));
        assert!(!text.contains(['▀', '▄', '█']));
        assert_eq!(app.minigame.as_ref().unwrap().text_lines(), before);
        app.handle_key(KeyEvent::from(KeyCode::Char('1')));
        assert_ne!(app.minigame.as_ref().unwrap().text_lines(), before);
        app.handle_key(KeyEvent::from(KeyCode::Esc));
        assert_eq!(app.game.inventory.get("ammunition"), 4);
        assert_eq!(app.game.loose_bullets, 19);
    }
    #[test]
    fn live_raft_escape_returns_to_fork_and_timer_completion_arrives() {
        let mut app = outfitted_app();
        app.game.current_node_id = Some("the_dalles".into());
        app.game.target_node_id = None;
        app.game.route_miles_remaining = 0;
        app.game.miles = 1813;
        app.game.status = pioneer_sim::RunStatus::AwaitingFork("the_dalles".into());
        app.sync_screen();
        app.handle_key(KeyEvent::from(KeyCode::Char('2')));
        assert_eq!(app.screen, Screen::Minigame);
        app.handle_key(KeyEvent::from(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Fork);
        assert_eq!(app.game.miles, 1813);
        app.handle_key(KeyEvent::from(KeyCode::Char('2')));
        for _ in 0..900 {
            app.tick_minigame();
        }
        assert_eq!(app.screen, Screen::Score);
        assert_eq!(app.game.status, pioneer_sim::RunStatus::Arrived);
        assert_eq!(app.game.miles, 1885);
    }
}
