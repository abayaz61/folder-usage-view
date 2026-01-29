use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Widget};

use crate::app::App;
use crate::model::NodeId;
use crate::treemap::{LayoutRect, TreemapLayout};
use crate::ui::theme::Theme;

pub struct TreemapWidget<'a> {
    app: &'a App,
}

impl<'a> TreemapWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl Widget for TreemapWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Treemap ")
            .border_style(Style::default().fg(Theme::border_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 3 || inner.height < 2 {
            return;
        }

        let Some(current_id) = self.app.current_node else {
            return;
        };

        // Get children for treemap
        let children = self.app.tree.get_children_sorted_by_size(current_id);
        if children.is_empty() {
            let msg = "Empty directory";
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            buf.set_string(x, y, msg, Style::default().fg(Color::DarkGray));
            return;
        }

        // Prepare items for layout
        let items: Vec<(NodeId, u64)> = children
            .iter()
            .filter(|(_, node)| node.size > 0)
            .map(|(id, node)| (*id, node.size))
            .collect();

        if items.is_empty() {
            return;
        }

        // Compute layout
        let layout = TreemapLayout::new();
        let bounds = LayoutRect::from(inner);
        let rectangles = layout.layout(&items, bounds);

        // Get selected index for highlighting
        let selected_children = self.app.get_current_children();
        let selected_id = selected_children
            .get(self.app.selected_index)
            .map(|(id, _, _, _)| *id);

        // Render each rectangle
        for (node_id, rect) in rectangles {
            let terminal_rect = rect.to_terminal_rect();

            // Skip if too small
            if terminal_rect.width < 1 || terminal_rect.height < 1 {
                continue;
            }

            let node = match self.app.tree.get(node_id) {
                Some(n) => n,
                None => continue,
            };

            let is_selected = Some(node_id) == selected_id;
            let is_marked = node.selected;

            // Get color based on entry type
            let base_color = Theme::color_for_entry(&node.entry_type);

            // Determine style
            let style = if is_selected {
                Style::default()
                    .bg(Theme::highlight_color())
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else if is_marked {
                Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
            } else {
                Style::default().bg(base_color).fg(Color::Black)
            };

            // Fill rectangle
            for y in terminal_rect.y..(terminal_rect.y + terminal_rect.height).min(inner.bottom()) {
                for x in terminal_rect.x..(terminal_rect.x + terminal_rect.width).min(inner.right())
                {
                    if let Some(cell) = buf.cell_mut((x, y)) {
                        cell.set_style(style);
                    }
                }
            }

            // Draw border
            if terminal_rect.width >= 2 && terminal_rect.height >= 1 {
                let border_style = if is_selected {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Black)
                };

                // Top border
                for x in terminal_rect.x..(terminal_rect.x + terminal_rect.width).min(inner.right())
                {
                    if terminal_rect.y < inner.bottom() {
                        if let Some(cell) = buf.cell_mut((x, terminal_rect.y)) {
                            cell.set_char('─').set_style(border_style);
                        }
                    }
                }

                // Left border
                for y in terminal_rect.y..(terminal_rect.y + terminal_rect.height).min(inner.bottom())
                {
                    if terminal_rect.x < inner.right() {
                        if let Some(cell) = buf.cell_mut((terminal_rect.x, y)) {
                            cell.set_char('│').set_style(border_style);
                        }
                    }
                }

                // Corner
                if terminal_rect.x < inner.right() && terminal_rect.y < inner.bottom() {
                    if let Some(cell) = buf.cell_mut((terminal_rect.x, terminal_rect.y)) {
                        cell.set_char('┌').set_style(border_style);
                    }
                }
            }

            // Draw label if space permits
            if terminal_rect.width >= 4 && terminal_rect.height >= 2 {
                let label = truncate_label(&node.name, (terminal_rect.width - 2) as usize);
                let size_label = crate::util::format::format_size_short(node.size);

                let label_x = terminal_rect.x + 1;
                let label_y = terminal_rect.y + 1;

                // Draw name
                let label_style = if is_selected {
                    Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Black)
                };

                for (i, ch) in label.chars().enumerate() {
                    let x = label_x + i as u16;
                    if x < terminal_rect.x + terminal_rect.width - 1 && x < inner.right() {
                        if let Some(cell) = buf.cell_mut((x, label_y)) {
                            cell.set_char(ch).set_style(label_style);
                        }
                    }
                }

                // Draw size if there's room
                if terminal_rect.height >= 3 && terminal_rect.width >= size_label.len() as u16 + 2 {
                    let size_y = label_y + 1;
                    let size_style = Style::default().fg(Color::DarkGray);

                    for (i, ch) in size_label.chars().enumerate() {
                        let x = label_x + i as u16;
                        if x < terminal_rect.x + terminal_rect.width - 1 && x < inner.right() {
                            if let Some(cell) = buf.cell_mut((x, size_y)) {
                                cell.set_char(ch).set_style(size_style);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn truncate_label(name: &str, max_len: usize) -> String {
    let char_count = name.chars().count();
    if char_count <= max_len {
        name.to_string()
    } else if max_len >= 4 {
        let truncated: String = name.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    } else {
        name.chars().take(max_len).collect()
    }
}
