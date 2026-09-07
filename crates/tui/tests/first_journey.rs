//! Cross-feature acceptance checks for the first-play improvements.
use crossterm::event::{KeyCode, KeyEvent};
use pioneer_sim::{Command, RunStatus};
use pioneer_trail::{app::App, persist::Settings, screens::Screen};
use ratatui::{backend::TestBackend, Terminal};

fn carpenter(no_art: bool) -> App {
    let mut app =
        App::new(pioneer_data::load().unwrap(), 41, Settings { no_art, ..Settings::default() });
    let outcomes = app.game.apply(Command::Configure {
        trail_id: "oregon".into(),
        era_id: "1848".into(),
        occupation_id: "carpenter".into(),
        party: ["Ada", "Ben", "Clara", "Daniel", "Eve"].map(str::to_owned).to_vec(),
        departure_month: 3,
    });
    assert!(!outcomes.iter().any(|outcome| matches!(outcome, pioneer_sim::Outcome::Rejected(_))));
    app.screen = Screen::Store;
    app
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

#[test]
fn reading_outfitting_advice_never_spends_money_or_changes_the_world() {
    for no_art in [false, true] {
        let mut app = carpenter(no_art);
        let before = serde_json::to_string(&app.game).unwrap();
        app.handle_key(KeyEvent::from(KeyCode::Char('?')));
        for (width, height) in [(80, 24), (120, 40)] {
            let screen = render(&mut app, width, height);
            assert!(screen.contains("OUTFITTING ADVICE"), "advice did not open: {screen}");
            assert!(screen.to_lowercase().contains("food"), "{screen}");
            assert!(screen.contains("lb"), "food units must be visible: {screen}");
            assert!(screen.contains("Esc") || screen.contains('?'), "{screen}");
        }
        assert_eq!(serde_json::to_string(&app.game).unwrap(), before);
        app.handle_key(KeyEvent::from(KeyCode::Char('s')));
        assert_eq!(serde_json::to_string(&app.game).unwrap(), before);
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        assert_eq!(serde_json::to_string(&app.game).unwrap(), before);
    }
}

#[test]
fn risky_departure_warns_visibly_but_keeps_a_deliberate_override() {
    for no_art in [false, true] {
        let mut app = carpenter(no_art);
        app.game.apply(Command::Buy { item_id: "oxen".into(), quantity: 1 });
        for _ in 0..app.game.content.items.len() {
            app.handle_key(KeyEvent::from(KeyCode::Down));
        }
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        assert_eq!(app.game.status, RunStatus::Outfitting);
        let view = render(&mut app, 80, 24);
        assert!(view.to_lowercase().contains("again"), "override not explained: {view}");
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        assert_eq!(app.game.status, RunStatus::Travelling);
    }
}

#[test]
fn journey_advice_and_illustrations_never_advance_the_simulation() {
    for no_art in [false, true] {
        let mut app = carpenter(no_art);
        app.game.apply(Command::Buy { item_id: "oxen".into(), quantity: 3 });
        app.game.apply(Command::Buy { item_id: "food".into(), quantity: 500 });
        app.game.apply(Command::Depart);
        assert_eq!(app.game.status, RunStatus::Travelling);
        let before = serde_json::to_string(&app.game).unwrap();
        for screen in
            [Screen::Journey, Screen::Supplies, Screen::Pace, Screen::Rations, Screen::Rest]
        {
            app.screen = screen;
            for (width, height) in [(80, 24), (120, 40)] {
                let view = render(&mut app, width, height);
                assert!(view.contains("Esc"), "navigation clipped on {screen:?}: {view}");
                assert_eq!(serde_json::to_string(&app.game).unwrap(), before);
            }
        }
    }
}

#[test]
fn sealed_letter_screen_is_keyboard_only_and_renders_at_both_sizes() {
    for no_art in [false, true] {
        let mut app = carpenter(no_art);
        app.game.current_node_id = Some("fort_kearney".into());
        app.game.status = RunStatus::AtLandmark("fort_kearney".into());
        app.screen = Screen::Journey;
        app.handle_key(KeyEvent::from(KeyCode::Char('L')));
        assert_eq!(app.screen, Screen::Letters);
        for (width, height) in [(80, 24), (120, 40)] {
            let view = render(&mut app, width, height);
            assert!(
                view.contains("SEALED LETTER") && view.contains("$15.00"),
                "letter offer missing: {view}"
            );
        }
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        app.screen = Screen::Journey;
        app.handle_key(KeyEvent::from(KeyCode::Char('L')));
        let carrying = render(&mut app, 80, 24);
        assert!(carrying.contains("Carry this sealed letter to Fort Laramie"), "{carrying}");
        assert!(!carrying.contains("Deliver sealed letter"), "{carrying}");
        app.game.current_node_id = Some("fort_laramie".into());
        app.game.status = RunStatus::AtLandmark("fort_laramie".into());
        app.screen = Screen::Journey;
        app.handle_key(KeyEvent::from(KeyCode::Char('L')));
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        assert!(app.game.active_letter.is_none());
    }
}

#[test]
fn every_trail_offers_and_delivers_its_sealed_letter_at_the_actual_stops() {
    let content = pioneer_data::load().unwrap();
    for (index, definition) in content.letters.iter().enumerate() {
        let mut game = pioneer_sim::GameState::with_content(90 + index as u64, content.clone());
        assert!(!game
            .apply(Command::Configure {
                trail_id: definition.trail_id.clone(),
                era_id: "1848".into(),
                occupation_id: "farmer".into(),
                party: ["Ada", "Ben", "Clara", "Dora", "Eli"].map(str::to_owned).to_vec(),
                departure_month: 3,
            })
            .iter()
            .any(|outcome| matches!(outcome, pioneer_sim::Outcome::Rejected(_))));
        game.current_node_id = Some(definition.origin_id.clone());
        game.status = RunStatus::AtLandmark(definition.origin_id.clone());
        assert_eq!(game.offered_letter().map(|letter| &letter.id), Some(&definition.id));
        game.apply(Command::AcceptLetter { letter_id: definition.id.clone() });
        game.current_node_id = Some(definition.destination_id.clone());
        game.status = RunStatus::AtLandmark(definition.destination_id.clone());
        assert!(game.can_deliver_letter());
        assert!(matches!(
            game.apply(Command::DeliverLetter).as_slice(),
            [pioneer_sim::Outcome::LetterDelivered { letter_id, .. }] if letter_id == &definition.id
        ));
    }
}

#[test]
fn illness_advice_distinguishes_treatment_from_unavailable_medicine() {
    let mut app = carpenter(true);
    app.game.apply(Command::Buy { item_id: "oxen".into(), quantity: 3 });
    app.game.apply(Command::Buy { item_id: "food".into(), quantity: 500 });
    app.game.apply(Command::Depart);
    app.screen = Screen::Journey;
    for member in &mut app.game.party {
        member.skills.medicine = 0;
    }
    app.game.party[0].ailments.push("fever".into());
    assert!(render(&mut app, 80, 24).contains("no kit or trained medic"));
    app.game.inventory.quantities.insert("medicine".into(), 1);
    assert!(render(&mut app, 80, 24).contains("Press I to treat"));
    app.game.inventory.quantities.insert("food".into(), 0);
    assert!(render(&mut app, 80, 24).contains("Food is short"));
}

#[test]
fn long_name_failure_keeps_seed_and_exit_visible_in_every_display_mode() {
    for no_art in [false, true] {
        for color in
            [pioneer_trail::persist::ColorMode::Truecolor, pioneer_trail::persist::ColorMode::Mono]
        {
            let mut app = carpenter(no_art);
            app.settings.color = color;
            app.game.status = RunStatus::Failed;
            app.game.day = 23;
            app.game.miles = 120;
            for (index, member) in app.game.party.iter_mut().enumerate() {
                member.name = format!("Traveler {index} with long name");
                member.alive = false;
                member.health = 0;
            }
            app.screen = Screen::Score;
            for (width, height) in [(80, 24), (120, 40)] {
                let view = render(&mut app, width, height);
                assert!(view.contains("23 days"), "{view}");
                assert!(view.contains("120 miles"), "{view}");
                assert!(view.contains("pt-"), "seed clipped: {view}");
                assert!(view.contains("Epitaph"), "epitaph control clipped: {view}");
                assert!(view.contains("Enter"), "exit control clipped: {view}");
            }
        }
    }
}
