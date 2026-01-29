use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::app::App;
use crate::util::format::{format_size, truncate_str};
use crate::util::i18n::Strings;

pub struct ComputerViewWidget<'a> {
    app: &'a App,
}

impl<'a> ComputerViewWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }

    fn render_drive_card(&self, drive_idx: usize, area: Rect, buf: &mut Buffer) {
        if drive_idx >= self.app.drives.len() {
            return;
        }

        let drive = &self.app.drives[drive_idx];
        let is_selected = drive_idx == self.app.drive_selected_index;
        let is_current = self.app.config.target_path.starts_with(&drive.mount_point);
        let theme = self.app.theme();
        let icons = self.app.icons();

        // Card border style
        let border_color = if is_selected {
            theme.highlight_color()
        } else if is_current {
            Color::Green
        } else {
            theme.border_color()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(if is_selected {
                Style::default().bg(theme.selected_bg())
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 3 || inner.width < 10 {
            return;
        }

        // Drive icon and name
        let icon = icons.drive(drive.is_removable);
        let name_style = Style::default()
            .fg(if is_selected { Color::White } else { Color::Cyan })
            .add_modifier(Modifier::BOLD);

        // Truncate name to fit
        let max_name_len = inner.width.saturating_sub(8) as usize;
        let display_name = drive.display_name();
        let truncated_name = truncate_str(&display_name, max_name_len);
        let name_line = format!("{} {}", icon, truncated_name);
        buf.set_string(inner.x + 1, inner.y, &name_line, name_style);

        // File system type (only if space available)
        if inner.width > 20 {
            let fs_str = format!("[{}]", drive.file_system);
            let fs_len = fs_str.chars().count() as u16;
            if inner.width > fs_len + 2 {
                buf.set_string(
                    inner.x + inner.width.saturating_sub(fs_len + 1),
                    inner.y,
                    &fs_str,
                    Style::default().fg(Color::DarkGray),
                );
            }
        }

        // Usage percentage and bar
        if inner.height >= 4 {
            let usage_pct = drive.usage_percentage();
            let bar_width = inner.width.saturating_sub(10) as usize;
            let filled = ((usage_pct / 100.0) * bar_width as f64) as usize;

            let bar_color = if usage_pct > 90.0 {
                Color::Red
            } else if usage_pct > 75.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            // Usage bar
            let bar: String = (0..bar_width)
                .map(|j| if j < filled { '█' } else { '░' })
                .collect();

            buf.set_string(inner.x + 1, inner.y + 2, &bar, Style::default().fg(bar_color));

            // Percentage
            let pct_str = format!("{:5.1}%", usage_pct);
            buf.set_string(
                inner.x + inner.width.saturating_sub(pct_str.len() as u16 + 1),
                inner.y + 2,
                &pct_str,
                Style::default().fg(Color::Yellow),
            );
        }

        // Size info
        if inner.height >= 5 {
            let used_str = format_size(drive.used_space);
            let total_str = format_size(drive.total_space);
            let size_info = format!("{} / {}", used_str, total_str);
            buf.set_string(
                inner.x + 1,
                inner.y + 3,
                &size_info,
                Style::default().fg(Color::White),
            );
        }

        // Free space
        if inner.height >= 6 {
            let free_str = format!("Free: {}", format_size(drive.available_space));
            buf.set_string(
                inner.x + 1,
                inner.y + 4,
                &free_str,
                Style::default().fg(Color::DarkGray),
            );
        }

        // Current marker
        if is_current {
            buf.set_string(
                inner.x + inner.width.saturating_sub(3),
                inner.y + inner.height.saturating_sub(1),
                "●",
                Style::default().fg(Color::Green),
            );
        }
    }

    fn render_total_summary(&self, area: Rect, buf: &mut Buffer) {
        let lang = self.app.settings.language;
        let s = Strings::new(lang);
        let theme = self.app.theme();
        let (total, used, free) = self.app.get_total_disk_stats();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", s.get("computer.total_usage")))
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(theme.border_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.height < 2 || inner.width < 20 {
            return;
        }

        // Calculate total usage
        let usage_pct = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let bar_width = inner.width.saturating_sub(12) as usize;
        let filled = ((usage_pct / 100.0) * bar_width as f64) as usize;

        let bar_color = if usage_pct > 90.0 {
            Color::Red
        } else if usage_pct > 75.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        // Build lines
        let mut lines = Vec::new();

        // Summary stats
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", s.get("drive.total")), Style::default().fg(Color::DarkGray)),
            Span::styled(format_size(total), Style::default().fg(Color::White)),
            Span::styled(format!("    {} ", s.get("drive.used")), Style::default().fg(Color::DarkGray)),
            Span::styled(format_size(used), Style::default().fg(bar_color)),
            Span::styled(format!("    {} ", s.get("drive.free")), Style::default().fg(Color::DarkGray)),
            Span::styled(format_size(free), Style::default().fg(Color::Green)),
        ]));

        // Usage bar
        let bar: String = (0..bar_width)
            .map(|j| if j < filled { '█' } else { '░' })
            .collect();

        lines.push(Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(bar, Style::default().fg(bar_color)),
            Span::styled(format!(" {:5.1}%", usage_pct), Style::default().fg(Color::Yellow)),
        ]));

        // Drive count
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} {}", self.app.drives.len(), s.get("computer.drives_detected")),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}

