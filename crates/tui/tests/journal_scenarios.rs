use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pioneer_sim::{Command, DeathCause, GameState, JournalKind, Pace, RunStatus};
use pioneer_trail::{
    app::App,
    persist::{RunRecord, Settings, Storage},
    screens::Screen,
};
use ratatui::{backend::TestBackend, Terminal};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "pioneer-journal-scenarios-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn journey() -> GameState {
    let mut game = GameState::with_content(9, pioneer_data::load().unwrap());
    game.apply(Command::Configure {
        trail_id: "oregon".into(),
        era_id: "1848".into(),
        occupation_id: "banker".into(),
        party: ["Ada", "Ben", "Clara", "Dora", "Eli"].map(str::to_owned).to_vec(),
        departure_month: 3,
    });
    game.apply(Command::Buy { item_id: "oxen".into(), quantity: 3 });
    game.apply(Command::Buy { item_id: "food".into(), quantity: 500 });
    game.apply(Command::Depart);
    game
}

#[test]
fn departure_is_not_a_death() {
    let game = journey();
    assert!(matches!(
        game.journal.entries.first().map(|entry| &entry.kind),
        Some(JournalKind::Departed)
    ));
    assert!(!game
        .journal
        .entries
        .iter()
        .any(|entry| matches!(entry.kind, JournalKind::Death { .. })));
}

#[test]
fn rest_records_a_death_once_on_its_actual_first_day() {
    let mut game = journey();
    game.party[0].health = 1;
    game.party[0].ailments = vec!["broken_leg".into()];
    game.party[0].ailment_days.insert("broken_leg".into(), 2);
    let start = game.day;
    game.apply(Command::Rest { days: 3 });
    let deaths = game
        .journal
        .entries
        .iter()
        .filter(|entry| matches!(&entry.kind, JournalKind::Death { name, .. } if name == "Ada"))
        .collect::<Vec<_>>();
    assert_eq!(deaths.len(), 1);
    assert_eq!(deaths[0].day, start + 1);
    assert!(matches!(&deaths[0].kind, JournalKind::Death { cause: DeathCause::Ailments(_), .. }));
}

#[test]
fn hall_shift_j_opens_a_stored_journal_after_a_new_run() {
    let temp = Temp::new();
    let storage = Storage::at(&temp.0);
    let mut game = journey();
    game.journal.record(
        4,
        60,
        JournalKind::Landmark { landmark_id: "test".into(), name: "Test Point".into() },
    );
    storage
        .record_run(RunRecord {
            run_id: "stored".into(),
            leader: "Ada".into(),
            seed: 9,
            trail: "oregon".into(),
            era: 1848,
            ended_on: "1848-04-04".into(),
            occupation: "banker".into(),
            score: 1,
            survivors: 5,
            days: 4,
            miles: 60,
            arrived: true,
            epitaph: String::new(),
            cause: "Arrived".into(),
            journal: game.journal.entries.clone(),
        })
        .unwrap();
    let mut app = App::new(
        pioneer_data::load().unwrap(),
        10,
        Settings { no_art: true, ..Settings::default() },
    )
    .with_storage(storage);
    app.screen = Screen::Hall;
    app.handle_key(KeyEvent::new(KeyCode::Char('J'), KeyModifiers::SHIFT));
    assert_eq!(app.screen, Screen::Journal);
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let text =
        terminal.backend().buffer().content.iter().map(|cell| cell.symbol()).collect::<String>();
    assert!(text.contains("Test Point"));
}

