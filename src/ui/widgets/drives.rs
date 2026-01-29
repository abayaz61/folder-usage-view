use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::app::App;
use crate::ui::theme::Theme;
use crate::util::format::format_size;

pub struct DriveListWidget<'a> {
    app: &'a App,
}

impl<'a> DriveListWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl Widget for DriveListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Select Drive - Press Enter to confirm, Esc to cancel ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Theme::highlight_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        if self.app.drives.is_empty() {
            let msg = "No drives found";
            buf.set_string(
                inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2,
                inner.y + inner.height / 2,
                msg,
                Style::default().fg(Color::DarkGray),
            );
            return;
        }

        let mut lines = Vec::new();
        lines.push(Line::from(""));

        for (i, drive) in self.app.drives.iter().enumerate() {
            let is_selected = i == self.app.drive_selected_index;
            let is_current = self.app.config.target_path.starts_with(&drive.mount_point);

            // Calculate usage bar
            let bar_width = 20;
            let usage_pct = drive.usage_percentage();
            let filled = ((usage_pct / 100.0) * bar_width as f64) as usize;

            // Usage bar color based on percentage
            let bar_color = if usage_pct > 90.0 {
                Color::Red
            } else if usage_pct > 75.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            // Build usage bar string
            let bar: String = (0..bar_width)
                .map(|j| if j < filled { '█' } else { '░' })
                .collect();

            // Drive icon
            let icon = if drive.is_removable { "💾" } else { "💿" };

            // Format sizes
            let used_str = format_size(drive.used_space);
            let total_str = format_size(drive.total_space);
            let free_str = format_size(drive.available_space);

            // Build the line
            let marker = if is_current { "●" } else { " " };

            let style = if is_selected {
                Style::default()
                    .bg(Theme::selected_bg())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let marker_style = if is_current {
                style.fg(Color::Green)
            } else {
                style.fg(Color::DarkGray)
            };

            // Line 1: Drive name and mount point
            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", marker), marker_style),
                Span::styled(format!("{} ", icon), style),
                Span::styled(
                    format!("{:<30}", drive.display_name()),
                    if is_selected {
                        style.fg(Color::White)
                    } else {
                        style.fg(Color::Cyan)
                    },
                ),
                Span::styled(
                    format!("[{}]", drive.file_system),
                    style.fg(Color::DarkGray),
                ),
            ]));

            // Line 2: Usage bar and stats
            lines.push(Line::from(vec![
                Span::styled("     ", style),
                Span::styled(bar, style.fg(bar_color)),
                Span::styled(
                    format!(" {:5.1}%", usage_pct),
                    style.fg(Color::Yellow),
                ),
                Span::styled(
                    format!("  {} / {} (Free: {})", used_str, total_str, free_str),
                    style.fg(Color::White),
                ),
            ]));

            lines.push(Line::from(""));
        }

        // Footer hint
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "  ↑/↓: Navigate   Enter: Select   Esc: Cancel   g: Refresh",
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}
