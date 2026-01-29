use ratatui::style::Color;

use crate::model::{EntryType, FileCategory};

/// Available color palettes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ColorPalette {
    #[default]
    Default,        // Original blue/purple theme
    HighContrast,   // Black & white with bold colors (good for RDP)
    SolarizedDark,  // Solarized dark theme
    SolarizedLight, // Solarized light theme
    Monokai,        // Monokai editor theme
    Nord,           // Nord arctic theme
    Dracula,        // Dracula dark theme
    GruvboxDark,    // Gruvbox dark theme
    OneDark,        // Atom One Dark theme
    Terminal,       // Classic green terminal
}

impl ColorPalette {
    pub fn next(self) -> Self {
        match self {
            ColorPalette::Default => ColorPalette::HighContrast,
            ColorPalette::HighContrast => ColorPalette::SolarizedDark,
            ColorPalette::SolarizedDark => ColorPalette::SolarizedLight,
            ColorPalette::SolarizedLight => ColorPalette::Monokai,
            ColorPalette::Monokai => ColorPalette::Nord,
            ColorPalette::Nord => ColorPalette::Dracula,
            ColorPalette::Dracula => ColorPalette::GruvboxDark,
            ColorPalette::GruvboxDark => ColorPalette::OneDark,
            ColorPalette::OneDark => ColorPalette::Terminal,
            ColorPalette::Terminal => ColorPalette::Default,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ColorPalette::Default => "Default",
            ColorPalette::HighContrast => "High Contrast",
            ColorPalette::SolarizedDark => "Solarized Dark",
            ColorPalette::SolarizedLight => "Solarized Light",
            ColorPalette::Monokai => "Monokai",
            ColorPalette::Nord => "Nord",
            ColorPalette::Dracula => "Dracula",
            ColorPalette::GruvboxDark => "Gruvbox Dark",
            ColorPalette::OneDark => "One Dark",
            ColorPalette::Terminal => "Terminal Green",
        }
    }

    pub fn all() -> &'static [ColorPalette] {
        &[
            ColorPalette::Default,
            ColorPalette::HighContrast,
            ColorPalette::SolarizedDark,
            ColorPalette::SolarizedLight,
            ColorPalette::Monokai,
            ColorPalette::Nord,
            ColorPalette::Dracula,
            ColorPalette::GruvboxDark,
            ColorPalette::OneDark,
            ColorPalette::Terminal,
        ]
    }
}

/// Theme colors based on selected palette
pub struct Theme {
    palette: ColorPalette,
}

impl Theme {
    pub fn new(palette: ColorPalette) -> Self {
        Self { palette }
    }

    // === Entry/File Type Colors ===

    pub fn color_for_entry(&self, entry_type: &EntryType) -> Color {
        match entry_type {
            EntryType::Directory => self.directory_color(),
            EntryType::File(category) => self.color_for_category(category),
            EntryType::Symlink => self.symlink_color(),
            EntryType::Unknown => self.muted_text(),
        }
    }

