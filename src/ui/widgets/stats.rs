use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::app::App;
use crate::util::format::{format_count, format_size};
use crate::util::i18n::Strings;

pub struct StatsWidget<'a> {
    app: &'a App,
}

impl<'a> StatsWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl Widget for StatsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let s = Strings::new(self.app.settings.language);
        let theme = self.app.theme();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", s.get("stats.title")))
            .border_style(Style::default().fg(theme.border_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 10 || inner.height < 5 {
            return;
        }

        // Split into sections
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),  // Summary
                Constraint::Length(1),  // Separator
                Constraint::Min(5),     // Category breakdown
                Constraint::Length(1),  // Separator
                Constraint::Min(5),     // Largest files
            ])
            .split(inner);

        // Summary section
        render_summary(self.app, buf, layout[0]);

        // Separator
        render_separator(buf, layout[1], s.get("stats.file_types"));

        // Category breakdown
        render_categories(self.app, buf, layout[2]);

        // Separator
        render_separator(buf, layout[3], s.get("stats.largest_files"));

        // Largest files
        render_largest_files(self.app, buf, layout[4]);
    }
}

fn render_summary(app: &App, buf: &mut Buffer, area: Rect) {
    let s = Strings::new(app.settings.language);
    let stats = &app.tree.statistics;

    let lines = vec![
        Line::from(vec![
            Span::raw(format!("{} ", s.get("stats.total_size"))),
            Span::styled(
                format_size(stats.total_size),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw(format!("{} ", s.get("stats.files"))),
            Span::styled(
                format_count(stats.total_files),
                Style::default().fg(Color::Green),
            ),
            Span::raw(format!("  {} ", s.get("stats.directories"))),
            Span::styled(
                format_count(stats.total_dirs),
                Style::default().fg(Color::Blue),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    paragraph.render(area, buf);
}

fn render_separator(buf: &mut Buffer, area: Rect, title: &str) {
    if area.width < 5 {
        return;
    }

    let title_style = Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD);

    // Draw line with title
    let max_len = area.width as usize - 4;
    let title_display: String = title.chars().take(max_len).collect();
    let title_len = title_display.chars().count();
    let title_start = area.x + 1;
    let line_end = area.x + area.width;

    buf.set_string(title_start, area.y, &title_display, title_style);

    // Draw dashes after title
    for x in (title_start + title_len as u16 + 1)..line_end {
        if let Some(cell) = buf.cell_mut((x, area.y)) {
            cell.set_char('─')
                .set_style(Style::default().fg(Color::DarkGray));
        }
    }
}

fn render_categories(app: &App, buf: &mut Buffer, area: Rect) {
    let categories = app.tree.statistics.get_category_percentages();
    let theme = app.theme();

    if categories.is_empty() {
        buf.set_string(
            area.x,
            area.y,
            "No data",
            Style::default().fg(Color::DarkGray),
        );
        return;
    }

    let bar_width = area.width.saturating_sub(25) as usize;

    for (i, (category, percentage, _size)) in categories.iter().enumerate().take(area.height as usize) {
        let y = area.y + i as u16;
        if y >= area.bottom() {
            break;
        }

        let color = theme.color_for_category(category);
        let name = category.name();

        // Category name
        let name_style = Style::default().fg(color);
        buf.set_string(area.x, y, &format!("{:<10}", name), name_style);

        // Bar
        let bar_x = area.x + 11;
        let filled = ((percentage / 100.0) * bar_width as f64) as usize;

        for j in 0..bar_width {
            let x = bar_x + j as u16;
            if x >= area.right() {
                break;
            }

            if let Some(cell) = buf.cell_mut((x, y)) {
                if j < filled {
                    cell.set_char('█').set_fg(color);
                } else {
                    cell.set_char('░').set_fg(Color::DarkGray);
                }
            }
        }

        // Percentage and size
        let stats_x = bar_x + bar_width as u16 + 1;
        if stats_x + 12 <= area.right() {
            buf.set_string(
                stats_x,
                y,
                &format!("{:5.1}%", percentage),
                Style::default().fg(Color::Yellow),
            );
        }
    }
}

fn render_largest_files(app: &App, buf: &mut Buffer, area: Rect) {
    let largest = &app.tree.statistics.largest_files;

    if largest.is_empty() {
        buf.set_string(
            area.x,
            area.y,
            "No files",
            Style::default().fg(Color::DarkGray),
        );
        return;
    }

    let max_items = area.height as usize;
    let name_width = area.width.saturating_sub(12) as usize;

    for (i, (size, _node_id, name)) in largest.iter().enumerate().take(max_items) {
        let y = area.y + i as u16;
        if y >= area.bottom() {
            break;
        }

        // Truncate name (character-safe)
        let char_count = name.chars().count();
        let display_name = if char_count > name_width {
            let truncated: String = name.chars().take(name_width.saturating_sub(3)).collect();
            format!("{}...", truncated)
        } else {
            name.clone()
        };

        // Render name
        buf.set_string(
            area.x,
            y,
            &display_name,
            Style::default().fg(Color::White),
        );

        // Render size (right-aligned)
        let size_str = format_size(*size);
        let size_x = area.right().saturating_sub(size_str.len() as u16 + 1);
        buf.set_string(
            size_x,
            y,
            &size_str,
            Style::default().fg(Color::Cyan),
        );
    }
}
