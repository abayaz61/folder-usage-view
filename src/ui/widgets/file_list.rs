use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::app::App;
use crate::ui::theme::Theme;
use crate::util::format::format_size;
use crate::util::i18n::Strings;

pub struct FileListWidget<'a> {
    app: &'a App,
}

impl<'a> FileListWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl Widget for FileListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let s = Strings::new(self.app.settings.language);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", s.get("filelist.title")))
            .border_style(Style::default().fg(Theme::border_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 10 || inner.height < 2 {
            return;
        }

        let children = self.app.get_current_children();
        if children.is_empty() {
            let msg = s.get("filelist.empty");
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, &msg, Style::default().fg(Color::DarkGray));
            return;
        }

        // Calculate visible range with scrolling
        let visible_height = inner.height as usize;
        let total_items = children.len();
        let selected = self.app.selected_index;

        let start_index = if selected >= visible_height {
            selected - visible_height + 1
        } else {
            0
        };

        let _end_index = (start_index + visible_height).min(total_items);

        // Get current node's total size for percentage calculation
        let current_total = self.app.current_node
            .and_then(|id| self.app.tree.get(id))
            .map(|n| n.size)
            .unwrap_or(1);

        // Render items
        for (i, (id, name, size, is_dir)) in children
            .iter()
            .enumerate()
            .skip(start_index)
            .take(visible_height)
        {
            let y = inner.y + (i - start_index) as u16;
            if y >= inner.bottom() {
                break;
            }

            let is_selected = i == selected;
            let node = self.app.tree.get(*id);
            let is_marked = node.map(|n| n.selected).unwrap_or(false);
            let entry_type = node.map(|n| &n.entry_type);

            // Build the line
            let icon = if *is_dir { "" } else { "" };
            let mark = if is_marked { "" } else { " " };

            let percentage = if current_total > 0 {
                *size as f64 / current_total as f64 * 100.0
            } else {
                0.0
            };

            let size_str = format_size(*size);
            let pct_str = format!("{:5.1}%", percentage);

            // Calculate available width for name
            let fixed_width = 3 + 1 + 10 + 1 + 7; // mark + icon + size + space + percentage
            let name_width = inner.width.saturating_sub(fixed_width as u16) as usize;
            let display_name = truncate_name(name, name_width);

            // Color based on entry type
            let type_color = entry_type
                .map(|t| Theme::color_for_entry(t))
                .unwrap_or(Color::White);

            // Style based on selection
            let base_style = if is_selected {
                Style::default()
                    .bg(Theme::selected_bg())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let mark_style = if is_marked {
                base_style.fg(Color::Red)
            } else {
                base_style.fg(Color::DarkGray)
            };

            let icon_style = base_style.fg(type_color);
            let name_style = if *is_dir {
                base_style.fg(Color::Blue).add_modifier(Modifier::BOLD)
            } else {
                base_style.fg(Color::White)
            };
            let size_style = base_style.fg(Color::Cyan);
            let pct_style = base_style.fg(Color::Yellow);

            // Clear line if selected
            if is_selected {
                for x in inner.x..inner.right() {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_style(base_style);
                    }
                }
            }

            // Render each part
            let mut x = inner.x;

            // Mark
            buf.set_string(x, y, mark, mark_style);
            x += 2;

            // Icon
            buf.set_string(x, y, icon, icon_style);
            x += 2;

            // Name
            buf.set_string(x, y, &display_name, name_style);

            // Size (right-aligned)
            let size_x = inner.right().saturating_sub(18);
            buf.set_string(size_x, y, &format!("{:>10}", size_str), size_style);

            // Percentage
            let pct_x = inner.right().saturating_sub(7);
            buf.set_string(pct_x, y, &pct_str, pct_style);
        }

        // Render scroll indicator if needed
        if total_items > visible_height {
            let scroll_info = format!(" {}/{} ", selected + 1, total_items);
            let info_x = inner.right().saturating_sub(scroll_info.len() as u16 + 1);
            buf.set_string(
                info_x,
                inner.y,
                &scroll_info,
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}

fn truncate_name(name: &str, max_len: usize) -> String {
    let char_count = name.chars().count();
    if char_count <= max_len {
        // Pad with spaces to fill width
        let padding = max_len - char_count;
        format!("{}{}", name, " ".repeat(padding))
    } else if max_len >= 4 {
        let truncated: String = name.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    } else {
        name.chars().take(max_len).collect()
    }
}
