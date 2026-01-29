use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::util::format::truncate_str;
use crate::util::i18n::{Language, Strings};

pub struct ErrorWidget<'a> {
    message: &'a str,
    lang: Language,
}

impl<'a> ErrorWidget<'a> {
    pub fn new(message: &'a str, lang: Language) -> Self {
        Self { message, lang }
    }
}

impl Widget for ErrorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let s = Strings::new(self.lang);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" ⚠ {} ", s.get("error.title")))
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(Color::Red));

        let inner = block.inner(area);
        block.render(area, buf);

        let error_style = Style::default().fg(Color::Red);
        let dim_style = Style::default().fg(Color::DarkGray);
        let highlight_style = Style::default().fg(Color::Yellow);

        // Retro ASCII art error box
        let max_msg_width = inner.width.saturating_sub(4) as usize;
        let truncated_msg = truncate_str(self.message, max_msg_width);

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  ╔══════════════════════════════════════════╗",
                error_style,
            )),
            Line::from(Span::styled(
                "  ║  ███████╗██████╗ ██████╗  ██████╗ ██████╗║",
                error_style,
            )),
            Line::from(Span::styled(
                "  ║  ██╔════╝██╔══██╗██╔══██╗██╔═══██╗██╔══██╗",
                error_style,
            )),
            Line::from(Span::styled(
                "  ║  █████╗  ██████╔╝██████╔╝██║   ██║██████╔╝",
                error_style,
            )),
            Line::from(Span::styled(
                "  ║  ██╔══╝  ██╔══██╗██╔══██╗██║   ██║██╔══██╗",
                error_style,
            )),
            Line::from(Span::styled(
                "  ║  ███████╗██║  ██║██║  ██║╚██████╔╝██║  ██║",
                error_style,
            )),
            Line::from(Span::styled(
                "  ║  ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝║",
                error_style,
            )),
            Line::from(Span::styled(
                "  ╚══════════════════════════════════════════╝",
                error_style,
            )),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("  {} ", s.get("error.message")), dim_style),
                Span::styled(truncated_msg, highlight_style),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "  ────────────────────────────────────────────",
                dim_style,
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", s.get("error.occurred")),
                dim_style,
            )),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                format!("  {}", s.get("error.continue")),
                Style::default().fg(Color::Cyan),
            )),
        ];

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}
