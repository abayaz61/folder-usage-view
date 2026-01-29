use super::rect::LayoutRect;

pub struct TreemapLayout {
    min_visible_area: f64,
}

impl TreemapLayout {
    pub fn new() -> Self {
        Self {
            min_visible_area: 1.0,
        }
    }

    pub fn layout<T: Clone>(&self, items: &[(T, u64)], bounds: LayoutRect) -> Vec<(T, LayoutRect)> {
        if items.is_empty() || bounds.width < 1.0 || bounds.height < 1.0 {
            return Vec::new();
        }

        let total_size: u64 = items.iter().map(|(_, s)| *s).sum();
        if total_size == 0 {
            return Vec::new();
        }

        // Sort by size descending
        let mut sorted_items: Vec<_> = items.to_vec();
        sorted_items.sort_by(|a, b| b.1.cmp(&a.1));

        let mut result = Vec::with_capacity(items.len());
        self.squarify(&sorted_items, bounds, total_size as f64, &mut result);
        result
    }

    fn squarify<T: Clone>(
        &self,
        items: &[(T, u64)],
        bounds: LayoutRect,
        total_size: f64,
        result: &mut Vec<(T, LayoutRect)>,
    ) {
        if items.is_empty() || bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        if items.len() == 1 {
            if bounds.area() >= self.min_visible_area {
                result.push((items[0].0.clone(), bounds));
            }
            return;
        }

        let area = bounds.area();
        let scale = area / total_size;

        // Layout along shorter side
        let shorter = bounds.shorter_side();
        let horizontal = bounds.is_horizontal();

        let mut row: Vec<(T, u64)> = Vec::new();
        let mut row_size: f64 = 0.0;
        let mut remaining_start = 0;

        for (i, (item, size)) in items.iter().enumerate() {
            let size_f = *size as f64;

            // Calculate worst aspect ratio with and without this item
            let current_worst = if row.is_empty() {
                f64::INFINITY
            } else {
                self.worst_aspect_ratio(&row, row_size, shorter, scale)
            };

            let new_worst = self.worst_aspect_ratio_with(
                &row,
                row_size,
                (item.clone(), *size),
                size_f,
                shorter,
                scale,
            );

            if new_worst <= current_worst {
                // Add to current row
                row.push((item.clone(), *size));
                row_size += size_f;
                remaining_start = i + 1;
            } else {
                // Layout current row and start fresh
                break;
            }
        }

        // If we collected all items into one row, just lay them out
        if remaining_start == 0 && !row.is_empty() {
            remaining_start = row.len();
        }

        // Layout the row
        let row_area = row_size * scale;
        let row_thickness = if shorter > 0.0 {
            row_area / shorter
        } else {
            0.0
        };

        let (row_bounds, remaining_bounds) = if horizontal {
            (
                LayoutRect::new(bounds.x, bounds.y, row_thickness, bounds.height),
                LayoutRect::new(
                    bounds.x + row_thickness,
                    bounds.y,
                    bounds.width - row_thickness,
                    bounds.height,
                ),
            )
        } else {
            (
                LayoutRect::new(bounds.x, bounds.y, bounds.width, row_thickness),
                LayoutRect::new(
                    bounds.x,
                    bounds.y + row_thickness,
                    bounds.width,
                    bounds.height - row_thickness,
                ),
            )
        };

        // Layout items in row
        self.layout_row(&row, row_bounds, row_size, scale, horizontal, result);

        // Recursively layout remaining items
        if remaining_start < items.len() {
            let remaining: Vec<_> = items[remaining_start..].to_vec();
            let remaining_size: f64 = remaining.iter().map(|(_, s)| *s as f64).sum();
            self.squarify(&remaining, remaining_bounds, remaining_size, result);
        }
    }

    fn worst_aspect_ratio<T>(&self, row: &[(T, u64)], row_size: f64, side: f64, scale: f64) -> f64 {
        if row.is_empty() || side == 0.0 || row_size == 0.0 {
            return f64::INFINITY;
        }

        let row_area = row_size * scale;
        let row_width = row_area / side;

        let mut worst = 0.0f64;
        for (_, size) in row {
            let item_area = *size as f64 * scale;
            let item_height = item_area / row_width;

            let ratio = if row_width > item_height {
                row_width / item_height
            } else {
                item_height / row_width
            };

            worst = worst.max(ratio);
        }

        worst
    }

    fn worst_aspect_ratio_with<T: Clone>(
        &self,
        row: &[(T, u64)],
        row_size: f64,
        new_item: (T, u64),
        new_size: f64,
        side: f64,
        scale: f64,
    ) -> f64 {
        let mut extended_row: Vec<_> = row.to_vec();
        extended_row.push(new_item);
        self.worst_aspect_ratio(&extended_row, row_size + new_size, side, scale)
    }

    fn layout_row<T: Clone>(
        &self,
        row: &[(T, u64)],
        bounds: LayoutRect,
        row_size: f64,
        _scale: f64,
        horizontal: bool,
        result: &mut Vec<(T, LayoutRect)>,
    ) {
        if row.is_empty() || row_size == 0.0 {
            return;
        }

        let mut offset = 0.0;

        for (item, size) in row {
            let fraction = *size as f64 / row_size;
            let item_length = if horizontal {
                bounds.height * fraction
            } else {
                bounds.width * fraction
            };

            let rect = if horizontal {
                LayoutRect::new(bounds.x, bounds.y + offset, bounds.width, item_length)
            } else {
                LayoutRect::new(bounds.x + offset, bounds.y, item_length, bounds.height)
            };

            if rect.area() >= self.min_visible_area {
                result.push((item.clone(), rect));
            }

            offset += item_length;
        }
    }
}

impl Default for TreemapLayout {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_layout() {
        let layout = TreemapLayout::new();
        let items = vec![("A", 60u64), ("B", 30), ("C", 10)];
        let bounds = LayoutRect::new(0.0, 0.0, 100.0, 100.0);

        let result = layout.layout(&items, bounds);

        assert_eq!(result.len(), 3);

        // Total area should equal bounds area
        let total_area: f64 = result.iter().map(|(_, r)| r.area()).sum();
        assert!((total_area - 10000.0).abs() < 100.0); // Allow small floating point error
    }

    #[test]
    fn test_empty_items() {
        let layout = TreemapLayout::new();
        let items: Vec<(&str, u64)> = vec![];
        let bounds = LayoutRect::new(0.0, 0.0, 100.0, 100.0);

        let result = layout.layout(&items, bounds);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_item() {
        let layout = TreemapLayout::new();
        let items = vec![("A", 100u64)];
        let bounds = LayoutRect::new(0.0, 0.0, 50.0, 50.0);

        let result = layout.layout(&items, bounds);

        assert_eq!(result.len(), 1);
        let (_, rect) = &result[0];
        assert!((rect.width - 50.0).abs() < 0.1);
        assert!((rect.height - 50.0).abs() < 0.1);
    }
}
