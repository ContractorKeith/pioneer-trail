use crossterm::event::{KeyCode, KeyEvent};
use pioneer_sim::{state::PendingEvent, Command, GatheringActivity, Outcome, WeatherKind};
use pioneer_trail::{app::App, persist::Settings, screens::Screen};
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn gathering_cannot_cover_an_unfinished_hunt() {
    let mut app = App::new(pioneer_data::load().unwrap(), 41, Settings::default());
    for command in [
        Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "banker".into(),
            party: ["Ada", "Ben", "Clara", "Dan", "Eve"].map(str::to_owned).to_vec(),
            departure_month: 3,
        },
        Command::Buy { item_id: "oxen".into(), quantity: 3 },
        Command::Buy { item_id: "ammunition".into(), quantity: 2 },
        Command::Depart,
        Command::BeginHunt,
    ] {
        assert!(!app.game.apply(command).iter().any(|o| matches!(o, Outcome::Rejected(_))));
    }
    app.screen = Screen::Minigame;
    let before = serde_json::to_value(&app.game).unwrap();
    app.open_gathering(GatheringActivity::Forage);
    assert_eq!(app.screen, Screen::Minigame);
    assert_eq!(before, serde_json::to_value(&app.game).unwrap());
}

#[test]
fn travel_selected_event_is_mandatory_and_rendering_is_observational() {
    let mut app = App::new(pioneer_data::load().unwrap(), 41, Settings::default());
    for command in [
        Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "banker".into(),
            party: ["Ada", "Ben", "Clara", "Dan", "Eve"].map(str::to_owned).to_vec(),
            departure_month: 3,
        },
        Command::Buy { item_id: "oxen".into(), quantity: 3 },
        Command::Buy { item_id: "food".into(), quantity: 100 },
        Command::Buy { item_id: "wheel".into(), quantity: 1 },
        Command::Depart,
    ] {
        assert!(!app
            .game
            .apply(command)
            .iter()
            .any(|outcome| matches!(outcome, Outcome::Rejected(_))));
    }
    app.game
        .scheduled_events
        .push(PendingEvent { event_id: "wheel".into(), due_day: app.game.day + 1 });
    app.screen = Screen::Journey;

    app.handle_key(KeyEvent::from(KeyCode::Enter));
    assert_eq!(app.screen, Screen::Event);
    assert_eq!(app.game.pending_event.as_deref(), Some("wheel"));

    // The overlay is presentation only; it cannot advance its weather or event RNG.
    app.game.weather = WeatherKind::Rain;
    let before_render = serde_json::to_value(&app.game).unwrap();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    assert_eq!(serde_json::to_value(&app.game).unwrap(), before_render);

    app.handle_key(KeyEvent::from(KeyCode::Esc));
    assert_eq!(app.screen, Screen::Event);
    assert_eq!(serde_json::to_value(&app.game).unwrap(), before_render);

    app.handle_key(KeyEvent::from(KeyCode::Enter));
    assert!(app.game.pending_event.is_none());
    assert_ne!(app.screen, Screen::Event);
}
