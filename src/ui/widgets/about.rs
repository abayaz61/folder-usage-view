use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::ui::theme::{ColorPalette, Theme};
use crate::util::i18n::{Language, Strings};

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = "Disk Usage Analyzer";
pub const APP_AUTHOR: &str = "Codegen";
pub const APP_EMAIL: &str = "abayaz61@gmail.com";

pub struct AboutWidget {
    lang: Language,
    palette: ColorPalette,
}

impl AboutWidget {
    pub fn new(lang: Language, palette: ColorPalette) -> Self {
        Self { lang, palette }
    }
}

impl Widget for AboutWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let s = Strings::new(self.lang);
        let theme = Theme::new(self.palette);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", s.get("about.title")))
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(theme.highlight_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        let title_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
        let label_style = Style::default().fg(Color::Yellow);
        let value_style = Style::default().fg(Color::White);
        let dim_style = Style::default().fg(Color::DarkGray);

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "╔═══════════════════════════════════════╗",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(Span::styled(
                "║       DISK USAGE ANALYZER             ║",
                title_style,
            )),
            Line::from(Span::styled(
                "╚═══════════════════════════════════════╝",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Version:     ", label_style),
                Span::styled(format!("v{}", APP_VERSION), value_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Author:      ", label_style),
                Span::styled(APP_AUTHOR, value_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Email:       ", label_style),
                Span::styled(APP_EMAIL, Style::default().fg(Color::Blue)),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "  ─────────────────────────────────────",
                dim_style,
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", dim_style),
                Span::styled(s.get("about.description").to_string(), dim_style),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Built with: ", dim_style),
                Span::styled("Rust + Ratatui", Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", s.get("about.close")),
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}
