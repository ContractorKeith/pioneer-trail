use super::*;

const MAX_DIAGRAM_STOPS: usize = 24;

impl App {
    pub(super) fn render_route_map(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black).fg(Color::White)),
            area,
        );
        let x = area.x + (area.width - 80) / 2;
        let Some(trail) =
            self.game.content.trails.iter().find(|t| Some(&t.id) == self.game.trail_id.as_ref())
        else {
            return;
        };
        let mode = self.color_mode();
        let mut y = area.y;
        frame.render_widget(
            Paragraph::new(format!(
                "{} ROUTE RECORD · {} miles · {:?}",
                trail.name.to_uppercase(),
                self.game.miles,
                self.game.weather
            )),
            Rect::new(x, y, 80, 1),
        );
        y += 1;
        let diagram = !self.settings.no_art && trail.nodes.len() <= MAX_DIAGRAM_STOPS;
        if diagram {
            let position = |index: usize| {
                let row = index / 8;
                let column = if row.is_multiple_of(2) { index % 8 } else { 7 - index % 8 };
                (x + 3 + column as u16 * 10, y + 1 + row as u16 * 2)
            };
            // A schematic of the actual trail graph, not a claim about geographic coordinates.
            for (index, node) in trail.nodes.iter().enumerate() {
                let (ax, ay) = position(index);
                for route in &node.routes {
                    let Some(target) = trail.nodes.iter().position(|n| n.id == route.target_id)
                    else {
                        continue;
                    };
                    let (bx, by) = position(target);
                    let traveled = self.game.visited_landmarks.windows(2).any(|v| {
                        v[0].landmark_id == node.id && v[1].landmark_id == route.target_id
                    });
                    let color = mode.color(if traveled {
                        crate::art::Pixel::Green
                    } else {
                        crate::art::Pixel::White
                    });
                    for dx in ax.min(bx)..=ax.max(bx) {
                        frame.buffer_mut()[(dx, ay)].set_symbol("─").set_fg(color);
                    }
                    for dy in ay.min(by)..=ay.max(by) {
                        frame.buffer_mut()[(bx, dy)].set_symbol("│").set_fg(color);
                    }
                }
            }
            for (index, node) in trail.nodes.iter().enumerate() {
                let (px, py) = position(index);
                let stamp = self.map_stamp(&node.id);
                let code = stop_label(index);
                let color = mode.color(if stamp == '@' {
                    crate::art::Pixel::Orange
                } else if matches!(stamp, '*' | '>') {
                    crate::art::Pixel::Green
                } else {
                    crate::art::Pixel::White
                });
                frame.render_widget(
                    Paragraph::new(format!("{stamp}{code}"))
                        .style(Style::default().fg(color).bg(Color::Black)),
                    Rect::new(px, py, 2, 1),
                );
            }
            if let Some((start, target, distance)) =
                trail.nodes.iter().enumerate().find_map(|(i, n)| {
                    if Some(&n.id) != self.game.current_node_id.as_ref() {
                        return None;
                    }
                    let route = n
                        .routes
                        .iter()
                        .find(|r| Some(&r.target_id) == self.game.target_node_id.as_ref())?;
                    let target = trail.nodes.iter().position(|n| n.id == route.target_id)?;
                    Some((i, target, route.distance_miles))
                })
            {
                let (ax, ay) = position(start);
                let (bx, by) = position(target);
                let horizontal = ax.abs_diff(bx);
                let total = u32::from(horizontal + ay.abs_diff(by));
                let traveled = distance.saturating_sub(self.game.route_miles_remaining);
                let offset =
                    traveled.saturating_mul(total).checked_div(distance).unwrap_or(0) as u16;
                let px = if offset <= horizontal {
                    if bx >= ax {
                        ax + offset
                    } else {
                        ax - offset
                    }
                } else {
                    bx
                };
                let py = if offset <= horizontal {
                    ay
                } else if by >= ay {
                    ay + offset - horizontal
                } else {
                    ay - (offset - horizontal)
                };
                frame.buffer_mut()[(px, py)]
                    .set_symbol("◆")
                    .set_fg(mode.color(crate::art::Pixel::Orange));
            }
            for grave in self.graves.iter().filter(|g| g.mile <= self.game.miles) {
                if let Some(visit) =
                    self.game.visited_landmarks.iter().min_by_key(|v| v.mile.abs_diff(grave.mile))
                {
                    if let Some(index) = trail.nodes.iter().position(|n| n.id == visit.landmark_id)
                    {
                        let (px, py) = position(index);
                        frame.buffer_mut()[(px + 2, py + 1)]
                            .set_symbol("†")
                            .set_fg(mode.color(crate::art::Pixel::White));
                    }
                }
            }
            y += (trail.nodes.len().div_ceil(8) * 2 + 1) as u16;
        }
        if !self.settings.no_art && !diagram {
            frame.render_widget(
                Paragraph::new(format!(
                    "ROUTE MAP LISTING · {} stops (diagram available for 24 or fewer stops)",
                    trail.nodes.len()
                )),
                Rect::new(x, y, 80, 1),
            );
            y += 1;
        }
        frame.render_widget(
            Paragraph::new(if diagram {
                "◆ wagon · @ current stop · > next · * reached · o unknown · † grave"
            } else {
                "@ current stop · > next · * reached · o unknown · † grave"
            }),
            Rect::new(x, y, 80, 1),
        );
        y += 1;
        let leg = self
            .game
            .target_node_id
            .as_ref()
            .and_then(|id| trail.nodes.iter().find(|n| &n.id == id))
            .map_or_else(
                || "No active leg. Choose a route or depart at the stop.".into(),
                |n| {
                    format!(
                        "On the way to {}: {} miles remain.",
                        n.name, self.game.route_miles_remaining
                    )
                },
            );
        frame.render_widget(Paragraph::new(leg), Rect::new(x, y, 80, 1));
        y += 1;
        let supply = self.game.next_supply_stop().map_or_else(
            || "No further supply stop on a reachable route.".into(),
            |(name, miles)| {
                format!("Nearest reachable supply: {name}, {miles} miles. Forks may vary.")
            },
        );
        frame.render_widget(Paragraph::new(supply), Rect::new(x, y, 80, 1));
        y += 1;
        if self.game.visited_landmarks.is_empty() {
            frame.render_widget(
                Paragraph::new("Earlier visits were not recorded in this save."),
                Rect::new(x, y, 80, 1),
            );
            y += 1;
        }
        let rows = area.bottom().saturating_sub(y + 4) as usize;
        let entries = trail
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let visit = self.game.visited_landmarks.iter().find(|v| v.landmark_id == n.id);
                let detail = visit.map_or_else(
                    || format!("{} route miles", n.mile),
                    |v| format!("day {}, mile {}", v.day, v.mile),
                );
                format!(
                    "{} {}{} {} · {detail}",
                    marker(i == self.cursor),
                    self.map_stamp(&n.id),
                    stop_label(i),
                    n.name
                )
            })
            .chain(self.graves.iter().enumerate().map(|(i, grave)| {
                format!(
                    "{} † Mile {} · {}",
                    marker(i + trail.nodes.len() == self.cursor),
                    grave.mile,
                    grave.leader
                )
            }))
            .skip(self.cursor)
            .take(rows)
            .map(Line::from)
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(entries), Rect::new(x, y, 80, rows as u16));
        if let Some(grave) =
            self.cursor.checked_sub(trail.nodes.len()).and_then(|i| self.graves.get(i))
        {
            frame.render_widget(
                Paragraph::new(format!("{} · {}: {}", grave.date, grave.cause, grave.epitaph))
                    .wrap(ratatui::widgets::Wrap { trim: true }),
                Rect::new(x, area.bottom() - 4, 80, 3),
            );
        }
        frame.render_widget(
            Paragraph::new("↑↓ / jk scroll stops and graves · Esc back · schematic, not to scale"),
            Rect::new(x, area.bottom() - 1, 80, 1),
        );
    }

    fn map_stamp(&self, id: &str) -> char {
        if self.game.current_node_id.as_deref() == Some(id) {
            '@'
        } else if self.game.target_node_id.as_deref() == Some(id) {
            '>'
        } else if self.game.visited_landmarks.iter().any(|v| v.landmark_id == id) {
            '*'
        } else {
            'o'
        }
    }
}

fn stop_label(index: usize) -> String {
    if index < 26 {
        char::from(b'A' + index as u8).to_string()
    } else {
        (index + 1).to_string()
    }
}
