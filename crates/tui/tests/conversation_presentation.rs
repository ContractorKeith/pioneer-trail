use crossterm::event::{KeyCode, KeyEvent};
use pioneer_sim::{Command, RunStatus};
use pioneer_trail::{app::App, persist::Settings, screens::Screen};
use ratatui::{backend::TestBackend, Terminal};

fn fort_app(no_art: bool) -> App {
    let mut app =
        App::new(pioneer_data::load().unwrap(), 7, Settings { no_art, ..Settings::default() });
    app.game.apply(Command::Configure {
        trail_id: "oregon".into(),
        era_id: "1848".into(),
        occupation_id: "banker".into(),
        party: ["Ada", "Ben", "Clara", "Dora", "Eli"].map(str::to_owned).to_vec(),
        departure_month: 3,
    });
    app.game.current_node_id = Some("fort_kearney".into());
    app.game.target_node_id = None;
    app.game.status = RunStatus::AtLandmark("fort_kearney".into());
    app.screen = Screen::Journey;
    app
}

fn view(app: &mut App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    terminal.backend().buffer().content.iter().map(|cell| cell.symbol()).collect()
}

#[test]
fn response_keeps_speaker_and_all_lines_until_dismissed_in_every_layout() {
    for (no_art, width, height) in
        [(false, 80, 24), (false, 120, 40), (true, 80, 24), (true, 120, 40)]
    {
        let mut app = fort_app(no_art);
        app.handle_key(KeyEvent::from(KeyCode::Char('t')));
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        app.handle_key(KeyEvent::from(KeyCode::Enter));
        let rendered = view(&mut app, width, height);
        assert!(rendered.contains("Silas Enright"), "{width}x{height} no_art={no_art}");
        assert!(rendered.contains("Rest the oxen a spell"), "{width}x{height} no_art={no_art}");
        assert!(
            rendered.contains("The road runs on to Chimney Rock"),
            "{width}x{height} no_art={no_art}"
        );
        assert!(rendered.contains("Enter/Esc returns"), "{width}x{height} no_art={no_art}");
        app.handle_key(KeyEvent::from(KeyCode::Esc));
        assert_eq!(app.screen, Screen::Talk);
    }
}
