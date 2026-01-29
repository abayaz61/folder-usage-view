use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::util::format::truncate_str;

pub struct ErrorWidget<'a> {
    message: &'a str,
}

impl<'a> ErrorWidget<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }
}

impl Widget for ErrorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" ⚠ ERROR ")
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
                Span::styled("  Message: ", dim_style),
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
                "  The application encountered an error but will",
                dim_style,
            )),
            Line::from(Span::styled(
                "  continue running. Your data is safe.",
                dim_style,
            )),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Press ", dim_style),
                Span::styled("any key", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" to continue...", dim_style),
            ]),
        ];

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        paragraph.render(inner, buf);
    }
}
