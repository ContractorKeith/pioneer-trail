//! Keyboard-only journeys exercise the same public input boundary as a terminal player.

use crossterm::event::{KeyCode, KeyEvent};
use pioneer_sim::{Command, Condition, Effect, EventChoice, EventDefinition, GameState, RunStatus};
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
    loaded_app_for(seed, storage, RunConfig::default())
}

fn loaded_app_for(seed: u64, storage: Storage, config: RunConfig) -> App {
    let content = pioneer_data::load().unwrap();
    assert!(content.events.len() >= 150, "keyboard paths require the full event catalogue");
    assert_eq!(content.quotes.len(), 200, "keyboard paths require the full conversation catalogue");
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
fn keyboard_hunt_resume_preserves_session_seed_and_spends_ammo_once() {
    let temp = TempDir::new();
    let storage = Storage::at(&temp.0);
    let mut app = loaded_app(52, storage.clone());
    start_setup(&mut app);
    buy_standard_outfit(&mut app);
    depart(&mut app);

    down(&mut app, 6);
    enter(&mut app);
    assert_eq!(app.screen, Screen::Minigame);
    let session = app.game.active_minigame.clone().expect("hunt session should be saved");
    let ammunition_before =
        app.game.inventory.get("ammunition") * 20 + u32::from(app.game.loose_bullets);

    let mut resumed = loaded_app(999, storage);
    resumed.resume();
    assert_eq!(resumed.screen, Screen::Minigame);
    assert_eq!(resumed.game.active_minigame.as_ref().map(|active| active.seed), Some(session.seed));
    assert_eq!(
        resumed.game.active_minigame.as_ref().map(|active| active.ammo_available),
        Some(session.ammo_available)
    );

    key(&mut resumed, KeyCode::Char('1'));
    key(&mut resumed, KeyCode::Esc);
    assert_eq!(resumed.screen, Screen::Journey);
    assert!(resumed.game.active_minigame.is_none());
    let ammunition_after =
        resumed.game.inventory.get("ammunition") * 20 + u32::from(resumed.game.loose_bullets);
    assert_eq!(ammunition_after, ammunition_before - 1);
}

#[test]
fn keyboard_trade_invite_and_dismiss_use_the_live_npc_menu() {
    let temp = TempDir::new();
    let mut app = loaded_app(53, Storage::at(&temp.0));
    start_setup(&mut app);
    buy_standard_outfit(&mut app);
    depart(&mut app);
    enter(&mut app); // Make room in the wagon's food supply before the trade.
    if app.screen == Screen::Event {
        choose_event(&mut app);
    }
    assert_eq!(app.screen, Screen::Journey);

    let food_before = app.game.inventory.get("food");
    key(&mut app, KeyCode::Char('u'));
    assert_eq!(app.screen, Screen::Trade);
    down(&mut app, 1); // Offer item.
    right(&mut app, 1); // Clothing.
    down(&mut app, 1); // Offer quantity.
    key(&mut app, KeyCode::Left);
    for _ in 0..48 {
        key(&mut app, KeyCode::Left);
    }
    down(&mut app, 1); // Requested item.
    key(&mut app, KeyCode::Left); // Food.
    down(&mut app, 2); // Trade action.
    enter(&mut app);
    assert_eq!(app.game.inventory.get("clothing"), 4);
    assert_eq!(app.game.inventory.get("food"), food_before + 1);

    down(&mut app, 2); // Invite.
    enter(&mut app);
    assert_eq!(app.game.party.len(), 6);
    assert!(app.game.party.iter().any(|member| member.npc_id.as_deref() == Some("emigrant_train")));

    down(&mut app, 1); // Dismiss.
    enter(&mut app);
    assert_eq!(app.game.party.len(), 5);
    assert!(app.game.party.iter().all(|member| member.npc_id.is_none()));
}

#[test]
fn terminal_event_save_resumes_at_score_without_a_pending_choice() {
    let temp = TempDir::new();
    let storage = Storage::at(&temp.0);
    let mut content = pioneer_data::load().unwrap();
    content.events.push(EventDefinition {
        id: "fatal_delay".into(),
        text: "The trail closes behind you.".into(),
        weight: 0,
        conditions: vec![Condition::Always],
        effects: vec![Effect::LoseDays(1)],
        choices: vec![EventChoice {
            id: "wait".into(),
            label: "Wait".into(),
            conditions: vec![],
            effects: vec![],
        }],
    });
    content.events.push(EventDefinition {
        id: "fatal_delay_trigger".into(),
        text: "A warning comes from the trail.".into(),
        weight: 1,
        conditions: vec![Condition::Always],
        effects: vec![Effect::Schedule { event_id: "fatal_delay".into(), days: 1 }],
        choices: vec![],
    });
    let mut game = GameState::with_content(54, content);
    game.apply(Command::Configure {
        trail_id: "oregon".into(),
        era_id: "1848".into(),
        occupation_id: "banker".into(),
        party: ["Ada", "Bert", "Clara", "Dora", "Eli"].map(str::to_owned).to_vec(),
        departure_month: 3,
    });
    game.apply(Command::Buy { item_id: "oxen".into(), quantity: 1 });
    game.apply(Command::Buy { item_id: "food".into(), quantity: 1 });
    game.apply(Command::Depart);
    game.inventory.quantities.insert("food".into(), 0);
    for member in &mut game.party {
        member.health = 10;
    }
    game.scheduled_events
        .push(pioneer_sim::state::PendingEvent { event_id: "fatal_delay".into(), due_day: 1 });
    game.apply(Command::Rest { days: 1 });
    assert_eq!(game.status, RunStatus::Failed);
    assert!(game.pending_event.is_none());
    storage.save_game(&game).unwrap();

    let mut app = loaded_app(999, storage);
    app.resume();
    assert_eq!(app.screen, Screen::Score);
    assert!(app.game.pending_event.is_none());
    assert!(render(&mut app, 80, 24).contains("JOURNEY COMPLETE"));
}

#[test]
fn keyboard_repair_key_resolves_a_loaded_wheel_event() {
    let temp = TempDir::new();
    let storage = Storage::at(&temp.0);
    let mut game = GameState::with_content(56, pioneer_data::load().unwrap());
    game.apply(Command::Configure {
        trail_id: "oregon".into(),
        era_id: "1848".into(),
        occupation_id: "banker".into(),
        party: ["Ada", "Bert", "Clara", "Dora", "Eli"].map(str::to_owned).to_vec(),
        departure_month: 3,
    });
    for (item_id, quantity) in [("oxen", 1), ("food", 100), ("wheel", 1)] {
        game.apply(Command::Buy { item_id: item_id.into(), quantity });
    }
    game.apply(Command::Depart);
    game.pending_event = Some("wheel".into());
    let day = game.day;
    storage.save_game(&game).unwrap();

    let mut app = loaded_app(999, storage);
    app.resume();
    assert_eq!(app.screen, Screen::Event);
    assert!(app.game.can_repair());
    key(&mut app, KeyCode::Char('r'));
    assert_eq!(app.screen, Screen::Journey);
    assert_eq!(app.game.day, day + 1);
    assert_eq!(app.game.inventory.get("wheel"), 0);
    assert!(app.game.pending_event.is_none());
}

#[test]
fn keyboard_1843_oregon_columbia_raft_reaches_willamette() {
    let temp = TempDir::new();
    let mut app = loaded_app_for(
        55,
        Storage::at(&temp.0),
        RunConfig {
            trail: "oregon".into(),
            era: "1843".into(),
            occupation: "banker".into(),
            month: 3,
        },
    );
    start_setup(&mut app);
    buy_standard_outfit(&mut app);
    depart(&mut app);

    let mut raft_started = false;
    for _ in 0..800 {
        match app.screen {
            Screen::Score => break,
            Screen::Event => choose_event(&mut app),
            Screen::River => {
                let guide = app.game.current_node_id.as_deref() == Some("snake_river")
                    && app.game.inventory.get("clothing") >= app.game.guide_cost();
                let ferry = app.game.ferry_cost().is_some_and(|cost| cost <= app.game.cash_cents);
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
            Screen::Fork if app.game.current_node_id.as_deref() == Some("the_dalles") => {
                key(&mut app, KeyCode::Char('2')); // Columbia, because Barlow is unavailable in 1843.
            }
            Screen::Fork => key(&mut app, KeyCode::Char('1')),
            Screen::Minigame => {
                raft_started = true;
                key(&mut app, KeyCode::Char('2')); // Turn-based center lane; 30 inputs finish the real course.
            }
            Screen::Journey
                if matches!(app.game.status, RunStatus::AtLandmark(_))
                    && app.game.can_shop()
                    && app.game.inventory.get("food") < 1_900 =>
            {
                while app.game.inventory.get("food") <= 1_900 {
                    let food_before = app.game.inventory.get("food");
                    key(&mut app, KeyCode::Char('b'));
                    assert_eq!(app.screen, Screen::Store);
                    for _ in 0..app.game.content.items.len() {
                        key(&mut app, KeyCode::Up);
                    }
                    down(&mut app, 1);
                    right(&mut app, 1);
                    enter(&mut app);
                    assert_eq!(app.game.inventory.get("food"), food_before + 100);
                }
                key(&mut app, KeyCode::Esc);
                enter(&mut app);
            }
            Screen::Journey => {
                if let Some(index) = (app.game.inventory.get("medicine") > 0)
                    .then(|| {
                        app.game
                            .party
                            .iter()
                            .position(|member| member.alive && !member.ailments.is_empty())
                    })
                    .flatten()
                {
                    key(&mut app, KeyCode::Char('i'));
                    down(&mut app, index);
                    enter(&mut app);
                    if app.screen == Screen::Treat {
                        key(&mut app, KeyCode::Esc);
                    }
                } else {
                    enter(&mut app);
                }
            }
            unexpected => panic!("unexpected screen on Columbia route: {unexpected:?}"),
        }
    }

    assert!(
        raft_started,
        "1843 route must enter the Columbia rafting minigame; status {:?}, node {:?}, mile {}, day {}, food {}",
        app.game.status,
        app.game.current_node_id,
        app.game.miles,
        app.game.day,
        app.game.inventory.get("food")
    );
    assert_eq!(
        app.screen,
        Screen::Score,
        "status {:?}, day {}, food {}",
        app.game.status,
        app.game.day,
        app.game.inventory.get("food")
    );
    assert_eq!(app.game.status, RunStatus::Arrived);
    assert_eq!(app.game.current_node_id.as_deref(), Some("willamette"));
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
                let guide = snake && app.game.inventory.get("clothing") >= app.game.guide_cost();
                let ferry = app.game.ferry_cost().is_some_and(|cost| cost <= app.game.cash_cents);
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

#[test]
fn keyboard_talk_at_a_fort_offers_a_named_speaker_grounded_in_state() {
    let temp = TempDir::new();
    let mut app = loaded_app(11, Storage::at(&temp.0));
    start_setup(&mut app);
    buy_standard_outfit(&mut app);
    depart(&mut app);

    app.game.current_node_id = Some("fort_kearney".into());
    app.game.target_node_id = Some("chimney_rock".into());
    app.game.route_miles_remaining = 250;
    app.game.miles = 304;
    app.game.status = RunStatus::AtLandmark("fort_kearney".into());
    // Eat down from the full 2000 lb store limit so there is room for a favor.
    app.game.inventory.quantities.insert("food".into(), 500);

    key(&mut app, KeyCode::Char('t'));
    assert_eq!(app.screen, Screen::Talk);
    let view = render(&mut app, 80, 24);
    assert!(view.contains("Speak with"), "a named speaker should be listed:\n{view}");

    enter(&mut app); // Choose the first listed speaker.
    let view = render(&mut app, 80, 24);
    assert!(view.contains("Ask about the route"), "topics should follow speaker choice:\n{view}");

    key(&mut app, KeyCode::Esc); // Esc backs out of the topic choice, not the whole screen.
    assert_eq!(app.screen, Screen::Talk);
    let view = render(&mut app, 80, 24);
    assert!(view.contains("Speak with"), "Esc should return to the speaker list:\n{view}");

    enter(&mut app); // Choose the speaker again.
    enter(&mut app); // Ask about the route.
    assert_eq!(app.screen, Screen::Talk, "the talk screen should stay open after an answer");
    assert_eq!(app.game.conversation_memory.len(), 1);
    let (speaker_id, memory) = app.game.conversation_memory.iter().next().unwrap();
    let speaker_id = speaker_id.clone();
    assert_eq!(memory.times_talked, 1);
    assert!(!memory.favor_received, "a favor must not be granted on a first meeting");
    let view = render(&mut app, 80, 24);
    assert!(
        view.contains("Chimney Rock") && view.contains("250"),
        "the route answer must name the real next landmark and distance:\n{view}"
    );

    enter(&mut app); // Dismiss the complete attributed reply.
    let food_before = app.game.inventory.get("food");

    enter(&mut app); // Approach the same speaker again, same day.
    down(&mut app, 1);
    enter(&mut app); // Ask about supplies this time.
    let memory = &app.game.conversation_memory[&speaker_id];
    assert_eq!(memory.times_talked, 2);
    assert!(!memory.favor_received, "same-day repeats are not a return visit");
    assert_eq!(app.game.inventory.get("food"), food_before);

    app.game.day += 1; // A real return visit, a day later.
    enter(&mut app); // Dismiss the previous reply.
    enter(&mut app); // Approach again.
    down(&mut app, 2);
    enter(&mut app); // Ask for news.
    let memory = &app.game.conversation_memory[&speaker_id];
    assert_eq!(memory.times_talked, 3);
    assert!(memory.favor_received, "a later-day return should be recognized with a one-time favor");
    assert_eq!(app.game.inventory.get("food"), food_before + 15);

    app.game.day += 1;
    enter(&mut app); // Dismiss the previous reply.
    enter(&mut app); // Approach a fourth time, another later day.
    down(&mut app, 0);
    enter(&mut app); // Ask about the route again.
    assert_eq!(
        app.game.inventory.get("food"),
        food_before + 15,
        "the favor must not be granted a second time"
    );
}

#[test]
fn keyboard_talk_no_art_mode_describes_the_setting_in_text() {
    let temp = TempDir::new();
    let mut app = loaded_app(11, Storage::at(&temp.0));
    app.settings.no_art = true;
    start_setup(&mut app);
    buy_standard_outfit(&mut app);
    depart(&mut app);

    app.game.current_node_id = Some("fort_kearney".into());
    app.game.status = RunStatus::AtLandmark("fort_kearney".into());
    key(&mut app, KeyCode::Char('t'));

    let view = render(&mut app, 80, 24);
    assert!(view.contains("AT THE FORT"), "no-art mode must still describe the setting:\n{view}");
    assert!(!view.contains('▀'), "no-art mode must not draw pixel art:\n{view}");
}

#[test]
fn keyboard_talk_speakers_vanish_once_the_party_leaves_the_fort() {
    let temp = TempDir::new();
    let mut app = loaded_app(11, Storage::at(&temp.0));
    start_setup(&mut app);
    buy_standard_outfit(&mut app);
    depart(&mut app);

    app.game.current_node_id = Some("fort_kearney".into());
    app.game.status = RunStatus::AtLandmark("fort_kearney".into());
    key(&mut app, KeyCode::Char('t'));
    let at_fort = render(&mut app, 80, 24);
    assert!(at_fort.contains("Speak with"), "a named speaker should be listed at the fort");
    key(&mut app, KeyCode::Esc);

    // The party has moved on; `current_node_id` still names the fort as the last
    // stop, but the status is no longer AtLandmark there.
    app.game.status = RunStatus::Travelling;
    app.game.target_node_id = Some("chimney_rock".into());
    app.game.route_miles_remaining = 120;
    key(&mut app, KeyCode::Char('t'));
    let underway = render(&mut app, 80, 24);
    assert!(
        !underway.contains("Speak with Silas Enright"),
        "the departed fort's speaker must not remain reachable:\n{underway}"
    );
    assert!(underway.contains("Listen to the camp"), "the fallback quote option must remain");
}

#[test]
fn keyboard_talk_max_capacity_grants_no_food_and_no_false_claim() {
    let temp = TempDir::new();
    let mut app = loaded_app(11, Storage::at(&temp.0));
    start_setup(&mut app);
    buy_standard_outfit(&mut app);
    depart(&mut app);

    app.game.current_node_id = Some("fort_kearney".into());
    app.game.status = RunStatus::AtLandmark("fort_kearney".into());
    assert_eq!(app.game.inventory.get("food"), 2_000, "store limit leaves no room for a favor");

    key(&mut app, KeyCode::Char('t'));
    enter(&mut app); // Choose the first speaker.
    enter(&mut app); // Ask about the route.

    app.game.day += 1;
    enter(&mut app); // Dismiss the previous reply.
    enter(&mut app); // Return a day later.
    down(&mut app, 1);
    enter(&mut app); // Ask about supplies.
    let view = render(&mut app, 80, 24);
    assert_eq!(app.game.inventory.get("food"), 2_000, "a full wagon must not overflow its limit");
    assert!(
        !view.contains("leaves you"),
        "a full wagon must not falsely claim a food delivery:\n{view}"
    );
    let memory = app.game.conversation_memory.values().next().unwrap();
    assert!(
        !memory.favor_received,
        "an ungranted favor stays available for a later, roomier visit"
    );
}

#[test]
fn keyboard_talk_wagon_favor_only_follows_a_real_trade() {
    let temp = TempDir::new();
    let mut app = loaded_app(53, Storage::at(&temp.0));
    start_setup(&mut app);
    buy_standard_outfit(&mut app);
    depart(&mut app);
    enter(&mut app); // Make room in the wagon's food supply before any trade.
    if app.screen == Screen::Event {
        choose_event(&mut app);
    }
    assert_eq!(app.screen, Screen::Journey);

    key(&mut app, KeyCode::Char('t'));
    assert_eq!(app.screen, Screen::Talk);
    let view = render(&mut app, 80, 24);
    assert!(view.contains("neighboring wagon"), "a nearby NPC train should be listed:\n{view}");
    enter(&mut app); // Approach the wagon speaker.
    enter(&mut app); // Ask a topic.
    app.game.day += 1;
    enter(&mut app); // Dismiss the previous reply.
    enter(&mut app); // Return a day later, still without ever having traded.
    down(&mut app, 1);
    enter(&mut app);
    let food_before = app.game.inventory.get("food");
    assert!(
        app.game.conversation_memory.values().all(|memory| !memory.favor_received),
        "mere repeated talk must not fabricate trade familiarity"
    );
    key(&mut app, KeyCode::Esc); // Dismiss the reply to the speaker list.
    key(&mut app, KeyCode::Esc); // Return to the trail.
    assert_eq!(app.screen, Screen::Journey);

    // Now trade for real.
    key(&mut app, KeyCode::Char('u'));
    assert_eq!(app.screen, Screen::Trade);
    down(&mut app, 1); // Offer item.
    right(&mut app, 1); // Clothing.
    down(&mut app, 1); // Offer quantity.
    for _ in 0..48 {
        key(&mut app, KeyCode::Left);
    }
    down(&mut app, 1); // Requested item.
    key(&mut app, KeyCode::Left); // Food.
    down(&mut app, 2); // Trade action.
    enter(&mut app);
    assert!(app.game.npcs[0].last_reputation_day.is_some(), "a real trade must be recorded");
    key(&mut app, KeyCode::Esc);
    assert_eq!(app.screen, Screen::Journey);

    app.game.day += 1;
    key(&mut app, KeyCode::Char('t'));
    enter(&mut app); // Approach the wagon speaker again.
    enter(&mut app); // Ask a topic.
    assert!(
        app.game.conversation_memory.values().any(|memory| memory.favor_received),
        "a real prior trade should now be recognized with a favor"
    );
    assert!(app.game.inventory.get("food") >= food_before);
}
