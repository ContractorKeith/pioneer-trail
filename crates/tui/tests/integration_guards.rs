use pioneer_sim::{Command, GatheringActivity, Outcome};
use pioneer_trail::{app::App, persist::Settings, screens::Screen};

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
