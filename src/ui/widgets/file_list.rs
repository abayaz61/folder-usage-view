use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::app::App;
use crate::util::format::format_size;
use crate::util::i18n::Strings;

pub struct FileListWidget<'a> {
    app: &'a App,
    show_bar: bool,
}

impl<'a> FileListWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app, show_bar: false }
    }

    /// Show the mini percentage bar next to the percentage value.
    /// Used in List mode where the panel is wide enough.
    pub fn show_bar(mut self, show: bool) -> Self {
        self.show_bar = show;
        self
    }
}

/// Width of the mini bar in characters. Each cell = 5% (20 cells = 100%).
const BAR_WIDTH: usize = 20;

/// Build the filled and empty parts of the percentage mini-bar separately so
/// each can be rendered with a different color (filled = magnitude color,
/// empty = subtle gray so the bar boundary stays visible on dark backgrounds).
///
/// Uses eighth-block partial characters for sub-cell resolution: a 20-cell bar
/// with 8 sub-units per cell effectively resolves ~0.6% steps.
fn percentage_bar_parts(percentage: f64) -> (String, String) {
    let pct = percentage.clamp(0.0, 100.0);
    // Total sub-units across the whole bar (8 sub-units per cell).
    let sub_units = pct / 100.0 * (BAR_WIDTH as f64) * 8.0;
    let full_cells = (sub_units / 8.0).floor() as usize;
    let remainder = sub_units as usize % 8;

    // Eighth-block characters: index = filled eighths (0 = empty, 8 = full).
    const PARTIALS: [char; 8] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'];

    let mut filled = String::new();
    for _ in 0..full_cells.min(BAR_WIDTH) {
        filled.push('█');
    }
    let has_partial = full_cells < BAR_WIDTH && remainder > 0;
    if has_partial {
        filled.push(PARTIALS[remainder]);
    }

    let filled_len = full_cells.min(BAR_WIDTH) + if has_partial { 1 } else { 0 };
    let empty = "░".repeat(BAR_WIDTH.saturating_sub(filled_len));

    (filled, empty)
}

/// Color for the filled portion based on how much of the parent's space the
/// entry occupies.
fn bar_color(percentage: f64) -> Color {
    if percentage >= 60.0 {
        Color::Red
    } else if percentage >= 25.0 {
        Color::Yellow
    } else {
        Color::Green
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
            // mark(2) + icon(3) + size(10) + space(1) + percentage(7) [+ bar(11) if shown]
            // bar = 1 space + BAR_WIDTH(20) = 21 chars
            let fixed_width = if self.show_bar {
                3 + 3 + 10 + 1 + 7 + 1 + BAR_WIDTH
            } else {
                3 + 3 + 10 + 1 + 7
            };
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
            // With bar: [size 10][space 1][pct 7][space 1][bar BAR_WIDTH] from right
            //          = 10 + 1 + 7 + 1 + BAR_WIDTH = 19 + BAR_WIDTH = 39
            // Without bar: [size 10][space 1][pct 7] = 18 chars from right
            let size_offset: u16 = if self.show_bar {
                19 + BAR_WIDTH as u16
            } else {
                18
            };
            let size_x = inner.right().saturating_sub(size_offset);
            buf.set_string(size_x, y_pos, format!("{:>10}", size_str), size_style);

            // Percentage
            let pct_offset: u16 = if self.show_bar {
                8 + BAR_WIDTH as u16
            } else {
                7
            };
            let pct_x = inner.right().saturating_sub(pct_offset);
            buf.set_string(pct_x, y_pos, &pct_str, pct_style);

            // Mini bar (only in List mode where panel is wide enough).
            // Filled and empty parts are rendered separately so each gets its
            // own color: filled = magnitude color, empty = subtle gray.
            if self.show_bar {
                let (filled, empty) = percentage_bar_parts(percentage);
                let bar_start_x = inner.right().saturating_sub(BAR_WIDTH as u16);
                let mag_color = bar_color(percentage);
                // Filled portion (magnitude color)
                if !filled.is_empty() {
                    buf.set_string(
                        bar_start_x,
                        y_pos,
                        &filled,
                        base_style.fg(mag_color),
                    );
                }
                // Empty portion (subtle gray so the bar boundary stays visible)
                let empty_x = bar_start_x + filled.chars().count() as u16;
                if empty_x < inner.right() {
                    buf.set_string(
                        empty_x,
                        y_pos,
                        &empty,
                        base_style.fg(Color::DarkGray),
                    );
                }
            }

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
