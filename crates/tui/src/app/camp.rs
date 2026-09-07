use super::*;

impl App {
    pub(super) fn open_camp(&mut self) {
        self.camp_return = true;
        self.auto_travel = false;
        self.cursor = 0;
        self.screen = Screen::Camp;
    }

    pub(super) fn select_camp(&mut self) {
        let choice = self.cursor;
        self.cursor = 0;
        match choice {
            0 => self.screen = Screen::Rest,
            1 => self.screen = Screen::Treat,
            2 => self.screen = Screen::Supplies,
            3 => self.apply(Command::Forage),
            4 => self.apply(Command::Fish),
            5 => self.apply(Command::BeginHunt),
            6 => self.screen = Screen::Party,
            7 => self.screen = Screen::Talk,
            _ => self.back(),
        }
    }

    pub(super) fn render_camp(&self, frame: &mut Frame) {
        use crate::art;
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black).fg(Color::White)),
            area,
        );
        let x = area.x + (area.width - 80) / 2;
        let scene = Rect::new(x, area.y, 80, 12);
        if !self.settings.no_art {
            let mode = self.color_mode();
            art::render_section(
                art::embedded("camp_night.px").expect("camp art"),
                frame.buffer_mut(),
                scene,
                4,
                mode,
            );
            for (file, dx, dy) in [("wagon_0.px", 1, 1), ("ox_0.px", 60, 6)] {
                art::render_at(
                    art::embedded(file).expect("camp actor"),
                    frame.buffer_mut(),
                    scene,
                    i32::from(x + dx),
                    i32::from(area.y + dy),
                    mode,
                );
            }
            let fire = if self.animation_tick.is_multiple_of(2) {
                "camp_fire_0.px"
            } else {
                "camp_fire_1.px"
            };
            art::render_at(
                art::embedded(fire).expect("camp fire"),
                frame.buffer_mut(),
                scene,
                i32::from(x + 43),
                i32::from(area.y + 6),
                mode,
            );
            // A load marker depicts actual cargo, rather than inventing a wagon health stat.
            let crates = (self.game.wagon_weight() / 500).min(4);
            for n in 0..crates {
                art::render_at(
                    art::embedded("camp_crate.px").expect("cargo art"),
                    frame.buffer_mut(),
                    scene,
                    i32::from(x + 7 + n as u16 * 7),
                    i32::from(area.y + 5),
                    mode,
                );
            }
        }
        let y = if self.settings.no_art { area.y + 1 } else { scene.bottom() };
        let spare_count =
            ["wheel", "axle", "tongue"].iter().map(|id| self.game.inventory.get(id)).sum::<u32>();
        let lines = vec![
            Line::from("CAMP  ·  Opening camp costs no time."),
            Line::from(format!(
                "A fire beside the parked wagon. Weather: {:?}.",
                self.game.weather
            )),
            Line::from(format!(
                "Wagon load: {} lb · {} · {spare_count} spares",
                self.game.wagon_weight(),
                if self.game.can_repair() { "repair needed" } else { "no repair pending" }
            )),
            Line::from(format!(
                "Oxen: {} yoke · Fatigue {}% · Visiting costs no food.",
                self.game.inventory.get("oxen"),
                self.game.ox_fatigue
            )),
            Line::from(self.trail_advice()),
        ];
        frame.render_widget(Paragraph::new(lines), Rect::new(x + 1, y, 78, 5));
        for (i, label) in [
            "Rest",
            "Treat",
            "Supplies",
            "Forage",
            "Fish",
            "Hunt",
            "Party",
            "Talk",
            "Return to trail",
        ]
        .iter()
        .enumerate()
        {
            frame.render_widget(
                Paragraph::new(format!("{} {}. {label}", marker(i == self.cursor), i + 1)),
                Rect::new(x + (i % 3) as u16 * 26, y + 5 + (i / 3) as u16, 26, 1),
            );
        }
        frame.render_widget(
            Paragraph::new("1-9 select · arrows move · Enter choose · Esc trail"),
            Rect::new(x + 1, y + 8, 78, 1),
        );
        if let Some(last) = self.log.last() {
            frame.render_widget(
                Paragraph::new(last.clone()).wrap(ratatui::widgets::Wrap { trim: true }),
                Rect::new(x + 1, y + 9, 78, 3),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn camp() -> App {
        let mut app = App::new(pioneer_data::load().unwrap(), 23, Settings::default());
        app.apply(Command::Configure {
            trail_id: "oregon".into(),
            era_id: "1848".into(),
            occupation_id: "banker".into(),
            party: ["Ada", "Ben", "Clara", "Dan", "Eve"].map(str::to_owned).to_vec(),
            departure_month: 3,
        });
        for (id, quantity) in [("oxen", 3), ("food", 500)] {
            app.apply(Command::Buy { item_id: id.into(), quantity });
        }
        app.apply(Command::Depart);
        app.handle_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE));
        app
    }

    #[test]
    fn browsing_camp_costs_nothing_and_subscreens_return_to_camp() {
        let mut app = camp();
        let before = serde_json::to_value(&app.game).unwrap();
        assert_eq!(app.screen, Screen::Camp);
        for digit in ['1', '2', '3', '7', '8'] {
            app.handle_key(KeyEvent::new(KeyCode::Char(digit), KeyModifiers::NONE));
            app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
            assert_eq!(app.screen, Screen::Camp);
        }
        app.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(app.screen, Screen::Journey);
        assert_eq!(serde_json::to_value(&app.game).unwrap(), before);
    }

    #[test]
    fn resting_from_camp_uses_the_existing_simulation_cost() {
        let mut app = camp();
        let mut expected = app.game.clone();
        expected.apply(Command::Rest { days: 1 });
        app.handle_key(KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE));
        app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(
            serde_json::to_value(&app.game).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
        assert_eq!(app.screen, Screen::Camp);
    }

    #[test]
    fn camp_controls_fit_both_sizes_and_text_mode() {
        for (w, h) in [(80, 24), (120, 40)] {
            for no_art in [false, true] {
                let mut app = camp();
                app.settings.no_art = no_art;
                let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
                term.draw(|frame| app.render(frame)).unwrap();
                let text =
                    term.backend().buffer().content.iter().map(|c| c.symbol()).collect::<String>();
                for label in [
                    "Opening camp costs no time",
                    "Wagon load",
                    "Oxen: 3 yoke",
                    "no repair pending",
                    "Rest",
                    "Treat",
                    "Forage",
                    "Fish",
                    "Hunt",
                    "Return to trail",
                    "Esc trail",
                ] {
                    assert!(text.contains(label), "missing {label}");
                }
            }
        }
    }
}