    pub fn directory_color(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::Blue,
            ColorPalette::HighContrast => Color::Blue,
            ColorPalette::SolarizedDark => Color::Blue,
            ColorPalette::SolarizedLight => Color::Blue,
            ColorPalette::Monokai => Color::Rgb(102, 217, 239), // Cyan
            ColorPalette::Nord => Color::Rgb(136, 192, 208),    // Nord8
            ColorPalette::Dracula => Color::Rgb(139, 233, 253), // Cyan
            ColorPalette::GruvboxDark => Color::Rgb(131, 165, 152), // Aqua
            ColorPalette::OneDark => Color::Rgb(97, 175, 239),  // Blue
            ColorPalette::Terminal => Color::Cyan,
        }
    }

    pub fn symlink_color(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::Cyan,
            ColorPalette::HighContrast => Color::Cyan,
            ColorPalette::SolarizedDark => Color::Cyan,
            ColorPalette::SolarizedLight => Color::Cyan,
            ColorPalette::Monokai => Color::Rgb(174, 129, 255), // Purple
            ColorPalette::Nord => Color::Rgb(180, 142, 173),    // Nord15
            ColorPalette::Dracula => Color::Rgb(189, 147, 249), // Purple
            ColorPalette::GruvboxDark => Color::Rgb(211, 134, 155), // Purple
            ColorPalette::OneDark => Color::Rgb(198, 120, 221), // Purple
            ColorPalette::Terminal => Color::Magenta,
        }
    }

    pub fn color_for_category(&self, category: &FileCategory) -> Color {
        match self.palette {
            ColorPalette::Default => Self::default_category_color(category),
            ColorPalette::HighContrast => Self::high_contrast_category_color(category),
            ColorPalette::SolarizedDark | ColorPalette::SolarizedLight => Self::solarized_category_color(category),
            ColorPalette::Monokai => Self::monokai_category_color(category),
            ColorPalette::Nord => Self::nord_category_color(category),
            ColorPalette::Dracula => Self::dracula_category_color(category),
            ColorPalette::GruvboxDark => Self::gruvbox_category_color(category),
            ColorPalette::OneDark => Self::onedark_category_color(category),
            ColorPalette::Terminal => Self::terminal_category_color(category),
        }
    }

    fn default_category_color(category: &FileCategory) -> Color {
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

    fn high_contrast_category_color(category: &FileCategory) -> Color {
        match category {
            FileCategory::Document => Color::Green,
            FileCategory::Image => Color::Magenta,
            FileCategory::Video => Color::Red,
            FileCategory::Audio => Color::Yellow,
            FileCategory::Archive => Color::Rgb(255, 200, 0),
            FileCategory::Code => Color::Rgb(0, 255, 0),
            FileCategory::Executable => Color::Rgb(255, 100, 100),
            FileCategory::Data => Color::White,
            FileCategory::Other => Color::Gray,
        }
    }

    fn solarized_category_color(category: &FileCategory) -> Color {
        match category {
            FileCategory::Document => Color::Rgb(133, 153, 0),   // Green
            FileCategory::Image => Color::Rgb(211, 54, 130),    // Magenta
            FileCategory::Video => Color::Rgb(220, 50, 47),     // Red
            FileCategory::Audio => Color::Rgb(181, 137, 0),     // Yellow
            FileCategory::Archive => Color::Rgb(203, 75, 22),   // Orange
            FileCategory::Code => Color::Rgb(38, 139, 210),     // Blue
            FileCategory::Executable => Color::Rgb(220, 50, 47), // Red
            FileCategory::Data => Color::Rgb(147, 161, 161),    // Base1
            FileCategory::Other => Color::Rgb(88, 110, 117),    // Base01
        }
    }

    fn monokai_category_color(category: &FileCategory) -> Color {
        match category {
            FileCategory::Document => Color::Rgb(166, 226, 46),  // Green
            FileCategory::Image => Color::Rgb(174, 129, 255),   // Purple
            FileCategory::Video => Color::Rgb(249, 38, 114),    // Pink
            FileCategory::Audio => Color::Rgb(230, 219, 116),   // Yellow
            FileCategory::Archive => Color::Rgb(253, 151, 31),  // Orange
            FileCategory::Code => Color::Rgb(102, 217, 239),    // Cyan
            FileCategory::Executable => Color::Rgb(249, 38, 114), // Pink
            FileCategory::Data => Color::Rgb(117, 113, 94),     // Comment
            FileCategory::Other => Color::Rgb(117, 113, 94),
        }
    }

    fn nord_category_color(category: &FileCategory) -> Color {
        match category {
            FileCategory::Document => Color::Rgb(163, 190, 140), // Nord14 Green
            FileCategory::Image => Color::Rgb(180, 142, 173),   // Nord15 Purple
            FileCategory::Video => Color::Rgb(191, 97, 106),    // Nord11 Red
            FileCategory::Audio => Color::Rgb(235, 203, 139),   // Nord13 Yellow
            FileCategory::Archive => Color::Rgb(208, 135, 112), // Nord12 Orange
            FileCategory::Code => Color::Rgb(136, 192, 208),    // Nord8 Cyan
            FileCategory::Executable => Color::Rgb(191, 97, 106), // Nord11 Red
            FileCategory::Data => Color::Rgb(76, 86, 106),      // Nord3
            FileCategory::Other => Color::Rgb(67, 76, 94),      // Nord2
        }
    }

    fn dracula_category_color(category: &FileCategory) -> Color {
        match category {
            FileCategory::Document => Color::Rgb(80, 250, 123),  // Green
            FileCategory::Image => Color::Rgb(255, 121, 198),   // Pink
            FileCategory::Video => Color::Rgb(255, 85, 85),     // Red
            FileCategory::Audio => Color::Rgb(241, 250, 140),   // Yellow
            FileCategory::Archive => Color::Rgb(255, 184, 108), // Orange
            FileCategory::Code => Color::Rgb(139, 233, 253),    // Cyan
            FileCategory::Executable => Color::Rgb(255, 85, 85), // Red
            FileCategory::Data => Color::Rgb(98, 114, 164),     // Comment
            FileCategory::Other => Color::Rgb(68, 71, 90),      // Current line
        }
    }

    fn gruvbox_category_color(category: &FileCategory) -> Color {
        match category {
            FileCategory::Document => Color::Rgb(184, 187, 38),  // Green
            FileCategory::Image => Color::Rgb(211, 134, 155),   // Purple
            FileCategory::Video => Color::Rgb(251, 73, 52),     // Red
            FileCategory::Audio => Color::Rgb(250, 189, 47),    // Yellow
            FileCategory::Archive => Color::Rgb(254, 128, 25),  // Orange
            FileCategory::Code => Color::Rgb(131, 165, 152),    // Aqua
            FileCategory::Executable => Color::Rgb(251, 73, 52), // Red
            FileCategory::Data => Color::Rgb(146, 131, 116),    // Gray
            FileCategory::Other => Color::Rgb(102, 92, 84),     // Dark gray
        }
    }

    fn onedark_category_color(category: &FileCategory) -> Color {
        match category {
            FileCategory::Document => Color::Rgb(152, 195, 121), // Green
            FileCategory::Image => Color::Rgb(198, 120, 221),   // Purple
            FileCategory::Video => Color::Rgb(224, 108, 117),   // Red
            FileCategory::Audio => Color::Rgb(229, 192, 123),   // Yellow
            FileCategory::Archive => Color::Rgb(209, 154, 102), // Orange
            FileCategory::Code => Color::Rgb(86, 182, 194),     // Cyan
            FileCategory::Executable => Color::Rgb(224, 108, 117), // Red
            FileCategory::Data => Color::Rgb(92, 99, 112),      // Comment
            FileCategory::Other => Color::Rgb(76, 82, 99),      // Gutter
        }
    }

    fn terminal_category_color(category: &FileCategory) -> Color {
        match category {
            FileCategory::Document => Color::Green,
            FileCategory::Image => Color::LightMagenta,
            FileCategory::Video => Color::LightRed,
            FileCategory::Audio => Color::LightYellow,
            FileCategory::Archive => Color::Yellow,
            FileCategory::Code => Color::LightGreen,
            FileCategory::Executable => Color::Red,
            FileCategory::Data => Color::Gray,
            FileCategory::Other => Color::DarkGray,
        }
    }

    // === UI Colors ===

    pub fn selected_bg(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::Rgb(60, 60, 80),
            ColorPalette::HighContrast => Color::White,
            ColorPalette::SolarizedDark => Color::Rgb(7, 54, 66),
            ColorPalette::SolarizedLight => Color::Rgb(238, 232, 213),
            ColorPalette::Monokai => Color::Rgb(73, 72, 62),
            ColorPalette::Nord => Color::Rgb(67, 76, 94),
            ColorPalette::Dracula => Color::Rgb(68, 71, 90),
            ColorPalette::GruvboxDark => Color::Rgb(80, 73, 69),
            ColorPalette::OneDark => Color::Rgb(44, 49, 58),
            ColorPalette::Terminal => Color::DarkGray,
        }
    }

    pub fn selected_fg(&self) -> Color {
        match self.palette {
            ColorPalette::HighContrast => Color::Black,
            ColorPalette::SolarizedLight => Color::Black,
            _ => Color::White,
        }
    }

    pub fn header_bg(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::Rgb(40, 40, 60),
            ColorPalette::HighContrast => Color::Black,
            ColorPalette::SolarizedDark => Color::Rgb(0, 43, 54),
            ColorPalette::SolarizedLight => Color::Rgb(253, 246, 227),
            ColorPalette::Monokai => Color::Rgb(39, 40, 34),
            ColorPalette::Nord => Color::Rgb(46, 52, 64),
            ColorPalette::Dracula => Color::Rgb(40, 42, 54),
            ColorPalette::GruvboxDark => Color::Rgb(40, 40, 40),
            ColorPalette::OneDark => Color::Rgb(33, 37, 43),
            ColorPalette::Terminal => Color::Black,
        }
    }

    pub fn border_color(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::Rgb(80, 80, 100),
            ColorPalette::HighContrast => Color::White,
            ColorPalette::SolarizedDark => Color::Rgb(88, 110, 117),
            ColorPalette::SolarizedLight => Color::Rgb(147, 161, 161),
            ColorPalette::Monokai => Color::Rgb(117, 113, 94),
            ColorPalette::Nord => Color::Rgb(76, 86, 106),
            ColorPalette::Dracula => Color::Rgb(98, 114, 164),
            ColorPalette::GruvboxDark => Color::Rgb(146, 131, 116),
            ColorPalette::OneDark => Color::Rgb(62, 68, 81),
            ColorPalette::Terminal => Color::Green,
        }
    }

    pub fn highlight_color(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::Rgb(100, 150, 255),
            ColorPalette::HighContrast => Color::Yellow,
            ColorPalette::SolarizedDark => Color::Rgb(38, 139, 210),
            ColorPalette::SolarizedLight => Color::Rgb(38, 139, 210),
            ColorPalette::Monokai => Color::Rgb(166, 226, 46),
            ColorPalette::Nord => Color::Rgb(136, 192, 208),
            ColorPalette::Dracula => Color::Rgb(189, 147, 249),
            ColorPalette::GruvboxDark => Color::Rgb(250, 189, 47),
            ColorPalette::OneDark => Color::Rgb(97, 175, 239),
            ColorPalette::Terminal => Color::LightGreen,
        }
    }

    pub fn progress_color(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::Green,
            ColorPalette::HighContrast => Color::Green,
            ColorPalette::SolarizedDark => Color::Rgb(133, 153, 0),
            ColorPalette::SolarizedLight => Color::Rgb(133, 153, 0),
            ColorPalette::Monokai => Color::Rgb(166, 226, 46),
            ColorPalette::Nord => Color::Rgb(163, 190, 140),
            ColorPalette::Dracula => Color::Rgb(80, 250, 123),
            ColorPalette::GruvboxDark => Color::Rgb(184, 187, 38),
            ColorPalette::OneDark => Color::Rgb(152, 195, 121),
            ColorPalette::Terminal => Color::Green,
        }
    }

    pub fn warning_color(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::Yellow,
            ColorPalette::HighContrast => Color::Yellow,
            ColorPalette::SolarizedDark => Color::Rgb(181, 137, 0),
            ColorPalette::SolarizedLight => Color::Rgb(181, 137, 0),
            ColorPalette::Monokai => Color::Rgb(230, 219, 116),
            ColorPalette::Nord => Color::Rgb(235, 203, 139),
            ColorPalette::Dracula => Color::Rgb(241, 250, 140),
            ColorPalette::GruvboxDark => Color::Rgb(250, 189, 47),
            ColorPalette::OneDark => Color::Rgb(229, 192, 123),
            ColorPalette::Terminal => Color::Yellow,
        }
    }

    pub fn error_color(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::Red,
            ColorPalette::HighContrast => Color::Red,
            ColorPalette::SolarizedDark => Color::Rgb(220, 50, 47),
            ColorPalette::SolarizedLight => Color::Rgb(220, 50, 47),
            ColorPalette::Monokai => Color::Rgb(249, 38, 114),
            ColorPalette::Nord => Color::Rgb(191, 97, 106),
            ColorPalette::Dracula => Color::Rgb(255, 85, 85),
            ColorPalette::GruvboxDark => Color::Rgb(251, 73, 52),
            ColorPalette::OneDark => Color::Rgb(224, 108, 117),
            ColorPalette::Terminal => Color::Red,
        }
    }

    pub fn muted_text(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::Rgb(128, 128, 128),
            ColorPalette::HighContrast => Color::Gray,
            ColorPalette::SolarizedDark => Color::Rgb(88, 110, 117),
            ColorPalette::SolarizedLight => Color::Rgb(147, 161, 161),
            ColorPalette::Monokai => Color::Rgb(117, 113, 94),
            ColorPalette::Nord => Color::Rgb(76, 86, 106),
            ColorPalette::Dracula => Color::Rgb(98, 114, 164),
            ColorPalette::GruvboxDark => Color::Rgb(146, 131, 116),
            ColorPalette::OneDark => Color::Rgb(92, 99, 112),
            ColorPalette::Terminal => Color::DarkGray,
        }
    }

    pub fn text_primary(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::White,
            ColorPalette::HighContrast => Color::White,
            ColorPalette::SolarizedDark => Color::Rgb(131, 148, 150),
            ColorPalette::SolarizedLight => Color::Rgb(101, 123, 131),
            ColorPalette::Monokai => Color::Rgb(248, 248, 242),
            ColorPalette::Nord => Color::Rgb(216, 222, 233),
            ColorPalette::Dracula => Color::Rgb(248, 248, 242),
            ColorPalette::GruvboxDark => Color::Rgb(235, 219, 178),
            ColorPalette::OneDark => Color::Rgb(171, 178, 191),
            ColorPalette::Terminal => Color::Green,
        }
    }

    pub fn background(&self) -> Color {
        match self.palette {
            ColorPalette::Default => Color::Reset,
            ColorPalette::HighContrast => Color::Black,
            ColorPalette::SolarizedDark => Color::Rgb(0, 43, 54),
            ColorPalette::SolarizedLight => Color::Rgb(253, 246, 227),
            ColorPalette::Monokai => Color::Rgb(39, 40, 34),
            ColorPalette::Nord => Color::Rgb(46, 52, 64),
            ColorPalette::Dracula => Color::Rgb(40, 42, 54),
            ColorPalette::GruvboxDark => Color::Rgb(40, 40, 40),
            ColorPalette::OneDark => Color::Rgb(40, 44, 52),
            ColorPalette::Terminal => Color::Black,
        }
    }
}

