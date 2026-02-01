use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseEventKind, MouseButton};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::DefaultTerminal;

use crate::app::{App, AppMode, ViewMode};
use crate::model::get_all_drives;
use crate::ui::widgets::{AboutWidget, ComputerViewWidget, DriveListWidget, ErrorWidget, FileListWidget, HelpWidget, SettingsWidget, StatsWidget, TreemapWidget};
use crate::util::i18n::Strings;

const TICK_RATE: Duration = Duration::from_millis(16); // ~60 FPS

/// Result of run_app: None means quit, Some(path) means rescan with new path
pub fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<Option<PathBuf>> {
    let mut last_tick = Instant::now();
    let mut terminal_width: u16 = 80;
    let mut terminal_height: u16 = 24;
    let mut last_click_time = Instant::now();
    let mut last_click_pos: (u16, u16) = (0, 0);

    loop {
        // Process scanner messages
        app.process_scan_messages();

        // Check for pending rescan request
        if let Some(new_path) = app.take_pending_rescan() {
            return Ok(Some(new_path));
        }

        // Check for pending admin restart
        if app.pending_admin_restart {
            app.pending_admin_restart = false;
            // The main.rs will handle the actual restart
            return Ok(None);
        }

        // Render
        terminal.draw(|frame| {
            let area = frame.area();
            terminal_width = area.width;
            terminal_height = area.height;
            render_ui(frame, app, area);
        })?;

        // Handle events with timeout
        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        handle_key_event(app, key.code, key.modifiers, terminal_width);
                    }
                }
                Event::Mouse(mouse) => {
                    let is_double_click = if mouse.kind == MouseEventKind::Down(MouseButton::Left) {
                        let now = Instant::now();
                        let pos = (mouse.column, mouse.row);
                        let is_double = last_click_pos == pos && now.duration_since(last_click_time).as_millis() < 400;
                        last_click_time = now;
                        last_click_pos = pos;
                        is_double
                    } else {
                        false
                    };
                    handle_mouse_event(app, mouse.kind, mouse.column, mouse.row, terminal_width, terminal_height, is_double_click);
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= TICK_RATE {
            last_tick = Instant::now();
        }

        if app.should_quit() {
            return Ok(None);
        }
    }
}

/// Calculate grid columns for drive view based on width
fn get_drive_grid_cols(width: u16) -> usize {
    if width > 120 { 3 } else if width > 80 { 2 } else { 1 }
}

