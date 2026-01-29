use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::app::{App, StartupLocation};
use crate::app::settings::windows;
use crate::ui::theme::Theme;

pub struct SettingsWidget<'a> {
    app: &'a App,
}

impl<'a> SettingsWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }

    fn render_option(
        &self,
        index: usize,
        label: &str,
        value: &str,
        description: &str,
        is_enabled: bool,
    ) -> Vec<Line<'static>> {
        let is_selected = self.app.settings_selected_index == index;

        let marker = if is_selected { "▶ " } else { "  " };
        let marker_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let label_style = if is_selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let value_color = if is_enabled { Color::Green } else { Color::Red };
        let value_style = Style::default().fg(value_color).add_modifier(Modifier::BOLD);

        let desc_style = Style::default().fg(Color::DarkGray);

        let bg_style = if is_selected {
            Style::default().bg(Theme::selected_bg())
        } else {
            Style::default()
        };

        vec![
            Line::from(vec![
                Span::styled(format!("  {}", marker), marker_style),
                Span::styled(format!("{:<35}", label), label_style.patch(bg_style)),
                Span::styled(value.to_string(), value_style.patch(bg_style)),
            ]),
            Line::from(vec![
                Span::styled("      ", desc_style),
                Span::styled(description.to_string(), desc_style),
            ]),
            Line::from(""),
        ]
    }
}

impl Widget for SettingsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Settings ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Theme::highlight_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        let header_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
        let dim_style = Style::default().fg(Color::DarkGray);

        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled("  ⚙ Application Settings", header_style)),
            Line::from(""),
            Line::from(Span::styled(
                "  ─────────────────────────────────────────────",
                dim_style,
            )),
            Line::from(""),
        ];

        // Context Menu option
        let context_enabled = windows::is_context_menu_registered();
        let context_value = if context_enabled { "Enabled" } else { "Disabled" };
        lines.extend(self.render_option(
            0,
            "Context Menu Integration",
            context_value,
            "Add 'Usage Analytics' to right-click menu",
            context_enabled,
        ));

        // Startup Location option
        let startup_value = match self.app.settings.startup_location {
            StartupLocation::LastLocation => "Last Location",
            StartupLocation::CurrentFolder => "Current Folder",
            StartupLocation::ComputerView => "Computer View",
        };
        lines.extend(self.render_option(
            1,
            "Startup Location",
            startup_value,
            "Where to start when launching the app",
            true,
        ));

        // PATH Registration option
        let path_enabled = windows::is_path_registered();
        let path_value = if path_enabled { "Registered" } else { "Not Registered" };
        lines.extend(self.render_option(
            2,
            "System PATH Registration",
            path_value,
            "Install to Program Files and add to PATH",
            path_enabled,
        ));

        // Footer
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ─────────────────────────────────────────────",
            dim_style,
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  ↑↓", Style::default().fg(Color::Yellow)),
            Span::styled(": Navigate   ", dim_style),
            Span::styled("Enter/Space", Style::default().fg(Color::Yellow)),
            Span::styled(": Toggle   ", dim_style),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::styled(": Close", dim_style),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Note: Some options require Administrator privileges",
            Style::default().fg(Color::Red),
        )));

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}
