use ratatui::style::Color;

use crate::model::{EntryType, FileCategory};

pub struct Theme;

impl Theme {
    pub fn color_for_entry(entry_type: &EntryType) -> Color {
        match entry_type {
            EntryType::Directory => Color::Blue,
            EntryType::File(category) => Self::color_for_category(category),
            EntryType::Symlink => Color::Cyan,
            EntryType::Unknown => Color::DarkGray,
        }
    }

    pub fn color_for_category(category: &FileCategory) -> Color {
        match category {
            FileCategory::Document => Color::Green,
            FileCategory::Image => Color::Magenta,
            FileCategory::Video => Color::Red,
            FileCategory::Audio => Color::Yellow,
            FileCategory::Archive => Color::LightYellow,
            FileCategory::Code => Color::LightGreen,
            FileCategory::Executable => Color::LightRed,
            FileCategory::Data => Color::Gray,
            FileCategory::Other => Color::DarkGray,
        }
    }

    pub fn selected_bg() -> Color {
        Color::Rgb(60, 60, 80)
    }

    pub fn header_bg() -> Color {
        Color::Rgb(40, 40, 60)
    }

    pub fn border_color() -> Color {
        Color::Rgb(80, 80, 100)
    }

    pub fn highlight_color() -> Color {
        Color::Rgb(100, 150, 255)
    }

    pub fn progress_color() -> Color {
        Color::Green
    }

    pub fn warning_color() -> Color {
        Color::Yellow
    }

    pub fn error_color() -> Color {
        Color::Red
    }

    pub fn muted_text() -> Color {
        Color::Rgb(128, 128, 128)
    }
}
