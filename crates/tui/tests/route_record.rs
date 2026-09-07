use crossterm::event::{KeyCode, KeyEvent};
use pioneer_sim::{Command, GameState, RunStatus};
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