#[test]
fn legacy_history_journal_is_explicitly_unknown() {
    let temp = Temp::new();
    fs::write(temp.0.join("hall_of_fame.json"), r#"{"runs":[{"run_id":"old","leader":"Ada","seed":1,"trail":"oregon","era":1848,"occupation":"farmer","score":1,"survivors":1,"days":1,"miles":1,"arrived":true,"epitaph":"","cause":"Unknown"}]}"#).unwrap();
    let mut app = App::new(
        pioneer_data::load().unwrap(),
        1,
        Settings { no_art: true, ..Settings::default() },
    )
    .with_storage(Storage::at(&temp.0));
    app.screen = Screen::Hall;
    app.handle_key(KeyEvent::new(KeyCode::Char('J'), KeyModifiers::SHIFT));
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let text =
        terminal.backend().buffer().content.iter().map(|cell| cell.symbol()).collect::<String>();
    assert!(text.contains("Journal unavailable"));
}

#[test]
fn exhaustion_death_keeps_its_source_cause() {
    let mut game = journey();
    game.ox_fatigue = 50;
    game.pace = Pace::Grueling;
    game.party[0].health = 1;
    game.apply(Command::TravelDay);
    assert!(game.journal.entries.iter().any(|entry| matches!(
        &entry.kind,
        JournalKind::Death { name, cause: DeathCause::Exhaustion } if name == "Ada"
    )));
}

#[test]
fn natural_recovery_is_once_and_never_written_for_a_dead_person() {
    let mut game = journey();
    let fever = game.content.ailments.iter_mut().find(|ailment| ailment.id == "fever").unwrap();
    fever.severity = 2;
    fever.daily_damage = 0;
    game.party[0].health = 80;
    game.party[0].ailments = vec!["fever".into()];
    game.party[0].ailment_days.insert("fever".into(), 5);
    game.party[1].alive = false;
    game.party[1].health = 0;
    game.party[1].ailments = vec!["fever".into()];
    game.party[1].ailment_days.insert("fever".into(), 5);
    game.apply(Command::Rest { days: 3 });
    let recoveries = game.journal.entries.iter().filter(|entry| matches!(
        &entry.kind, JournalKind::Recovered { name, ailment } if name == "Ada" && ailment == "fever"
    )).count();
    assert_eq!(recoveries, 1);
    assert!(!game.journal.entries.iter().any(|entry| matches!(
        &entry.kind, JournalKind::Recovered { name, .. } if name == "Ben"
    )));
}

#[test]
fn relationships_do_not_flood_the_journal_below_notable_thresholds() {
    let mut game = journey();
    for member in &mut game.party {
        for other in ["Ada", "Ben", "Clara", "Dora", "Eli"] {
            if member.name != other {
                member.relationships.affinity.insert(other.into(), 0);
            }
        }
    }
    game.apply(Command::Rest { days: 30 });
    let relationships = game
        .journal
        .entries
        .iter()
        .filter(|entry| matches!(entry.kind, JournalKind::Relationship { .. }))
        .count();
    assert!(
        relationships <= 10,
        "one threshold crossing at most per party pair, got {relationships}"
    );
}

#[test]
fn five_long_dead_names_leave_score_controls_visible() {
    let names = [
        "AveryLongTravelerNameOne",
        "BennettTravelerNameTwo",
        "ClaraTravelerNameThree",
        "DorianTravelerNameFour",
        "EloiseTravelerNameFive",
    ];
    let mut app = App::new(
        pioneer_data::load().unwrap(),
        4,
        Settings { no_art: true, ..Settings::default() },
    );
    app.game.apply(Command::Configure {
        trail_id: "oregon".into(),
        era_id: "1848".into(),
        occupation_id: "banker".into(),
        party: names.map(str::to_owned).to_vec(),
        departure_month: 3,
    });
    for member in &mut app.game.party {
        member.alive = false;
        member.health = 0;
    }
    app.game.status = RunStatus::Failed;
    app.screen = Screen::Score;
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let text =
        terminal.backend().buffer().content.iter().map(|cell| cell.symbol()).collect::<String>();
    assert!(text.contains("JOURNEY COMPLETE") || text.contains("THE TRAIL ENDS HERE"));
    assert!(text.contains("[J]") || text.contains("Journal"));
    assert!(text.contains("Enter returns"));
}

fn render(app: &mut App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .chunks(usize::from(width))
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn journal_view(no_art: bool, kind: JournalKind, status: RunStatus, day: u32, miles: u32) -> App {
    let mut app =
        App::new(pioneer_data::load().unwrap(), 12, Settings { no_art, ..Settings::default() });
    app.game = journey();
    app.game.day = day;
    app.game.miles = miles;
    app.game.status = status;
    if let JournalKind::Death { name, .. } = &kind {
        if let Some(member) = app.game.party.iter_mut().find(|member| member.name == *name) {
            member.alive = false;
            member.health = 0;
        }
    }
    app.game.journal.record(day, miles, kind);
    app.screen = Screen::Journal;
    app.handle_key(KeyEvent::from(KeyCode::Down));
    app
}

#[test]
fn journal_active_epilogue_and_memorial_snapshots_cover_sizes_and_art_modes() {
    let mut snapshots = String::new();
    for no_art in [false, true] {
        for (width, height) in [(80, 24), (120, 40)] {
            let views = [
                (
                    "active",
                    journal_view(
                        no_art,
                        JournalKind::Landmark {
                            landmark_id: "fort_kearney".into(),
                            name: "Fort Kearney".into(),
                        },
                        RunStatus::Travelling,
                        12,
                        304,
                    ),
                ),
                (
                    "epilogue",
                    journal_view(no_art, JournalKind::Arrived, RunStatus::Arrived, 120, 1_885),
                ),
                (
                    "memorial",
                    journal_view(
                        no_art,
                        JournalKind::Death { name: "Ada".into(), cause: DeathCause::Exhaustion },
                        RunStatus::Failed,
                        47,
                        932,
                    ),
                ),
            ];
            for (name, mut app) in views {
                snapshots.push_str(&format!("\n=== {name} art={} {width}x{height} ===\n", !no_art));
                snapshots.push_str(&render(&mut app, width, height));
                snapshots.push('\n');
            }
        }
    }
    insta::assert_snapshot!("journal_active_epilogue_memorial", snapshots);
}

#[test]
fn journal_and_memorial_render_full_cause_at_both_sizes_with_and_without_art() {
    for (width, height) in [(80, 24), (120, 40)] {
        for no_art in [false, true] {
            let mut app = App::new(
                pioneer_data::load().unwrap(),
                6,
                Settings { no_art, ..Settings::default() },
            );
            app.game = journey();
            app.game.status = RunStatus::Failed;
            app.game.journal.record(
                12,
                240,
                JournalKind::Death {
                    name: "AveryLongTravelerNameOne".into(),
                    cause: DeathCause::Ailments(vec![
                        "childbirth_complications".into(),
                        "mountain_fever".into(),
                    ]),
                },
            );
            app.screen = Screen::Journal;
            app.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
            let text = render(&mut app, width, height);
            assert!(text.contains("PARTY JOURNAL"));
            assert!(text.contains("IN MEMORY"));
            assert!(text.contains("childbirth"));
            assert!(text.contains("mountain"));
            assert!(text.contains("Esc returns"));
        }
    }
}

#[test]
fn journey_advertises_shift_j_without_clipping_at_both_sizes() {
    for (width, height) in [(80, 24), (120, 40)] {
        for no_art in [false, true] {
            let mut app = App::new(
                pioneer_data::load().unwrap(),
                8,
                Settings { no_art, ..Settings::default() },
            );
            app.game = journey();
            app.screen = Screen::Journey;
            let text = render(&mut app, width, height);
            assert!(text.contains("Shift-J Journal"));
        }
    }
}
