//! Keyboard-only journeys exercise the same public input boundary as a terminal player.

use crossterm::event::{KeyCode, KeyEvent};
use pioneer_sim::RunStatus;
use pioneer_trail::{
    app::App,
    headless::RunConfig,
    persist::{Settings, Storage},
    screens::Screen,
};
use ratatui::{backend::TestBackend, Terminal};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "pioneer-keyboard-journeys-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn key(app: &mut App, code: KeyCode) {
    app.handle_key(KeyEvent::from(code));
}

fn enter(app: &mut App) {
    key(app, KeyCode::Enter);
}

fn render(app: &mut App, width: u16, height: u16) -> String {
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

fn loaded_app(seed: u64, storage: Storage) -> App {
    let content = pioneer_data::load().unwrap();
    assert!(content.events.len() >= 150, "keyboard paths require the full event catalogue");
    assert_eq!(content.quotes.len(), 200, "keyboard paths require the full conversation catalogue");
    let config = RunConfig {
        trail: "oregon".into(),
        era: "1848".into(),
        occupation: "banker".into(),
        month: 3,
    };
    let settings = Settings { no_art: true, ..Settings::default() };
    App::new(content, seed, settings).with_defaults(&config).with_storage(storage)
}

fn start_setup(app: &mut App) {
    enter(app); // Title: New Journey
    assert_eq!(app.screen, Screen::SetupTrail);
    enter(app); // Oregon, with 1848 selected by RunConfig defaults
    assert_eq!(app.screen, Screen::SetupOccupation);
    enter(app); // Banker
    assert_eq!(app.screen, Screen::SetupParty);
    for name in ["Ada", "Bert", "Clara", "Dora", "Eli"] {
        for character in name.chars() {
            key(app, KeyCode::Char(character));
        }
        enter(app);
    }
    enter(app); // Done
    assert_eq!(app.screen, Screen::SetupDeparture);
    assert!(render(app, 80, 24).contains("Departure month: 3"));
    enter(app);
    assert_eq!(app.screen, Screen::Store);
}

fn right(app: &mut App, count: usize) {
    for _ in 0..count {
        key(app, KeyCode::Right);
    }
}

fn down(app: &mut App, count: usize) {
    for _ in 0..count {
        key(app, KeyCode::Down);
    }
}

/// Purchases are deliberately ordered as displayed by the real store menu.
fn buy_standard_outfit(app: &mut App) {
    // Oxen starts selected.  The next food selection must normalize quantity before bulk input.
    right(app, 2);
    enter(app);
    assert_eq!(app.game.inventory.get("oxen"), 3);

    down(app, 1);
    right(app, 20);
    assert!(render(app, 80, 24).contains("Qty 2000"));
    enter(app);
    assert_eq!(app.game.inventory.get("food"), 2_000);

    for (item, quantity) in [
        ("clothing", 5usize),
        ("ammunition", 10),
        ("wheel", 1),
        ("axle", 1),
        ("tongue", 1),
        ("medicine", 2),
        ("tools", 1),
    ] {
        down(app, 1);
        right(app, quantity.saturating_sub(1));
        enter(app);
        assert_eq!(app.game.inventory.get(item), quantity as u32);
    }
}

fn depart(app: &mut App) {
    down(app, app.game.content.items.len());
    enter(app);
    assert_eq!(app.screen, Screen::Journey);
}

fn event_choice(app: &App, available: bool) -> Option<usize> {
    let id = app.game.pending_event.as_deref()?;
    app.game
        .content
        .events
        .iter()
        .find(|event| event.id == id)?
        .choices
        .iter()
        .position(|choice| app.game.choice_available(choice) == available)
}

fn choose_event(app: &mut App) {
    let choice = event_choice(app, true).expect("every pending event needs an available choice");
    key(app, KeyCode::Char(char::from_digit((choice + 1) as u32, 10).unwrap()));
}

fn reach_event(app: &mut App) {
    for _ in 0..150 {
        if app.screen == Screen::Event {
            return;
        }
        assert_eq!(app.screen, Screen::Journey, "unexpected screen before an event");
        enter(app);
    }
    panic!("seeded journey did not surface an event");
}

fn reach_river(app: &mut App) {
    for _ in 0..200 {
        match app.screen {
            Screen::River => return,
            Screen::Event => choose_event(app),
            Screen::Journey => enter(app),
            unexpected => panic!("unexpected screen before a river: {unexpected:?}"),
        }
    }
    panic!("seeded journey did not reach a river");
}

#[test]
fn keyboard_setup_store_and_menu_renders_are_complete() {
    let temp = TempDir::new();
    let mut app = loaded_app(41, Storage::at(&temp.0));
    start_setup(&mut app);
    buy_standard_outfit(&mut app);

    for (width, height) in [(80, 24), (120, 40)] {
        let view = render(&mut app, width, height);
        assert!(view.contains("GENERAL STORE"));
        assert!(view.contains("Food"));
        assert!(view.contains("Qty 1"));
        assert!(view.contains("TRAIL LOG"));
        assert!(!view.contains("requires 80"));
    }
}

#[test]
fn keyboard_event_rejection_is_mandatory_and_survives_resume() {
    let temp = TempDir::new();
    let storage = Storage::at(&temp.0);
    let mut app = loaded_app(41, storage.clone());
    start_setup(&mut app);
    buy_standard_outfit(&mut app);
    depart(&mut app);
    reach_event(&mut app);

    let event_id = app.game.pending_event.clone().unwrap();
    key(&mut app, KeyCode::Esc);
    assert_eq!(app.screen, Screen::Event);
    assert!(render(&mut app, 80, 24).contains("A decision is required"));

    let mut resumed = loaded_app(999, storage);
    resumed.resume();
    assert_eq!(resumed.screen, Screen::Event);
    assert_eq!(resumed.game.pending_event.as_deref(), Some(event_id.as_str()));
    choose_event(&mut resumed);
    assert_ne!(resumed.screen, Screen::Event);
}

#[test]
fn keyboard_river_rejection_is_mandatory_and_survives_resume() {
    let temp = TempDir::new();
    let storage = Storage::at(&temp.0);
    let mut app = loaded_app(41, storage.clone());
    start_setup(&mut app);
    buy_standard_outfit(&mut app);
    depart(&mut app);
    reach_river(&mut app);

    let river_id = app.game.current_node_id.clone().unwrap();
    key(&mut app, KeyCode::Esc);
    assert_eq!(app.screen, Screen::River);
    key(&mut app, KeyCode::Char('5')); // Guide is invalid away from Snake River.
    assert_eq!(app.screen, Screen::River);
    assert_eq!(app.game.current_node_id.as_deref(), Some(river_id.as_str()));

    let mut resumed = loaded_app(999, storage);
    resumed.resume();
    assert_eq!(resumed.screen, Screen::River);
    assert_eq!(resumed.game.current_node_id.as_deref(), Some(river_id.as_str()));
    key(&mut resumed, KeyCode::Char('3')); // Kansas River ferry: the safe keyboard choice.
    assert_eq!(resumed.screen, Screen::Journey);
}

#[test]
fn keyboard_journey_reaches_score_hall_and_fresh_setup() {
    let temp = TempDir::new();
    let storage = Storage::at(&temp.0);
    let mut app = loaded_app(41, storage);
    start_setup(&mut app);
    buy_standard_outfit(&mut app);
    depart(&mut app);

    let mut resupplies = 0;
    for _ in 0..600 {
        match app.screen {
            Screen::Score => break,
            Screen::Event => choose_event(&mut app),
            Screen::Fork => {
                // The first route at The Dalles is Barlow, avoiding the raft minigame.
                key(&mut app, KeyCode::Char('1'));
            }
            Screen::River => {
                let snake = app.game.current_node_id.as_deref() == Some("snake_river");
                let guide = snake && app.game.inventory.get("clothing") >= 3;
                let ferry = app
                    .game
                    .current_landmark()
                    .and_then(|node| node.river.as_ref())
                    .and_then(|river| river.ferry_cost_cents)
                    .is_some_and(|cost| cost <= app.game.cash_cents);
                key(
                    &mut app,
                    KeyCode::Char(if guide {
                        '5'
                    } else if ferry {
                        '3'
                    } else {
                        '2'
                    }),
                );
            }
            Screen::Journey
                if matches!(app.game.status, RunStatus::AtLandmark(_))
                    && app.game.current_landmark().is_some_and(|node| node.store) =>
            {
                while app.game.inventory.get("food") <= 1_900 {
                    let food_before = app.game.inventory.get("food");
                    key(&mut app, KeyCode::Char('b'));
                    assert_eq!(app.screen, Screen::Store);
                    down(&mut app, 1);
                    right(&mut app, 1);
                    enter(&mut app);
                    assert_eq!(app.game.inventory.get("food"), food_before + 100);
                    assert_eq!(app.screen, Screen::Store, "stay in the fort store for more supplies");
                    key(&mut app, KeyCode::Esc);
                    resupplies += 1;
                }
                enter(&mut app);
            }
            Screen::Journey => {
                if let Some(index) = app
                    .game
                    .party
                    .iter()
                    .position(|member| member.alive && !member.ailments.is_empty())
                {
                    key(&mut app, KeyCode::Char('i'));
                    down(&mut app, index);
                    enter(&mut app);
                    if app.screen == Screen::Treat { key(&mut app, KeyCode::Esc); }
                } else {
                    enter(&mut app);
                }
            }
            unexpected => panic!(
                "unexpected keyboard screen during journey: {unexpected:?}; status {:?}; node {:?}; resupplies {resupplies}",
                app.game.status, app.game.current_node_id
            ),
        }
    }

    assert_eq!(app.screen, Screen::Score, "keyboard policy should finish the journey");
    assert!(resupplies > 0, "keyboard policy should resupply at a fort");
    assert!(
        matches!(app.game.status, RunStatus::Arrived),
        "journey failed at {:?} with food {} and party {:?}",
        app.game.current_node_id,
        app.game.inventory.get("food"),
        app.game.party
    );
    assert_eq!(app.game.current_node_id.as_deref(), Some("willamette"));
    assert!(render(&mut app, 120, 40).contains("Arrived!"));

    key(&mut app, KeyCode::Esc);
    assert_eq!(app.screen, Screen::Title);
    key(&mut app, KeyCode::Char('3'));
    assert_eq!(app.screen, Screen::Hall);
    assert!(render(&mut app, 120, 40).contains("Ada"));
    key(&mut app, KeyCode::Esc);
    key(&mut app, KeyCode::Char('1'));
    assert_eq!(app.screen, Screen::SetupTrail);
}
