use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

use crate::app::{App, StartupLocation};
use crate::platform::get_platform_labels;
use crate::util::i18n::Strings;

pub struct SettingsWidget<'a> {
    app: &'a App,
}

impl<'a> SettingsWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }

    fn render_option(
        &self,
        index: usize,
        label: &str,
        value: &str,
        description: &str,
        is_enabled: bool,
    ) -> Vec<Line<'static>> {
        let is_selected = self.app.settings_selected_index == index;

        let marker = if is_selected { "▶ " } else { "  " };
        let marker_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let label_style = if is_selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let value_color = if is_enabled { Color::Green } else { Color::Red };
        let value_style = Style::default().fg(value_color).add_modifier(Modifier::BOLD);

        let desc_style = Style::default().fg(Color::DarkGray);

        let bg_style = if is_selected {
            Style::default().bg(self.app.theme().selected_bg())
        } else {
            Style::default()
        };

        vec![
            Line::from(vec![
                Span::styled(format!("  {}", marker), marker_style),
                Span::styled(format!("{:<35}", label), label_style.patch(bg_style)),
                Span::styled(value.to_string(), value_style.patch(bg_style)),
            ]),
            Line::from(vec![
                Span::styled("      ", desc_style),
                Span::styled(description.to_string(), desc_style),
            ]),
            Line::from(""),
        ]
    }
}

