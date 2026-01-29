use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::ui::theme::Theme;

pub struct HelpWidget;

impl HelpWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HelpWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Help - Keyboard Shortcuts ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Theme::highlight_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        let key_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::White);
        let section_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled("Navigation", section_style)),
            Line::from(vec![
                Span::styled("  ↑/k      ", key_style),
                Span::styled("Move selection up", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  ↓/j      ", key_style),
                Span::styled("Move selection down", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Enter/→/l", key_style),
                Span::styled("Enter directory", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Backspace", key_style),
                Span::styled("Go to parent / Computer view", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  PgUp/PgDn", key_style),
                Span::styled("Move 10 items up/down", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Home/End ", key_style),
                Span::styled("Jump to first/last item", desc_style),
            ]),
            Line::from(""),
            Line::from(Span::styled("Views", section_style)),
            Line::from(vec![
                Span::styled("  Tab      ", key_style),
                Span::styled("Toggle view mode (Treemap/List/Split)", desc_style),
            ]),
            Line::from(""),
            Line::from(Span::styled("Selection & Deletion", section_style)),
            Line::from(vec![
                Span::styled("  Space    ", key_style),
                Span::styled("Toggle selection for deletion", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  d        ", key_style),
                Span::styled("Delete selected items (if not read-only)", desc_style),
            ]),
            Line::from(""),
            Line::from(Span::styled("Drives", section_style)),
            Line::from(vec![
                Span::styled("  g        ", key_style),
                Span::styled("Open drive selector (with usage info)", desc_style),
            ]),
            Line::from(""),
            Line::from(Span::styled("General", section_style)),
            Line::from(vec![
                Span::styled("  ?/h      ", key_style),
                Span::styled("Toggle this help", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  a        ", key_style),
                Span::styled("About", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  s        ", key_style),
                Span::styled("Settings", desc_style),
            ]),
            Line::from(vec![
                Span::styled("  q/Esc    ", key_style),
                Span::styled("Quit application", desc_style),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key to close this help",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}
