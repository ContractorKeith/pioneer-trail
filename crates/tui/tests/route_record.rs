use crossterm::event::{KeyCode, KeyEvent};
use pioneer_sim::{
    content::{LandmarkDefinition, LandmarkKind, RouteDefinition},
    route_record::Visit,
    Command, CrossMethod, GameState, RunStatus,
};
use pioneer_trail::{app::App, persist::Settings, screens::Screen};
use ratatui::{backend::TestBackend, Terminal};

fn game(trail: &str) -> GameState {
    let mut content = pioneer_data::load().unwrap();
    content.events.clear();
    let mut game = GameState::with_content(23, content);
    game.apply(Command::Configure {
        trail_id: trail.into(),
        era_id: "1848".into(),
        occupation_id: "banker".into(),
        party: ["Ada", "Ben", "Clara", "Dan", "Eve"].map(str::to_owned).to_vec(),
        departure_month: 3,
    });
    for (id, quantity) in [("oxen", 3), ("food", 500)] {
        game.apply(Command::Buy { item_id: id.into(), quantity });
    }
    game.apply(Command::Depart);
    game
}

#[test]
fn visits_are_factual_persisted_and_old_saves_remain_unknown() {
    for trail in ["oregon", "california", "mormon"] {
        let mut game = game(trail);
        assert_eq!(game.visited_landmarks.len(), 1);
        let target = game.target_node_id.clone().unwrap();
        game.route_miles_remaining = 1;
        game.apply(Command::TravelDay);
        assert_eq!(game.visited_landmarks.last().unwrap().landmark_id, target);
        assert_eq!(game.visited_landmarks.last().unwrap().day, game.day);
        let json = serde_json::to_value(&game).unwrap();
        let restored: GameState = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(restored.visited_landmarks, game.visited_landmarks);
        let mut legacy = json;
        legacy.as_object_mut().unwrap().remove("visited_landmarks");
        let old: GameState = serde_json::from_value(legacy).unwrap();
        assert!(old.visited_landmarks.is_empty());
    }
}

#[test]
fn selecting_a_fork_does_not_stamp_the_other_branch() {
    let mut original = game("oregon");
    original.current_node_id = Some("south_pass".into());
    original.status = RunStatus::AwaitingFork("south_pass".into());
    original.target_node_id = None;
    let routes = original.current_landmark().unwrap().routes.clone();
    assert_eq!(routes.len(), 2);
    for route in &routes {
        let mut game = original.clone();
        game.apply(Command::ChooseRoute { route_id: route.id.clone() });
        assert_eq!(game.target_node_id.as_ref(), Some(&route.target_id));
        game.route_miles_remaining = 1;
        game.apply(Command::TravelDay);
        assert!(game.visited_landmarks.iter().any(|v| v.landmark_id == route.target_id));
        let other = routes.iter().find(|r| r.id != route.id).unwrap();
        assert!(!game.visited_landmarks.iter().any(|v| v.landmark_id == other.target_id));
    }
}

fn apply(game: &mut GameState, command: Command) {
    let outcomes = game.apply(command);
    assert!(
        !outcomes.iter().any(|outcome| matches!(outcome, pioneer_sim::Outcome::Rejected(_))),
        "command was rejected: {outcomes:?}"
    );
}

fn reach_node(game: &mut GameState, target: &str) {
    for _ in 0..200 {
        if game.current_node_id.as_deref() == Some(target) {
            return;
        }
        // Keep the fixture focused on route commands rather than provisioning
        // or illness attrition; it never changes route position or choices.
        game.inventory.quantities.insert("food".into(), 500);
        for member in game.party.iter_mut().filter(|member| member.alive) {
            member.health = 100;
        }
        match &game.status {
            RunStatus::Travelling => apply(game, Command::TravelDay),
            RunStatus::AtLandmark(_) => apply(game, Command::Continue),
            // Caulking can lose cargo but cannot choose a casualty, keeping this
            // command-only fixture deterministic enough to reach the fork.
            RunStatus::AwaitingRiver(_) => {
                apply(game, Command::CrossRiver { method: CrossMethod::Caulk })
            }
            status => panic!("could not reach {target}; stopped at {status:?}"),
        }
    }
    panic!("did not reach {target} within the command budget");
}

#[test]
fn real_journey_commands_stamp_only_the_oregon_fork_that_was_taken() {
    for (route_id, target, supply, miles) in [
        ("green", "green_river", "Fort Hall", 268),
        ("bridger", "fort_bridger", "Fort Bridger", 219),
    ] {
        let mut state = game("oregon");
        reach_node(&mut state, "south_pass");
        assert!(matches!(state.status, RunStatus::AwaitingFork(_)));
        assert!(state.visited_landmarks.iter().any(|visit| visit.landmark_id == "south_pass"));

        apply(&mut state, Command::ChooseRoute { route_id: route_id.into() });
        assert_eq!(state.next_supply_stop(), Some((supply, miles)));
        reach_node(&mut state, target);

        assert!(state.visited_landmarks.iter().any(|visit| visit.landmark_id == target));
        let other = if target == "green_river" { "fort_bridger" } else { "green_river" };
        assert!(!state.visited_landmarks.iter().any(|visit| visit.landmark_id == other));
    }
}