impl Widget for ComputerViewWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lang = self.app.settings.language;
        let s = Strings::new(lang);

        // Main layout: title, drive grid, total summary
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(10),    // Drive grid
                Constraint::Length(6),  // Total summary
            ])
            .split(area);

        // Title
        let theme = self.app.theme();
        let title_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", s.get("computer.title")))
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(theme.highlight_color()));

        let title_inner = title_block.inner(main_layout[0]);
        title_block.render(main_layout[0], buf);

        let hint = s.get("computer.hint");
        let max_hint_len = title_inner.width.saturating_sub(2) as usize;
        let truncated_hint = truncate_str(&hint, max_hint_len);
        buf.set_string(
            title_inner.x + 1,
            title_inner.y,
            &truncated_hint,
            Style::default().fg(Color::DarkGray),
        );

        // Drive grid
        let drive_count = self.app.drives.len();
        if drive_count == 0 {
            let msg = s.get("computer.no_drives");
            buf.set_string(
                main_layout[1].x + (main_layout[1].width.saturating_sub(msg.len() as u16)) / 2,
                main_layout[1].y + main_layout[1].height / 2,
                &msg,
                Style::default().fg(Color::DarkGray),
            );
        } else {
            // Calculate grid layout (2-3 columns depending on width)
            let cols = if area.width > 120 { 3 } else if area.width > 80 { 2 } else { 1 };
            let rows = (drive_count + cols - 1) / cols;

            let card_height = 7u16; // Height per drive card
            let available_height = main_layout[1].height;
            let actual_rows = std::cmp::min(rows, (available_height / card_height) as usize);

            // Create column constraints
            let col_constraints: Vec<Constraint> = (0..cols)
                .map(|_| Constraint::Ratio(1, cols as u32))
                .collect();

            let grid_area = main_layout[1];

            for row in 0..actual_rows {
                let row_y = grid_area.y + (row as u16 * card_height);
                if row_y + card_height > grid_area.y + grid_area.height {
                    break;
                }

                let row_area = Rect::new(
                    grid_area.x,
                    row_y,
                    grid_area.width,
                    card_height,
                );

                let col_areas = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(col_constraints.clone())
                    .split(row_area);

                for col in 0..cols {
                    let drive_idx = row * cols + col;
                    if drive_idx < drive_count {
                        self.render_drive_card(drive_idx, col_areas[col], buf);
                    }
                }
            }
        }

        // Total summary
        self.render_total_summary(main_layout[2], buf);
    }
}
