use humansize::{format_size as hs_format, BINARY};

pub fn format_size(bytes: u64) -> String {
    hs_format(bytes, BINARY)
}

pub fn format_size_short(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }

    const UNITS: &[&str] = &["B", "K", "M", "G", "T", "P"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} B", bytes)
    } else if size >= 100.0 {
        format!("{:.0}{}", size, UNITS[unit_index])
    } else if size >= 10.0 {
        format!("{:.1}{}", size, UNITS[unit_index])
    } else {
        format!("{:.2}{}", size, UNITS[unit_index])
    }
}

pub fn format_count(count: u64) -> String {
    if count < 1000 {
        count.to_string()
    } else if count < 1_000_000 {
        format!("{:.1}K", count as f64 / 1000.0)
    } else if count < 1_000_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else {
        format!("{:.1}B", count as f64 / 1_000_000_000.0)
    }
}

pub fn truncate_path(path: &str, max_len: usize) -> String {
    let char_count = path.chars().count();
    if char_count <= max_len {
        return path.to_string();
    }

    if max_len < 5 {
        return path.chars().take(max_len).collect();
    }

    let start_len = max_len / 3;
    let end_len = max_len - start_len - 3;

    let start: String = path.chars().take(start_len).collect();
    let end: String = path.chars().skip(char_count - end_len).collect();
    format!("{}...{}", start, end)
}

/// Safely truncate a string to max_len characters (not bytes)
/// Appends "..." if truncated
pub fn truncate_str(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        s.chars().take(max_len).collect()
    } else {
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

/// Safely truncate a string and pad to exact width
pub fn truncate_and_pad(s: &str, width: usize) -> String {
    let char_count = s.chars().count();
    if char_count == width {
        s.to_string()
    } else if char_count < width {
        format!("{}{}", s, " ".repeat(width - char_count))
    } else if width <= 3 {
        s.chars().take(width).collect()
    } else {
        let truncated: String = s.chars().take(width - 3).collect();
        format!("{}...", truncated)
    }
}

/// Get character count (not byte count)
pub fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Safe substring by character indices
pub fn safe_substring(s: &str, start: usize, end: usize) -> String {
    s.chars().skip(start).take(end - start).collect()
}
