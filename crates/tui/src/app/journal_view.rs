use super::*;
use pioneer_sim::JournalKind;

impl App {
    pub(super) fn journal_entries(&self) -> &[pioneer_sim::JournalEntry] {
        self.journal_history
            .as_ref()
            .map_or(self.game.journal.entries.as_slice(), |run| run.journal.as_slice())
    }

    pub(super) fn render_journal(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black).fg(Color::White)),
            area,
        );
        let x = area.x + (area.width - 80) / 2;
        let y = area.y;
        let entries = self.journal_entries();
        let ended = self.journal_history.is_some()
            || matches!(
                self.game.status,
                pioneer_sim::RunStatus::Arrived | pioneer_sim::RunStatus::Failed
            );
        frame.render_widget(
            Paragraph::new(if ended {
                "PARTY JOURNAL · JOURNEY EPILOGUE"
            } else {
                "PARTY JOURNAL"
            }),
            Rect::new(x + 1, y, 78, 1),
        );
        let (leader, survivors, days, miles, arrived) = if let Some(run) = &self.journal_history {
            (run.leader.as_str(), run.survivors, run.days, run.miles, run.arrived)
        } else {
            (
                self.game.party.first().map_or("Your party", |p| p.name.as_str()),
                self.game.party.iter().filter(|p| p.alive).count(),
                self.game.day,
                self.game.miles,
                self.game.status == pioneer_sim::RunStatus::Arrived,
            )
        };
        let summary = if ended {
            format!(
                "{leader}'s party {} after {days} days and {miles} miles. {survivors} survived.",
                if arrived { "reached its destination" } else { "ended its journey" }
            )
        } else {
            format!("{leader}'s party · day {days} · {miles} miles · {survivors} living travelers")
        };
        frame.render_widget(
            Paragraph::new(summary).wrap(ratatui::widgets::Wrap { trim: true }),
            Rect::new(x + 1, y + 2, 78, 2),
        );
        let count = |f: fn(&JournalKind) -> bool| entries.iter().filter(|e| f(&e.kind)).count();
        frame.render_widget(
            Paragraph::new(format!(
                "Recorded: {} stops · {} recoveries · {} losses",
                count(|k| matches!(k, JournalKind::Landmark { .. })),
                count(|k| matches!(k, JournalKind::Recovered { .. })),
                count(|k| matches!(k, JournalKind::Death { .. }))
            )),
            Rect::new(x + 1, y + 5, 78, 1),
        );
        if let Some(entry) = entries.get(self.cursor.min(entries.len().saturating_sub(1))) {
            frame.render_widget(
                Paragraph::new(format!(
                    "Entry {} of {} · Day {} · Mile {}",
                    self.cursor + 1,
                    entries.len(),
                    entry.day,
                    entry.miles
                )),
                Rect::new(x + 1, y + 7, 78, 1),
            );
            let memorial = matches!(entry.kind, JournalKind::Death { .. });
            if memorial {
                frame.render_widget(Paragraph::new("IN MEMORY"), Rect::new(x + 1, y + 9, 66, 1));
                if !self.settings.no_art {
                    crate::art::render(
                        crate::art::embedded("tombstone.px").expect("memorial art"),
                        frame.buffer_mut(),
                        Rect::new(x + 70, y + 9, 8, 5),
                        self.color_mode(),
                    );
                }
            }
            frame.render_widget(
                Paragraph::new(entry.kind.text()).wrap(ratatui::widgets::Wrap { trim: true }),
                Rect::new(
                    x + 1,
                    y + 10,
                    if memorial && !self.settings.no_art { 66 } else { 78 },
                    area.height.saturating_sub(14),
                ),
            );
        } else {
            frame.render_widget(
                Paragraph::new(if self.journal_history.is_some() {
                    "Journal unavailable for this legacy journey."
                } else {
                    "No notable moments have been recorded yet. Earlier history may be unavailable."
                })
                .wrap(ratatui::widgets::Wrap { trim: true }),
                Rect::new(x + 1, y + 8, 78, 3),
            );
        }
        frame.render_widget(
            Paragraph::new("Up/Down or k/j browse entries · Esc returns · Ctrl-Q quits"),
            Rect::new(x + 1, area.bottom() - 2, 78, 1),
        );
    }
}
