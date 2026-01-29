use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::ui::theme::Theme;
use crate::util::i18n::{Language, Strings};

pub struct HelpWidget {
    lang: Language,
}

impl HelpWidget {
    pub fn new(lang: Language) -> Self {
        Self { lang }
    }
}

impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let s = Strings::new(self.lang);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", s.get("help.title")))
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Theme::highlight_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        let key_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::White);
        let section_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(s.get("help.navigation").to_string(), section_style)),
            Line::from(vec![
                Span::styled("  ↑/k      ", key_style),
                Span::styled(s.get("help.nav_up_down").to_string(), desc_style),
            ]),
            Line::from(vec![
                Span::styled("  ↓/j      ", key_style),
                Span::styled(s.get("help.nav_up_down").to_string(), desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Enter/→/l", key_style),
                Span::styled(s.get("help.nav_into").to_string(), desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Backspace", key_style),
                Span::styled(s.get("help.nav_back").to_string(), desc_style),
            ]),
            Line::from(vec![
                Span::styled("  PgUp/PgDn", key_style),
                Span::styled(s.get("help.nav_page").to_string(), desc_style),
            ]),
            Line::from(vec![
                Span::styled("  Home/End ", key_style),
                Span::styled(s.get("help.nav_home_end").to_string(), desc_style),
            ]),
            Line::from(""),
            Line::from(Span::styled(s.get("help.views").to_string(), section_style)),
            Line::from(vec![
                Span::styled("  Tab      ", key_style),
                Span::styled(s.get("help.view_toggle").to_string(), desc_style),
            ]),
            Line::from(vec![
                Span::styled("  o        ", key_style),
                Span::styled(s.get("help.sort_toggle").to_string(), desc_style),
            ]),
            Line::from(""),
            Line::from(Span::styled(s.get("help.actions").to_string(), section_style)),
            Line::from(vec![
                Span::styled("  Space    ", key_style),
                Span::styled(s.get("help.action_select").to_string(), desc_style),
            ]),
            Line::from(vec![
                Span::styled("  d        ", key_style),
                Span::styled(s.get("help.action_delete").to_string(), desc_style),
            ]),
            Line::from(vec![
                Span::styled("  g        ", key_style),
                Span::styled(s.get("help.action_refresh").to_string(), desc_style),
            ]),
            Line::from(""),
            Line::from(Span::styled(s.get("help.other").to_string(), section_style)),
            Line::from(vec![
                Span::styled("  ?/h      ", key_style),
                Span::styled(s.get("help.other_help").to_string(), desc_style),
            ]),
            Line::from(vec![
                Span::styled("  a        ", key_style),
                Span::styled(s.get("help.other_about").to_string(), desc_style),
            ]),
            Line::from(vec![
                Span::styled("  s        ", key_style),
                Span::styled(s.get("help.other_settings").to_string(), desc_style),
            ]),
            Line::from(vec![
                Span::styled("  q/Esc    ", key_style),
                Span::styled(s.get("help.other_quit").to_string(), desc_style),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                s.get("help.close").to_string(),
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}