fn handle_key_event(app: &mut App, key: KeyCode, modifiers: KeyModifiers, terminal_width: u16) {
    // Clear any existing message on key press
    app.clear_message();

    // Calculate grid columns for drive views
    let drive_cols = get_drive_grid_cols(terminal_width);

    match app.mode {
        AppMode::Scanning => match key {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => app.quit(),
            // Allow navigation during scanning
            // Shift+Up/Down for multi-select
            KeyCode::Up if modifiers.contains(KeyModifiers::SHIFT) => app.move_selection_with_select(-1),
            KeyCode::Down if modifiers.contains(KeyModifiers::SHIFT) => app.move_selection_with_select(1),
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => app.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => app.move_selection(1),
            KeyCode::Enter => app.navigate_into(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => app.navigate_into_dir_only(),
            KeyCode::Backspace | KeyCode::Left => app.navigate_back(),
            KeyCode::Tab => app.toggle_view(),
            KeyCode::Char(' ') => app.toggle_selection(),
            KeyCode::PageUp if modifiers.contains(KeyModifiers::SHIFT) => {
                for _ in 0..10 { app.move_selection_with_select(-1); }
            }
            KeyCode::PageDown if modifiers.contains(KeyModifiers::SHIFT) => {
                for _ in 0..10 { app.move_selection_with_select(1); }
            }
            KeyCode::PageUp => app.move_selection(-10),
            KeyCode::PageDown => app.move_selection(10),
            KeyCode::Home => app.selected_index = 0,
            KeyCode::End => {
                let count = app.get_current_children().len();
                if count > 0 {
                    app.selected_index = count - 1;
                }
            }
            // Allow menus during scanning
            KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => app.toggle_help(),
            KeyCode::Char('a') | KeyCode::Char('A') => app.open_about(),
            KeyCode::Char('s') | KeyCode::Char('S') => app.open_settings(),
            KeyCode::Char('g') | KeyCode::Char('G') => app.open_drive_selector(),
            KeyCode::Char('o') | KeyCode::Char('O') => app.cycle_sort_mode(),
            KeyCode::Char('e') | KeyCode::Char('E') => app.open_in_explorer(),
            KeyCode::Char('c') | KeyCode::Char('C') if modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
            _ => {}
        },
        AppMode::Browsing => match key {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => app.quit(),
            KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => app.toggle_help(),
            KeyCode::Char('a') | KeyCode::Char('A') => app.open_about(),
            KeyCode::Char('s') | KeyCode::Char('S') => app.open_settings(),
            KeyCode::Char('g') | KeyCode::Char('G') => app.open_drive_selector(),
            KeyCode::Char('e') | KeyCode::Char('E') => app.open_in_explorer(),
            // Shift+Up/Down for multi-select
            KeyCode::Up if modifiers.contains(KeyModifiers::SHIFT) => app.move_selection_with_select(-1),
            KeyCode::Down if modifiers.contains(KeyModifiers::SHIFT) => app.move_selection_with_select(1),
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => app.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => app.move_selection(1),
            KeyCode::Enter => app.navigate_into(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => app.navigate_into_dir_only(),
            KeyCode::Backspace | KeyCode::Left => app.navigate_back(),
            KeyCode::Tab => app.toggle_view(),
            KeyCode::Char(' ') => app.toggle_selection(),
            KeyCode::Char('d') | KeyCode::Char('D') => app.confirm_delete(),
            KeyCode::Delete => app.delete_selected_item(),
            KeyCode::Char('o') | KeyCode::Char('O') => app.cycle_sort_mode(),
            KeyCode::PageUp if modifiers.contains(KeyModifiers::SHIFT) => {
                for _ in 0..10 { app.move_selection_with_select(-1); }
            }
            KeyCode::PageDown if modifiers.contains(KeyModifiers::SHIFT) => {
                for _ in 0..10 { app.move_selection_with_select(1); }
            }
            KeyCode::PageUp => app.move_selection(-10),
            KeyCode::PageDown => app.move_selection(10),
            KeyCode::Home => app.selected_index = 0,
            KeyCode::End => {
                let count = app.get_current_children().len();
                if count > 0 {
                    app.selected_index = count - 1;
                }
            }
            KeyCode::Char('c') | KeyCode::Char('C') if modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
            _ => {}
        },
        AppMode::Help => match key {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => {
                app.toggle_help()
            }
            _ => {}
        },
        AppMode::About => match key {
            _ => app.close_about(), // Any key closes about
        },
        AppMode::Settings => match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Char('s') | KeyCode::Char('S') => app.close_settings(),
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => app.move_settings_selection(-1),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => app.move_settings_selection(1),
            KeyCode::Enter | KeyCode::Char(' ') => app.toggle_current_setting(),
            _ => {}
        },
        AppMode::DeleteConfirm => match key {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                let to_trash = app.settings.delete_to_trash;
                let results = app.execute_delete(to_trash);
                let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
                let fail_count = results.len() - success_count;
                let action = if to_trash { "Moved to Recycle Bin" } else { "Deleted" };
                if fail_count > 0 {
                    app.message = Some(format!(
                        "{} {} items, {} failed",
                        action, success_count, fail_count
                    ));
                } else {
                    app.message = Some(format!("{} {} items", action, success_count));
                }
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                // Force permanent delete regardless of settings
                let results = app.execute_delete(false);
                let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
                let fail_count = results.len() - success_count;
                if fail_count > 0 {
                    app.message = Some(format!(
                        "Permanently deleted {} items, {} failed",
                        success_count, fail_count
                    ));
                } else {
                    app.message = Some(format!("Permanently deleted {} items", success_count));
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_delete(),
            _ => {}
        },
        AppMode::DriveSelect => match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => app.close_drive_selector(),
            // Simple list navigation (popup shows drives as vertical list)
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => app.move_drive_selection(-1),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => app.move_drive_selection(1),
            KeyCode::Enter => app.select_drive(),
            KeyCode::Char('g') | KeyCode::Char('G') => {
                // Refresh drive list
                app.drives = get_all_drives();
                app.message = Some("Drive list refreshed".to_string());
            }
            _ => {}
        },
        AppMode::ComputerView => match key {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => app.quit(),
            KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => app.toggle_help(),
            KeyCode::Char('a') | KeyCode::Char('A') => app.open_about(),
            KeyCode::Char('s') | KeyCode::Char('S') => app.open_settings(),
            KeyCode::Char('e') | KeyCode::Char('E') => app.open_in_explorer(),
            // Grid navigation: Up/Down move vertically (by column count)
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => app.move_drive_selection_vertical(-1, drive_cols),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => app.move_drive_selection_vertical(1, drive_cols),
            // Left/Right move horizontally (by 1)
            KeyCode::Left => app.move_drive_selection(-1),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => app.move_drive_selection(1),
            KeyCode::Enter => app.navigate_into(),
            KeyCode::Char('g') | KeyCode::Char('G') => {
                // Refresh drive list
                app.refresh_drives();
            }
            KeyCode::Char('c') | KeyCode::Char('C') if modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
            _ => {}
        },
        AppMode::Error => {
            // Any key dismisses error
            app.dismiss_error();
        },
        AppMode::Quitting => {}
    }
}

fn handle_mouse_event(
    app: &mut App,
    kind: MouseEventKind,
    col: u16,
    row: u16,
    terminal_width: u16,
    terminal_height: u16,
    is_double_click: bool,
) {
    // Calculate layout areas (must match render_ui layout)
    let header_height = 3u16;
    let footer_height = 3u16;
    let content_start = header_height;
    let content_end = terminal_height.saturating_sub(footer_height);
    let _content_height = content_end.saturating_sub(content_start);

    match kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Handle overlay clicks first
            match app.mode {
                AppMode::Help | AppMode::About | AppMode::Error => {
                    // Click anywhere dismisses overlay
                    match app.mode {
                        AppMode::Help => app.toggle_help(),
                        AppMode::About => app.close_about(),
                        AppMode::Error => app.dismiss_error(),
                        _ => {}
                    }
                    return;
                }
                AppMode::Settings => {
                    // Check if click is inside settings area
                    let settings_width = (terminal_width * 65 / 100).min(terminal_width - 4);
                    let settings_height = (terminal_height * 70 / 100).min(terminal_height - 4);
                    let settings_x = (terminal_width - settings_width) / 2;
                    let settings_y = (terminal_height - settings_height) / 2;

                    if col < settings_x || col >= settings_x + settings_width ||
                       row < settings_y || row >= settings_y + settings_height {
                        // Click outside - close settings
                        app.close_settings();
                        return;
                    }
                    // Inside settings - check for option clicks
                    let relative_row = row.saturating_sub(settings_y + 5); // Skip header
                    let option_index = (relative_row / 3) as usize; // Each option takes 3 rows
                    if option_index < 6 {
                        app.settings_selected_index = option_index;
                        if is_double_click {
                            app.toggle_current_setting();
                        }
                    }
                    return;
                }
                AppMode::DriveSelect => {
                    // Check if click is inside drive selector
                    // Must match centered_rect(70, 80, area) used in render_drive_selector
                    let popup_width = terminal_width * 70 / 100;
                    let popup_height = terminal_height * 80 / 100;
                    let popup_x = terminal_width * 15 / 100; // (100-70)/2 = 15%
                    let popup_y = terminal_height * 10 / 100; // (100-80)/2 = 10%

                    if col < popup_x || col >= popup_x + popup_width ||
                       row < popup_y || row >= popup_y + popup_height {
                        app.close_drive_selector();
                        return;
                    }
                    // Inside - select drive based on row
                    // Content starts at popup_y + 1 (border) + 1 (empty line) = popup_y + 2
                    // Each drive takes 3 rows: name, bar, empty
                    let content_start = popup_y + 2;
                    if row >= content_start {
                        let relative_row = row - content_start;
                        let drive_index = (relative_row / 3) as usize;
                        if drive_index < app.drives.len() {
                            app.drive_selected_index = drive_index;
                            if is_double_click {
                                app.select_drive();
                            }
                        }
                    }
                    return;
                }
                _ => {}
            }

            // Footer menu clicks
            let footer_row = terminal_height.saturating_sub(2); // Middle row of footer (border is at -3 and -1)
            if row == footer_row {
                let menu_positions = get_menu_positions(app);
                for (start_x, end_x, action) in menu_positions {
                    if col >= start_x && col < end_x {
                        match action {
                            "help" => app.toggle_help(),
                            "about" => app.open_about(),
                            "settings" => app.open_settings(),
                            "drives" => {
                                if matches!(app.mode, AppMode::Browsing | AppMode::Scanning | AppMode::ComputerView) {
                                    app.open_drive_selector();
                                }
                            }
                            "view" => app.toggle_view(),
                            "sort" => app.cycle_sort_mode(),
                            "explorer" => app.open_in_explorer(),
                            "select" => {
                                if !app.config.read_only {
                                    app.toggle_selection();
                                }
                            }
                            "delete" => {
                                if !app.config.read_only && !app.tree.get_selected().is_empty() {
                                    app.confirm_delete();
                                }
                            }
                            _ => {}
                        }
                        return;
                    }
                }
            }

            // Content area clicks
            if row >= content_start && row < content_end {
                match app.mode {
                    AppMode::ComputerView => {
                        // Drive grid clicks
                        let drive_area_start = content_start + 3; // After title
                        let drive_area_end = content_end.saturating_sub(6); // Before summary

                        if row >= drive_area_start && row < drive_area_end {
                            let cols = get_drive_grid_cols(terminal_width);
                            let card_height = 7u16;
                            let card_width = terminal_width / cols as u16;

                            let relative_row = row - drive_area_start;
                            let grid_row = (relative_row / card_height) as usize;
                            let grid_col = (col / card_width) as usize;

                            let drive_index = grid_row * cols + grid_col;
                            if drive_index < app.drives.len() {
                                app.drive_selected_index = drive_index;
                                if is_double_click {
                                    app.navigate_into();
                                }
                            }
                        }
                    }
                    AppMode::Scanning | AppMode::Browsing => {
                        // File list clicks (in List or Split view)
                        if app.view_mode == ViewMode::List || app.view_mode == ViewMode::Split {
                            let list_start = if app.mode == AppMode::Scanning {
                                content_start + 8 // After scan progress
                            } else {
                                content_start
                            };

                            // List takes 60% in List mode, 30% in Split mode
                            let list_width = match app.view_mode {
                                ViewMode::List => terminal_width * 60 / 100,
                                ViewMode::Split => terminal_width * 30 / 100,
                                _ => 0,
                            };
                            let list_x_start = match app.view_mode {
                                ViewMode::Split => terminal_width * 40 / 100,
                                _ => 0,
                            };

                            if col >= list_x_start && col < list_x_start + list_width &&
                               row >= list_start + 1 && row < content_end - 1 {
                                let relative_row = (row - list_start - 1) as usize;
                                let children_count = app.get_current_children().len();
                                let has_parent = app.current_node.is_some() && !app.in_computer_view;

                                // Account for ".." entry at top
                                if has_parent {
                                    if relative_row == 0 {
                                        // Clicked on ".."
                                        if is_double_click {
                                            app.navigate_back();
                                        } else if app.parent_entry_selected {
                                            // Already selected, navigate back
                                            app.navigate_back();
                                        } else {
                                            app.selected_index = 0;
                                            app.parent_entry_selected = true;
                                        }
                                    } else {
                                        let item_index = relative_row - 1;
                                        if item_index < children_count {
                                            let was_selected = !app.parent_entry_selected && app.selected_index == item_index;
                                            if is_double_click {
                                                app.selected_index = item_index;
                                                app.parent_entry_selected = false;
                                                app.navigate_into();
                                            } else if was_selected {
                                                // Click on already selected item - open it
                                                app.open_selected_item();
                                            } else {
                                                app.selected_index = item_index;
                                                app.parent_entry_selected = false;
                                            }
                                        }
                                    }
                                } else {
                                    if relative_row < children_count {
                                        let was_selected = !app.parent_entry_selected && app.selected_index == relative_row;
                                        if is_double_click {
                                            app.selected_index = relative_row;
                                            app.parent_entry_selected = false;
                                            app.navigate_into();
                                        } else if was_selected {
                                            // Click on already selected item - open it
                                            app.open_selected_item();
                                        } else {
                                            app.selected_index = relative_row;
                                            app.parent_entry_selected = false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        MouseEventKind::Down(MouseButton::Right) => {
            // Right click to toggle selection for deletion
            if row >= content_start && row < content_end {
                if matches!(app.mode, AppMode::Scanning | AppMode::Browsing) {
                    if !app.config.read_only {
                        app.toggle_selection();
                    }
                }
            }
        }
        MouseEventKind::ScrollUp => {
            match app.mode {
                AppMode::Scanning | AppMode::Browsing => app.move_selection(-3),
                AppMode::ComputerView => app.move_drive_selection(-1),
                AppMode::DriveSelect => app.move_drive_selection(-1),
                AppMode::Settings => app.move_settings_selection(-1),
                _ => {}
            }
        }
        MouseEventKind::ScrollDown => {
            match app.mode {
                AppMode::Scanning | AppMode::Browsing => app.move_selection(3),
                AppMode::ComputerView => app.move_drive_selection(1),
                AppMode::DriveSelect => app.move_drive_selection(1),
                AppMode::Settings => app.move_settings_selection(1),
                _ => {}
            }
        }
        _ => {}
    }
}

fn render_ui(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    // Main layout: header, content, footer
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Footer
        ])
        .split(area);

    render_header(frame, app, main_layout[0]);
    render_content(frame, app, main_layout[1]);
    render_footer(frame, app, main_layout[2]);

    // Render overlays
    match app.mode {
        AppMode::Help => render_help_overlay(frame, app, area),
        AppMode::About => render_about_overlay(frame, app, area),
        AppMode::Settings => render_settings_overlay(frame, app, area),
        AppMode::DeleteConfirm => render_delete_confirm(frame, app, area),
        AppMode::DriveSelect => render_drive_selector(frame, app, area),
        AppMode::Error => render_error_overlay(frame, app, area),
        _ => {}
    }
}

fn render_header(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let s = Strings::new(app.settings.language);
    let current_path = app
        .current_node
        .and_then(|id| app.tree.get_path(id))
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| app.config.target_path.display().to_string());

    let title = format!(
        " {} - {} ",
        s.get("app.title"),
        crate::util::format::truncate_path(&current_path, area.width.saturating_sub(35) as usize)
    );

    let stats = app.tree.get(app.current_node.unwrap_or_default());
    let size_str = stats
        .map(|n| crate::util::format::format_size(n.size))
        .unwrap_or_default();

    let theme = app.theme();
    let header = Paragraph::new(Line::from(vec![
        Span::styled(title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(size_str, Style::default().fg(Color::Cyan)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_color())),
    );

    frame.render_widget(header, area);
}

fn render_content(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    match app.mode {
        AppMode::Scanning => {
            // Split view: show progress on top, content below
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),  // Progress bar
                    Constraint::Min(5),     // Content
                ])
                .split(area);

            render_scanning_compact(frame, app, layout[0]);
            render_main_view(frame, app, layout[1]);
        }
        AppMode::ComputerView => {
            let computer_view = ComputerViewWidget::new(app);
            frame.render_widget(computer_view, area);
        }
        _ => render_main_view(frame, app, area),
    }
}

fn render_scanning_compact(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let s = Strings::new(app.settings.language);
    let progress = app.scan_progress.as_ref();

    let text = if let Some(p) = progress {
        let elapsed = format!("{:.1}s", p.elapsed.as_secs_f64());
        let speed = format!("{:.0}/s", p.entries_per_second);

        vec![
            Line::from(vec![
                Span::styled(format!(" ● {} ", s.get("scan.scanning")), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(format!("{} ", s.get("scan.files"))),
                Span::styled(
                    crate::util::format::format_count(p.files_scanned),
                    Style::default().fg(Color::Green),
                ),
                Span::raw(format!("  {} ", s.get("scan.dirs"))),
                Span::styled(
                    crate::util::format::format_count(p.dirs_scanned),
                    Style::default().fg(Color::Blue),
                ),
                Span::raw(format!("  {} ", s.get("scan.size"))),
                Span::styled(
                    crate::util::format::format_size(p.total_size),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(format!("  {} ", s.get("scan.time"))),
                Span::styled(elapsed, Style::default().fg(Color::Magenta)),
                Span::raw(format!("  {} ", s.get("scan.speed"))),
                Span::styled(speed, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(
                    crate::util::format::truncate_path(
                        &p.current_path.display().to_string(),
                        area.width.saturating_sub(6) as usize,
                    ),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                format!(" {}", s.get("scan.hint")),
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                format!(" ● {}", s.get("scan.starting")),
                Style::default().fg(Color::Yellow),
            )),
        ]
    };

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", s.get("app.scanning")))
                .title_style(Style::default().fg(Color::Yellow))
                .border_style(Style::default().fg(Color::Yellow)),
        );

    frame.render_widget(paragraph, area);
}

fn render_main_view(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    match app.view_mode {
        ViewMode::Treemap => {
            let treemap = TreemapWidget::new(app);
            frame.render_widget(treemap, area);
        }
        ViewMode::List => {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(area);

            let file_list = FileListWidget::new(app);
            frame.render_widget(file_list, layout[0]);

            let stats = StatsWidget::new(app);
            frame.render_widget(stats, layout[1]);
        }
        ViewMode::Split => {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                ])
                .split(area);

            let treemap = TreemapWidget::new(app);
            frame.render_widget(treemap, layout[0]);

            let file_list = FileListWidget::new(app);
            frame.render_widget(file_list, layout[1]);

            let stats = StatsWidget::new(app);
            frame.render_widget(stats, layout[2]);
        }
    }
}

fn render_footer(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let s = Strings::new(app.settings.language);
    let mode_str = match app.view_mode {
        ViewMode::Treemap => "TREEMAP",
        ViewMode::List => "LIST",
        ViewMode::Split => "SPLIT",
    };

    let selected_count = app.tree.get_selected().len();
    let selected_str = if selected_count > 0 {
        format!(" {} {}", selected_count, s.get("footer.selected"))
    } else {
        String::new()
    };

    let message = app.message.clone().unwrap_or_default();

    // Build clickable menu items
    let menu_style = Style::default().fg(Color::Black).bg(Color::DarkGray);
    let key_style = Style::default().fg(Color::Yellow).bg(Color::DarkGray).add_modifier(Modifier::BOLD);

    let mut spans = vec![
        Span::styled(
            format!(" [{}] ", mode_str),
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::raw(" "),
    ];

    // Menu items: [?Help] [aAbout] [sSettings] [gDrives] [TabView]
    spans.push(Span::styled(" ", menu_style));
    spans.push(Span::styled("?", key_style));
    spans.push(Span::styled(format!("{} ", s.get("footer.help")), menu_style));

    spans.push(Span::raw(" "));

    spans.push(Span::styled(" ", menu_style));
    spans.push(Span::styled("a", key_style));
    spans.push(Span::styled(format!("{} ", s.get("footer.about")), menu_style));

    spans.push(Span::raw(" "));

    spans.push(Span::styled(" ", menu_style));
    spans.push(Span::styled("s", key_style));
    spans.push(Span::styled(format!("{} ", s.get("footer.settings")), menu_style));

    spans.push(Span::raw(" "));

    spans.push(Span::styled(" ", menu_style));
    spans.push(Span::styled("g", key_style));
    spans.push(Span::styled(format!("{} ", s.get("footer.drives")), menu_style));

    spans.push(Span::raw(" "));

    spans.push(Span::styled(" ", menu_style));
    spans.push(Span::styled("Tab", key_style));
    spans.push(Span::styled(format!("{} ", s.get("footer.view")), menu_style));

    spans.push(Span::raw(" "));

    // Sort button with current sort mode
    spans.push(Span::styled(" ", menu_style));
    spans.push(Span::styled("o", key_style));
    spans.push(Span::styled(format!("{} ", s.get("footer.sort")), menu_style));
    spans.push(Span::styled(
        format!("[{}]", app.sort_mode.label()),
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
    ));

    spans.push(Span::raw(" "));

    // Explorer button
    spans.push(Span::styled(" ", menu_style));
    spans.push(Span::styled("e", key_style));
    spans.push(Span::styled(format!("{} ", s.get("footer.explorer")), menu_style));

    if !app.config.read_only {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(" ", menu_style));
        spans.push(Span::styled("Sp", key_style));
        spans.push(Span::styled(format!("{} ", s.get("footer.select")), menu_style));

        spans.push(Span::raw(" "));
        spans.push(Span::styled(" ", menu_style));
        spans.push(Span::styled("d", key_style));
        spans.push(Span::styled(format!("{} ", s.get("footer.delete")), menu_style));
    } else {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            format!(" [{}] ", s.get("footer.read_only")),
            Style::default().fg(Color::Red),
        ));
    }

    if !selected_str.is_empty() {
        spans.push(Span::styled(selected_str, Style::default().fg(Color::Yellow)));
    }

    if !message.is_empty() {
        spans.push(Span::styled(format!(" | {} ", message), Style::default().fg(Color::Green)));
    }

    let theme = app.theme();
    let footer = Paragraph::new(Line::from(spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_color())),
        );

    frame.render_widget(footer, area);
}

/// Calculate menu item positions for click detection
/// Returns a list of (start_x, end_x, action) tuples
/// Calculate display width of a string (character count, not byte count)
/// This is important for proper click detection with non-ASCII characters
fn display_width(s: &str) -> u16 {
    s.chars().count() as u16
}

fn get_menu_positions(app: &App) -> Vec<(u16, u16, &'static str)> {
    let s = Strings::new(app.settings.language);
    let mode_len: u16 = match app.view_mode {
        ViewMode::Treemap => 10, // " [TREEMAP] "
        ViewMode::List => 8,    // " [LIST] "
        ViewMode::Split => 9,   // " [SPLIT] "
    };

    let mut positions = Vec::new();
    let mut x = mode_len + 2; // After mode indicator and space

    // ?Help: " " + "?" + "text "
    let help_len = 2 + display_width(s.get("footer.help")) + 1;
    positions.push((x, x + help_len, "help"));
    x += help_len + 1;

    // aAbout
    let about_len = 2 + display_width(s.get("footer.about")) + 1;
    positions.push((x, x + about_len, "about"));
    x += about_len + 1;

    // sSettings
    let settings_len = 2 + display_width(s.get("footer.settings")) + 1;
    positions.push((x, x + settings_len, "settings"));
    x += settings_len + 1;

    // gDrives
    let drives_len = 2 + display_width(s.get("footer.drives")) + 1;
    positions.push((x, x + drives_len, "drives"));
    x += drives_len + 1;

    // TabView: " " + "Tab" + "text "
    let view_len = 4 + display_width(s.get("footer.view")) + 1;
    positions.push((x, x + view_len, "view"));
    x += view_len + 1;

    // oSort: " " + "o" + "text " + "[MODE]"
    let sort_len = 2 + display_width(s.get("footer.sort")) + 1 + display_width(app.sort_mode.label()) + 2;
    positions.push((x, x + sort_len, "sort"));
    x += sort_len + 1;

    // eExplorer: " " + "e" + "text "
    let explorer_len = 2 + display_width(s.get("footer.explorer")) + 1;
    positions.push((x, x + explorer_len, "explorer"));
    x += explorer_len + 1;

    if !app.config.read_only {
        // SpSelect: " " + "Sp" + "text "
        let select_len = 3 + display_width(s.get("footer.select")) + 1;
        positions.push((x, x + select_len, "select"));
        x += select_len + 1;

        // dDelete
        let delete_len = 2 + display_width(s.get("footer.delete")) + 1;
        positions.push((x, x + delete_len, "delete"));
    }

    positions
}

fn render_help_overlay(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let help = HelpWidget::new(app.settings.language, app.settings.color_palette);
    let help_area = centered_rect(60, 70, area);
    frame.render_widget(Clear, help_area);
    frame.render_widget(help, help_area);
}

fn render_about_overlay(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let about = AboutWidget::new(app.settings.language, app.settings.color_palette);
    let about_area = centered_rect(50, 60, area);
    frame.render_widget(Clear, about_area);
    frame.render_widget(about, about_area);
}

fn render_settings_overlay(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let settings = SettingsWidget::new(app);
    let settings_area = centered_rect(65, 70, area);
    frame.render_widget(Clear, settings_area);
    frame.render_widget(settings, settings_area);
}

fn render_error_overlay(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let message = app.error_message.as_deref().unwrap_or("Unknown error");
    let error = ErrorWidget::new(message, app.settings.language);
    let error_area = centered_rect(60, 65, area);
    frame.render_widget(Clear, error_area);
    frame.render_widget(error, error_area);
}

fn render_delete_confirm(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let s = Strings::new(app.settings.language);
    let selected = app.get_selected_for_deletion();
    let total_size: u64 = selected.iter().map(|(_, _, s)| s).sum();
    let to_trash = app.settings.delete_to_trash;

    let warning_text = if to_trash {
        s.get("delete.warning_trash")
    } else {
        s.get("delete.warning")
    };

    let confirm_text = if to_trash {
        format!("{} ({})", s.get("delete.yes"), s.get("settings.delete_to_trash"))
    } else {
        s.get("delete.yes").to_string()
    };

    let mut text = vec![
        Line::from(""),
        Line::from(Span::styled(
            s.get("delete.confirm").to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("{} {}", s.get("delete.items"), selected.len())),
        Line::from(format!(
            "{} {}",
            s.get("delete.total_size"),
            crate::util::format::format_size(total_size)
        )),
        Line::from(""),
        Line::from(Span::styled(
            warning_text.to_string(),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
    ];

    // Add buttons
    let mut buttons = vec![
        Span::styled("[Y]", Style::default().fg(Color::Green)),
        Span::raw(format!(" {}  ", confirm_text)),
    ];

    // Always show permanent delete option
    buttons.push(Span::styled("[P]", Style::default().fg(Color::Magenta)));
    buttons.push(Span::raw(format!(" {}  ", s.get("delete.yes_permanent"))));

    buttons.push(Span::styled("[N]", Style::default().fg(Color::Red)));
    buttons.push(Span::raw(format!(" {}", s.get("delete.no"))));

    text.push(Line::from(buttons));

    let dialog = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", s.get("delete.title")))
                .border_style(Style::default().fg(Color::Red)),
        )
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    let dialog_area = centered_rect(55, 45, area);
    frame.render_widget(Clear, dialog_area);
    frame.render_widget(dialog, dialog_area);
}

fn render_drive_selector(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let drive_list = DriveListWidget::new(app);
    let dialog_area = centered_rect(70, 80, area);
    frame.render_widget(Clear, dialog_area);
    frame.render_widget(drive_list, dialog_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