// Static helper for backward compatibility when palette isn't available
impl Theme {
    pub fn border_color_static() -> Color {
        Color::Rgb(80, 80, 100)
    }

    pub fn selected_bg_static() -> Color {
        Color::Rgb(60, 60, 80)
    }

    pub fn highlight_color_static() -> Color {
        Color::Rgb(100, 150, 255)
    }
}

/// Icon set that supports both Unicode (emoji) and ASCII modes
/// ASCII mode is useful for remote desktop connections where Unicode doesn't render properly
pub struct Icons {
    use_ascii: bool,
}

impl Icons {
    pub fn new(use_ascii: bool) -> Self {
        Self { use_ascii }
    }

    /// Folder icon
    pub fn folder(&self) -> &'static str {
        if self.use_ascii { "[D]" } else { "📁" }
    }

    /// File icon
    pub fn file(&self) -> &'static str {
        if self.use_ascii { "[F]" } else { "📄" }
    }

    /// Removable drive icon (USB, etc.)
    pub fn drive_removable(&self) -> &'static str {
        if self.use_ascii { "[R]" } else { "💾" }
    }

    /// Fixed drive icon (HDD, SSD)
    pub fn drive_fixed(&self) -> &'static str {
        if self.use_ascii { "[D]" } else { "💿" }
    }

    /// Parent directory (..)
    pub fn parent(&self) -> &'static str {
        if self.use_ascii { "[..]" } else { "📁" }
    }

    /// Settings/gear icon
    pub fn settings(&self) -> &'static str {
        if self.use_ascii { "[*]" } else { "⚙" }
    }

    /// Get icon for a drive based on whether it's removable
    pub fn drive(&self, is_removable: bool) -> &'static str {
        if is_removable {
            self.drive_removable()
        } else {
            self.drive_fixed()
        }
    }

    /// Get icon for an entry based on whether it's a directory
    pub fn entry(&self, is_dir: bool) -> &'static str {
        if is_dir {
            self.folder()
        } else {
            self.file()
        }
    }
}
