use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::app::App;
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
        let icons = self.app.icons();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", s.get("filelist.title")))
            .border_style(Style::default().fg(self.app.theme().border_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 10 || inner.height < 2 {
            return;
        }

        let children = self.app.children();
        // Always show ".." when browsing/scanning - either to go to parent folder or to ComputerView
        let has_parent = self.app.current_node.is_some() && !self.app.in_computer_view;

        // Check if empty (no children and at root)
        if children.is_empty() && !has_parent {
            let msg = s.get("filelist.empty");
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, msg, Style::default().fg(Color::DarkGray));
            return;
        }

        // Calculate total items including ".." entry
        let total_items = children.len() + if has_parent { 1 } else { 0 };
        let visible_height = inner.height as usize;

        // Determine effective selected index (accounting for ".." entry)
        let effective_selected = if has_parent && self.app.parent_entry_selected {
            0
        } else if has_parent {
            self.app.selected_index + 1
        } else {
            self.app.selected_index
        };

        let start_index = if effective_selected >= visible_height {
            effective_selected - visible_height + 1
        } else {
            0
        };

        // Get current node's total size for percentage calculation
        let current_total = self.app.current_node
            .and_then(|id| self.app.tree.get(id))
            .map(|n| n.size)
            .unwrap_or(1);

        let mut y_pos = inner.y;

        // Render ".." entry if has parent
        if has_parent && start_index == 0 {
            let is_selected = self.app.parent_entry_selected;

            let base_style = if is_selected {
                Style::default()
                    .bg(self.app.theme().selected_bg())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Clear line if selected
            if is_selected {
                for x in inner.x..inner.right() {
                    if let Some(cell) = buf.cell_mut((x, y_pos)) {
                        cell.set_style(base_style);
                    }
                }
            }

            let mut x = inner.x;
            buf.set_string(x, y_pos, "  ", base_style.fg(Color::DarkGray));
            x += 2;
            let folder_icon = icons.folder();
            buf.set_string(x, y_pos, folder_icon, base_style);
            x += folder_icon.chars().count() as u16 + 1;
            buf.set_string(x, y_pos, "..", base_style.fg(Color::Yellow).add_modifier(Modifier::BOLD));

            y_pos += 1;
        }

        // Render file items
        let items_start = if has_parent { 1 } else { 0 };
        let items_to_skip = start_index.saturating_sub(items_start);

        for (i, (id, name, size, is_dir)) in children
            .iter()
            .enumerate()
            .skip(items_to_skip)
        {
            if y_pos >= inner.bottom() {
                break;
            }

            let item_display_index = i + items_start;
            if item_display_index < start_index {
                continue;
            }

            let is_selected = !self.app.parent_entry_selected && i == self.app.selected_index;
            let node = self.app.tree.get(*id);
            let is_marked = node.map(|n| n.selected).unwrap_or(false);
            let entry_type = node.map(|n| &n.entry_type);

            // Build the line
            let icon = icons.entry(*is_dir);
            let mark = if is_marked { "●" } else { " " };

            let percentage = if current_total > 0 {
                *size as f64 / current_total as f64 * 100.0
            } else {
                0.0
            };

            let size_str = format_size(*size);
            let pct_str = format!("{:5.1}%", percentage);

            // Calculate available width for name
            let fixed_width = 3 + 3 + 10 + 1 + 7; // mark + icon + size + space + percentage
            let name_width = inner.width.saturating_sub(fixed_width as u16) as usize;
            let display_name = truncate_name(name, name_width);

            // Color based on entry type
            let theme = self.app.theme();
            let type_color = entry_type
                .map(|t| theme.color_for_entry(t))
                .unwrap_or(Color::White);

            // Style based on selection
            let base_style = if is_selected {
                Style::default()
                    .bg(self.app.theme().selected_bg())
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
                    if let Some(cell) = buf.cell_mut((x, y_pos)) {
                        cell.set_style(base_style);
                    }
                }
            }

            // Render each part
            let mut x = inner.x;

            // Mark
            buf.set_string(x, y_pos, mark, mark_style);
            x += 2;

            // Icon
            buf.set_string(x, y_pos, icon, icon_style);
            x += 3;

            // Name
            buf.set_string(x, y_pos, &display_name, name_style);

            // Size (right-aligned)
            let size_x = inner.right().saturating_sub(18);
            buf.set_string(size_x, y_pos, format!("{:>10}", size_str), size_style);

            // Percentage
            let pct_x = inner.right().saturating_sub(7);
            buf.set_string(pct_x, y_pos, &pct_str, pct_style);

            y_pos += 1;
        }

        // Render scroll indicator if needed
        if total_items > visible_height {
            let current_pos = effective_selected + 1;
            let scroll_info = format!(" {}/{} ", current_pos, total_items);
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
