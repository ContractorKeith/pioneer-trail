//! Sealed-letter errands use the real route graph; rendering fixtures only set the visible stop.

use crossterm::event::{KeyCode, KeyEvent};
use pioneer_sim::{Command, CrossMethod, GameState, JournalKind, Outcome, RunStatus};
use pioneer_trail::{
    app::App,
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
            "pioneer-letters-test-{}-{}",
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

fn apply(game: &mut GameState, command: Command) -> Vec<Outcome> {
    let outcomes = game.apply(command.clone());
    assert!(
        !outcomes.iter().any(|outcome| matches!(outcome, Outcome::Rejected(_))),
        "{command:?} rejected at {:?}: {outcomes:?}",
        game.status
    );
    outcomes
}

fn restore_fixture_health(game: &mut GameState) {
    game.inventory.quantities.insert("food".into(), 1_000);
    for member in &mut game.party {
        if member.alive {
            member.health = 100;
            member.ailments.clear();
        }
    }
}

fn advance_to_letter_delivery(mut game: GameState, letter_id: &str, storage: &Storage) {
    let definition =
        game.content.letters.iter().find(|letter| letter.id == letter_id).unwrap().clone();
    let destination_name = game
        .content
        .trails
        .iter()
        .find(|trail| trail.id == definition.trail_id)
        .and_then(|trail| trail.nodes.iter().find(|node| node.id == definition.destination_id))
        .unwrap()
        .name
        .clone();
    let mut accepted = false;
    let mut midleg_rejected = false;

    for _ in 0..500 {
        restore_fixture_health(&mut game);
        if let Some(event_id) = game.pending_event.clone() {
            let choice_id = game
                .content
                .events
                .iter()
                .find(|event| event.id == event_id)
                .unwrap()
                .choices
                .iter()
                .find(|choice| game.choice_available(choice))
                .unwrap()
                .id
                .clone();
            apply(&mut game, Command::Respond { event_id, choice_id });
            continue;
        }
        match game.status.clone() {
            RunStatus::AtLandmark(id) if id == definition.origin_id && !accepted => {
                assert_eq!(game.offered_letter().map(|letter| &letter.id), Some(&definition.id));
                let accepted_at = (game.day, game.miles);
                apply(&mut game, Command::AcceptLetter { letter_id: definition.id.clone() });
                assert!(matches!(
                    game.journal.entries.last(),
                    Some(pioneer_sim::JournalEntry {
                        day,
                        miles,
                        kind: JournalKind::LetterAccepted { recipient, destination, reward_cents },
                        ..
                    })
                        if recipient == &definition.recipient
                            && destination == &destination_name
                            && *reward_cents == definition.reward_cents
                            && (*day, *miles) == accepted_at
                ));
                let journal = game.journal.entries.clone();
                storage.save_game(&game).unwrap();
                game = storage.load_game().unwrap().unwrap();
                assert_eq!(
                    game.active_letter.as_ref().map(|letter| &letter.id),
                    Some(&definition.id)
                );
                assert_eq!(game.journal.entries, journal);
                accepted = true;
                apply(&mut game, Command::Continue);
            }
            RunStatus::AtLandmark(id) if id == definition.destination_id && accepted => {
                let cash = game.cash_cents;
                let delivered_at = (game.day, game.miles);
                assert!(game.can_deliver_letter());
                assert!(matches!(
                    apply(&mut game, Command::DeliverLetter).as_slice(),
                    [Outcome::LetterDelivered { reward_cents: 1_500, .. }]
                ));
                assert_eq!(game.cash_cents, cash + 1_500);
                assert!(matches!(
                    game.journal.entries.last(),
                    Some(pioneer_sim::JournalEntry {
                        day,
                        miles,
                        kind: JournalKind::LetterDelivered { recipient, destination, reward_cents },
                        ..
                    })
                        if recipient == &definition.recipient
                            && destination == &destination_name
                            && *reward_cents == definition.reward_cents
                            && (*day, *miles) == delivered_at
                ));
                let before_duplicate = serde_json::to_value(&game).unwrap();
                assert!(matches!(
                    game.apply(Command::DeliverLetter).as_slice(),
                    [Outcome::Rejected(_)]
                ));
                assert_eq!(serde_json::to_value(&game).unwrap(), before_duplicate);
                assert_eq!(
                    game.journal
                        .entries
                        .iter()
                        .filter(|entry| matches!(entry.kind, JournalKind::LetterAccepted { .. }))
                        .count(),
                    1
                );
                assert_eq!(
                    game.journal
                        .entries
                        .iter()
                        .filter(|entry| matches!(entry.kind, JournalKind::LetterDelivered { .. }))
                        .count(),
                    1
                );
                assert!(
                    midleg_rejected,
                    "letter reached its destination without a real travelling leg"
                );
                return;
            }
            RunStatus::AtLandmark(_) | RunStatus::Travelling => {
                if accepted && matches!(game.status, RunStatus::Travelling) && !midleg_rejected {
                    let before = serde_json::to_value(&game).unwrap();
                    assert!(!game.can_deliver_letter());
                    assert!(matches!(
                        game.apply(Command::DeliverLetter).as_slice(),
                        [Outcome::Rejected(_)]
                    ));
                    assert_eq!(serde_json::to_value(&game).unwrap(), before);
                    midleg_rejected = true;
                }
                apply(&mut game, Command::Continue);
            }
            RunStatus::AwaitingRiver(_) => {
                let method = if game.ferry_cost().is_some() {
                    CrossMethod::Ferry
                } else {
                    CrossMethod::Caulk
                };
                apply(&mut game, Command::CrossRiver { method });
            }
            RunStatus::AwaitingFork(_) => {
                let route_id = game.current_landmark().unwrap().routes.first().unwrap().id.clone();
                apply(&mut game, Command::ChooseRoute { route_id });
            }
            RunStatus::Arrived | RunStatus::Failed => {
                panic!("did not reach {}", definition.destination_id)
            }
            RunStatus::Setup | RunStatus::Outfitting => panic!("journey was not departed"),
        }
    }
    panic!("letter route did not reach {}", definition.destination_id);
}

#[test]
fn new_game_routes_accept_save_and_deliver_each_trail_letter_once() {
    let content = pioneer_data::load().unwrap();
    for (index, letter) in content.letters.iter().enumerate() {
        let temp = TempDir::new();
        let storage = Storage::at(&temp.0);
        let mut game = GameState::with_content(500 + index as u64, content.clone());
        apply(&mut game, Command::SetDifficulty(pioneer_sim::state::Difficulty::Easy));
        apply(
            &mut game,
            Command::Configure {
                trail_id: letter.trail_id.clone(),
                era_id: "1848".into(),
                occupation_id: "banker".into(),
                party: ["Ada", "Ben", "Clara", "Dora", "Eli"].map(str::to_owned).to_vec(),
                departure_month: 3,
            },
        );
        game.cash_cents = 100_000;
        apply(&mut game, Command::Buy { item_id: "oxen".into(), quantity: 3 });
        apply(&mut game, Command::Depart);
        restore_fixture_health(&mut game);
        advance_to_letter_delivery(game, &letter.id, &storage);
    }
}

fn render(app: &mut App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| (0..width).map(|x| buffer[(x, y)].symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn letter_app(no_art: bool) -> App {
    let mut app =
        App::new(pioneer_data::load().unwrap(), 41, Settings { no_art, ..Settings::default() });
    apply(
        &mut app.game,
        Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "carpenter".into(),
            party: ["Ada", "Ben", "Clara", "Daniel", "Eve"].map(str::to_owned).to_vec(),
            departure_month: 3,
        },
    );
    app.game.current_node_id = Some("fort_kearney".into());
    app.game.status = RunStatus::AtLandmark("fort_kearney".into());
    app.screen = Screen::Journey;
    app
}

#[test]
fn letters_open_from_camp_and_escape_returns_to_camp() {
    let mut app = letter_app(true);
    app.handle_key(KeyEvent::from(KeyCode::Char('c')));
    assert_eq!(app.screen, Screen::Camp);
    assert!(render(&mut app, 80, 24)
        .contains("1-9 choose · arrows move · Shift-J Journal · Shift-L letters · Esc trail"));

    app.handle_key(KeyEvent::from(KeyCode::Char('L')));
    assert_eq!(app.screen, Screen::Letters);
    app.handle_key(KeyEvent::from(KeyCode::Esc));
    assert_eq!(app.screen, Screen::Camp);
}

#[test]
fn sealed_letter_states_snapshot_at_both_sizes_with_and_without_art() {
    for no_art in [false, true] {
        for (width, height) in [(80, 24), (120, 40)] {
            let mut app = letter_app(no_art);
            app.handle_key(KeyEvent::from(KeyCode::Char('L')));
            let offer = render(&mut app, width, height);
            app.handle_key(KeyEvent::from(KeyCode::Enter));
            app.screen = Screen::Journey;
            app.handle_key(KeyEvent::from(KeyCode::Char('L')));
            let carrying = render(&mut app, width, height);
            app.game.current_node_id = Some("fort_laramie".into());
            app.game.status = RunStatus::AtLandmark("fort_laramie".into());
            app.screen = Screen::Journey;
            app.handle_key(KeyEvent::from(KeyCode::Char('L')));
            let delivery = render(&mut app, width, height);
            insta::assert_snapshot!(
                format!("letters_{}_{}x{}", if no_art { "no_art" } else { "art" }, width, height),
                format!("=== OFFER ===\n{offer}\n=== CARRYING ===\n{carrying}\n=== DELIVERY ===\n{delivery}")
            );
        }
    }
}