#[test]
fn visits_reject_forged_history_but_allow_a_partial_legacy_start() {
    let mut state = game("oregon");
    let target = state.target_node_id.clone().unwrap();
    state.route_miles_remaining = 1;
    state.apply(Command::TravelDay);
    assert_eq!(state.current_node_id.as_deref(), Some(target.as_str()));
    assert!(state.validate().is_ok());

    for forged in [
        Visit { landmark_id: "not-a-stop".into(), day: state.day, mile: state.miles },
        Visit { landmark_id: target.clone(), day: state.day + 1, mile: state.miles },
        Visit { landmark_id: target.clone(), day: state.day, mile: state.miles + 1 },
        Visit { landmark_id: "willamette".into(), day: state.day, mile: state.miles },
    ] {
        let mut corrupt = state.clone();
        corrupt.visited_landmarks.push(forged);
        assert!(corrupt.validate().is_err());
    }

    let mut partial = game("oregon");
    partial.current_node_id = Some("south_pass".into());
    partial.target_node_id = None;
    partial.status = RunStatus::AwaitingFork("south_pass".into());
    partial.day = 20;
    partial.miles = 932;
    partial.visited_landmarks =
        vec![Visit { landmark_id: "south_pass".into(), day: partial.day, mile: partial.miles }];
    assert!(partial.validate().is_ok(), "a migrated history can begin after the trailhead");
}

fn south_pass_game() -> GameState {
    let mut game = game("oregon");
    let trail = game.content.trails.iter().find(|trail| trail.id == "oregon").unwrap();
    let ids = [
        "independence",
        "kansas_river",
        "big_blue",
        "fort_kearney",
        "chimney_rock",
        "fort_laramie",
        "independence_rock",
        "south_pass",
    ];
    game.visited_landmarks = ids
        .iter()
        .enumerate()
        .map(|(day, id)| {
            let node = trail.nodes.iter().find(|node| node.id == *id).unwrap();
            Visit { landmark_id: (*id).into(), day: day as u32, mile: node.mile }
        })
        .collect();
    game.current_node_id = Some("south_pass".into());
    game.target_node_id = Some("fort_bridger".into());
    game.status = RunStatus::Travelling;
    game.route_miles_remaining = 110;
    game.day = 21;
    game.miles = 1_041;
    game
}

#[test]
fn map_marks_only_the_chosen_fork_and_places_the_wagon_mid_leg() {
    let game = south_pass_game();
    let mut app = App::new(game.content.clone(), 23, Settings::default());
    app.game = game;
    app.screen = Screen::Journey;
    app.handle_key(KeyEvent::from(KeyCode::Char('m')));
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let buffer = terminal.backend().buffer();

    // South Pass (index 7) to Fort Bridger (index 9) is the selected fork.
    assert_eq!(buffer[(63, 4)].fg, ratatui::style::Color::Rgb(27, 203, 1));
    // The unchosen Green River branch remains white.
    assert_eq!(buffer[(73, 4)].fg, ratatui::style::Color::Rgb(255, 255, 255));
    assert_eq!(buffer[(68, 2)].symbol(), "◆");
}

#[test]
fn legacy_history_renders_unknown_stops_in_text_mode() {
    let mut game = game("oregon");
    game.visited_landmarks.clear();
    let mut app =
        App::new(game.content.clone(), 23, Settings { no_art: true, ..Settings::default() });
    app.game = game;
    app.screen = Screen::Journey;
    app.handle_key(KeyEvent::from(KeyCode::Char('m')));
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let text =
        terminal.backend().buffer().content.iter().map(|cell| cell.symbol()).collect::<String>();
    assert!(text.contains("Earlier visits were not recorded in this save."));
    assert!(text.contains("oC Big Blue River Crossing · 185 route miles"));
    assert!(!text.contains("day 0, mile 0"));
}

#[test]
fn extended_future_trail_uses_a_safe_text_listing() {
    let mut game = game("oregon");
    let trail = game.content.trails.iter_mut().find(|trail| trail.id == "oregon").unwrap();
    for index in 0..9 {
        trail.nodes.push(LandmarkDefinition {
            id: format!("future_stop_{index}"),
            name: format!("Future stop {}", index + 1),
            mile: 2_000 + index,
            kind: LandmarkKind::Landmark,
            routes: vec![RouteDefinition {
                id: "finish".into(),
                label: "Continue to journey's end".into(),
                target_id: "willamette".into(),
                distance_miles: 1,
            }],
            river: None,
            store: false,
        });
    }
    let mut app = App::new(game.content.clone(), 23, Settings::default());
    app.game = game;
    app.screen = Screen::Journey;
    app.handle_key(KeyEvent::from(KeyCode::Char('m')));
    for _ in 0..26 {
        app.handle_key(KeyEvent::from(KeyCode::Down));
    }
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let text =
        terminal.backend().buffer().content.iter().map(|cell| cell.symbol()).collect::<String>();
    assert!(text.contains("ROUTE MAP LISTING · 27 stops"));
    assert!(text.contains("27 Future stop 9"));
}

#[test]
fn map_is_read_only_and_fits_all_trails_and_modes() {
    for trail in ["oregon", "california", "mormon"] {
        for no_art in [false, true] {
            let game = game(trail);
            let mut app =
                App::new(game.content.clone(), 23, Settings { no_art, ..Settings::default() });
            app.game = game;
            app.screen = Screen::Journey;
            let before = serde_json::to_value(&app.game).unwrap();
            app.handle_key(KeyEvent::from(KeyCode::Char('m')));
            for (w, h) in [(80, 24), (120, 40)] {
                let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
                term.draw(|frame| app.render(frame)).unwrap();
                let text =
                    term.backend().buffer().content.iter().map(|c| c.symbol()).collect::<String>();
                for label in [
                    "ROUTE RECORD",
                    "miles remain",
                    "Nearest reachable supply",
                    "Esc back",
                    "day 0, mile 0",
                ] {
                    assert!(text.contains(label), "missing {label} on {trail}: {text}");
                }
            }
            app.handle_key(KeyEvent::from(KeyCode::Esc));
            assert_eq!(app.screen, Screen::Journey);
            assert_eq!(serde_json::to_value(&app.game).unwrap(), before);
        }
    }
}
