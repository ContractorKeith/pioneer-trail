use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
    Digit(u8),
    Character(char),
    Quit,
    None,
}

pub fn decode(key: KeyEvent) -> Input {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return Input::None;
    }
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Input::Up,
        KeyCode::Down | KeyCode::Char('j') => Input::Down,
        KeyCode::Left | KeyCode::Char('h') => Input::Left,
        KeyCode::Right | KeyCode::Char('l') => Input::Right,
        KeyCode::Enter | KeyCode::Char(' ') => Input::Select,
        KeyCode::Esc => Input::Back,
        KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => Input::Quit,
        KeyCode::Char(c) if c.is_ascii_digit() => Input::Digit(c as u8 - b'0'),
        KeyCode::Char(c) => Input::Character(c),
        _ => Input::None,
    }
}
