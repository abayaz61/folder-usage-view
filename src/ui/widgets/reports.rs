use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::app::App;
use crate::util::i18n::Strings;

pub struct ReportsWidget<'a> {
    app: &'a App,
}

impl<'a> ReportsWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl Widget for ReportsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let s = Strings::new(self.app.settings.language);
        let theme = self.app.theme();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", s.get("reports.title")))
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(theme.highlight_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        let options = [
            ("x", s.get("reports.snapshot"), s.get("reports.snapshot_desc")),
            ("f", s.get("reports.cleanup"), s.get("reports.cleanup_desc")),
            ("u", s.get("reports.duplicates"), s.get("reports.duplicates_desc")),
        ];

        let mut lines = vec![Line::from(""), Line::from(Span::styled(
            s.get("reports.subtitle").to_string(),
            Style::default().fg(Color::DarkGray),
        )), Line::from("")];

        for (index, (key, label, desc)) in options.iter().enumerate() {
            let selected = self.app.reports_selected_index == index;
            let marker = if selected { "▶" } else { " " };
            let row_style = if selected {
                Style::default()
                    .bg(theme.selected_bg())
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", marker), row_style),
                Span::styled(format!("[{}] ", key), Style::default().fg(Color::Yellow)),
                Span::styled(*label, row_style),
            ]));
            lines.push(Line::from(vec![
                Span::raw("      "),
                Span::styled(*desc, Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(""));
        }

        lines.push(Line::from(Span::styled(
            s.get("reports.close").to_string(),
            Style::default().fg(Color::DarkGray),
        )));

        Paragraph::new(lines).wrap(Wrap { trim: false }).render(inner, buf);
    }
}