impl Widget for SettingsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lang = self.app.settings.language;
        let s = Strings::new(lang);
        let theme = self.app.theme();
        let icons = self.app.icons();
        let platform_labels = get_platform_labels();

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", s.get("settings.title")))
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .border_style(Style::default().fg(theme.highlight_color()));

        let inner = block.inner(area);
        block.render(area, buf);

        let header_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
        let dim_style = Style::default().fg(Color::DarkGray);

        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(format!("  {} {}", icons.settings(), s.get("settings.header")), header_style)),
            Line::from(""),
            Line::from(Span::styled(
                "  ─────────────────────────────────────────────",
                dim_style,
            )),
            Line::from(""),
        ];

        let cache = &self.app.settings_cache;

        // Context Menu option
        let context_enabled = cache.context_menu_registered;
        let context_value = if context_enabled {
            s.get("settings.enabled")
        } else {
            s.get("settings.disabled")
        };
        lines.extend(self.render_option(
            0,
            platform_labels.context_menu,
            context_value,
            platform_labels.context_menu_desc,
            context_enabled,
        ));

        // Startup Location option
        let startup_value = match self.app.settings.startup_location {
            StartupLocation::LastLocation => s.get("startup.last_location"),
            StartupLocation::CurrentFolder => s.get("startup.current_folder"),
            StartupLocation::ComputerView => s.get("startup.computer_view"),
        };
        lines.extend(self.render_option(
            1,
            s.get("settings.startup"),
            startup_value,
            s.get("settings.startup_desc"),
            true,
        ));

        // PATH Registration option
        let path_enabled = cache.path_registered;
        let path_value = if path_enabled {
            s.get("settings.registered")
        } else {
            s.get("settings.not_registered")
        };
        let path_desc = if path_enabled {
            cache.install_path.clone()
        } else {
            s.get("settings.path_reg_desc").to_string()
        };
        lines.extend(self.render_option(
            2,
            s.get("settings.path_reg"),
            path_value,
            &path_desc,
            path_enabled,
        ));

        // Menu shortcut option (Start Menu on Windows, Applications on Linux/macOS)
        let menu_enabled = cache.start_menu_shortcut_exists;
        let menu_value = if menu_enabled {
            s.get("settings.created")
        } else {
            s.get("settings.not_created")
        };
        let menu_desc = if menu_enabled {
            cache.start_menu_path.clone()
        } else {
            platform_labels.menu_shortcut_desc.to_string()
        };
        lines.extend(self.render_option(
            3,
            platform_labels.menu_shortcut,
            menu_value,
            &menu_desc,
            menu_enabled,
        ));

        // Desktop shortcut option
        let desktop_enabled = cache.desktop_shortcut_exists;
        let desktop_value = if desktop_enabled {
            s.get("settings.created")
        } else {
            s.get("settings.not_created")
        };
        let desktop_desc = if desktop_enabled {
            cache.desktop_path.clone()
        } else {
            platform_labels.desktop_shortcut_desc.to_string()
        };
        lines.extend(self.render_option(
            4,
            platform_labels.desktop_shortcut,
            desktop_value,
            &desktop_desc,
            desktop_enabled,
        ));

        // Language option
        let lang_value = self.app.settings.language.display_name();
        lines.extend(self.render_option(
            5,
            s.get("settings.language"),
            lang_value,
            s.get("settings.language_desc"),
            true,
        ));

        // Color palette option
        let palette_value = self.app.settings.color_palette.name();
        lines.extend(self.render_option(
            6,
            s.get("settings.palette"),
            palette_value,
            s.get("settings.palette_desc"),
            true,
        ));

        // ASCII icons option
        let icons_value = if self.app.settings.use_ascii_icons {
            s.get("settings.icons_ascii")
        } else {
            s.get("settings.icons_unicode")
        };
        lines.extend(self.render_option(
            7,
            s.get("settings.icons"),
            icons_value,
            s.get("settings.icons_desc"),
            true,
        ));

        // Allow delete option
        let allow_delete_value = if self.app.settings.allow_delete {
            s.get("settings.enabled")
        } else {
            s.get("settings.disabled")
        };
        lines.extend(self.render_option(
            8,
            s.get("settings.allow_delete"),
            allow_delete_value,
            s.get("settings.allow_delete_desc"),
            self.app.settings.allow_delete,
        ));

        // Delete method option
        let delete_method_value = if self.app.settings.delete_to_trash {
            platform_labels.trash_name
        } else {
            s.get("settings.delete_permanent")
        };
        lines.extend(self.render_option(
            9,
            s.get("settings.delete_method"),
            delete_method_value,
            s.get("settings.delete_method_desc"),
            true,
        ));

        // Delete confirmation option
        let delete_confirm_value = if self.app.settings.show_delete_confirmation {
            s.get("settings.enabled")
        } else {
            s.get("settings.disabled")
        };
        lines.extend(self.render_option(
            10,
            s.get("settings.delete_confirm"),
            delete_confirm_value,
            s.get("settings.delete_confirm_desc"),
            self.app.settings.show_delete_confirmation,
        ));

        // Run as Admin/Root option
        let is_admin = cache.running_as_admin;
        let admin_value = if self.app.settings.run_as_admin {
            s.get("settings.enabled")
        } else {
            s.get("settings.disabled")
        };
        let admin_desc = if is_admin {
            s.get("settings.admin_active")
        } else {
            s.get("settings.admin_not_active")
        };
        lines.extend(self.render_option(
            11,
            platform_labels.admin_label,
            admin_value,
            &format!("{} - {}", platform_labels.admin_desc, admin_desc),
            self.app.settings.run_as_admin,
        ));

        // Font name option
        let font_available = !cache.available_fonts.is_empty();
        lines.extend(self.render_option(
            12,
            s.get("settings.font_name"),
            &self.app.settings.font_name,
            s.get("settings.font_name_desc"),
            font_available,
        ));

        // Font size option
        let font_size_str = format!("{}pt", self.app.settings.font_size);
        lines.extend(self.render_option(
            13,
            s.get("settings.font_size"),
            &font_size_str,
            s.get("settings.font_size_desc"),
            font_available,
        ));

        // Footer
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  ─────────────────────────────────────────────",
            dim_style,
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  ↑↓", Style::default().fg(Color::Yellow)),
            Span::styled(format!(": {} ", s.get("settings.hint")), dim_style),
            Span::styled("Enter/Space", Style::default().fg(Color::Yellow)),
            Span::styled(format!(": {} ", s.get("settings.toggle")), dim_style),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::styled(format!(": {}", s.get("settings.close")), dim_style),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {}", s.get("settings.admin_note")),
            Style::default().fg(Color::Red),
        )));

        // Calculate scroll offset to keep selected option visible
        // Header takes 5 lines, each option takes 3 lines
        let header_lines: u16 = 5;
        let selected = self.app.settings_selected_index as u16;
        let option_top = header_lines + selected * 3;
        let visible_height = inner.height;
        let scroll_y = if option_top + 3 > visible_height {
            option_top.saturating_sub(visible_height / 3)
        } else {
            0
        };

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((scroll_y, 0));
        paragraph.render(inner, buf);
    }
}
