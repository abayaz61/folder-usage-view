use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Terminal;

use crate::app::{App, AppMode, ViewMode};
use crate::model::get_all_drives;
use crate::ui::widgets::{AboutWidget, ComputerViewWidget, DriveListWidget, ErrorWidget, FileListWidget, HelpWidget, SettingsWidget, StatsWidget, TreemapWidget};

const TICK_RATE: Duration = Duration::from_millis(16); // ~60 FPS

/// Result of run_app: None means quit, Some(path) means rescan with new path
pub fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<Option<PathBuf>> {
    let mut last_tick = Instant::now();
    let mut terminal_width: u16 = 80;

    loop {
        // Process scanner messages
        app.process_scan_messages();

        // Check for pending rescan request
        if let Some(new_path) = app.take_pending_rescan() {
            return Ok(Some(new_path));
        }

        // Render
        terminal.draw(|frame| {
            let area = frame.area();
            terminal_width = area.width;
            render_ui(frame, app, area);
        })?;

        // Handle events with timeout
        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(app, key.code, key.modifiers, terminal_width);
                }
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
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => app.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => app.move_selection(1),
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => app.navigate_into(),
            KeyCode::Backspace | KeyCode::Left => app.navigate_back(),
            KeyCode::Tab => app.toggle_view(),
            KeyCode::PageUp => app.move_selection(-10),
            KeyCode::PageDown => app.move_selection(10),
            KeyCode::Home => app.selected_index = 0,
            KeyCode::End => {
                let count = app.get_current_children().len();
                if count > 0 {
                    app.selected_index = count - 1;
                }
            }
            KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => app.toggle_help(),
            KeyCode::Char('c') | KeyCode::Char('C') if modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
            _ => {}
        },
        AppMode::Browsing => match key {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => app.quit(),
            KeyCode::Char('?') | KeyCode::Char('h') | KeyCode::Char('H') => app.toggle_help(),
            KeyCode::Char('a') | KeyCode::Char('A') => app.open_about(),
            KeyCode::Char('s') | KeyCode::Char('S') => app.open_settings(),
            KeyCode::Char('g') | KeyCode::Char('G') => app.open_drive_selector(),
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => app.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => app.move_selection(1),
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => app.navigate_into(),
            KeyCode::Backspace | KeyCode::Left => app.navigate_back(),
            KeyCode::Tab => app.toggle_view(),
            KeyCode::Char(' ') => app.toggle_selection(),
            KeyCode::Char('d') | KeyCode::Char('D') => app.confirm_delete(),
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
                let results = app.execute_delete();
                let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
                let fail_count = results.len() - success_count;
                if fail_count > 0 {
                    app.message = Some(format!(
                        "Deleted {} items, {} failed",
                        success_count, fail_count
                    ));
                } else {
                    app.message = Some(format!("Deleted {} items", success_count));
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
        AppMode::Help => render_help_overlay(frame, area),
        AppMode::About => render_about_overlay(frame, area),
        AppMode::Settings => render_settings_overlay(frame, app, area),
        AppMode::DeleteConfirm => render_delete_confirm(frame, app, area),
        AppMode::DriveSelect => render_drive_selector(frame, app, area),
        AppMode::Error => render_error_overlay(frame, app, area),
        _ => {}
    }
}

fn render_header(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let current_path = app
        .current_node
        .and_then(|id| app.tree.get_path(id))
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| app.config.target_path.display().to_string());

    let title = format!(
        " Disk Usage Analyzer - {} ",
        crate::util::format::truncate_path(&current_path, area.width.saturating_sub(30) as usize)
    );

    let stats = app.tree.get(app.current_node.unwrap_or_default());
    let size_str = stats
        .map(|n| crate::util::format::format_size(n.size))
        .unwrap_or_default();

    let header = Paragraph::new(Line::from(vec![
        Span::styled(title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(size_str, Style::default().fg(Color::Cyan)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(super::theme::Theme::border_color())),
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
    let progress = app.scan_progress.as_ref();

    let text = if let Some(p) = progress {
        let elapsed = format!("{:.1}s", p.elapsed.as_secs_f64());
        let speed = format!("{:.0}/s", p.entries_per_second);

        vec![
            Line::from(vec![
                Span::styled(" ● Scanning ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Files: "),
                Span::styled(
                    crate::util::format::format_count(p.files_scanned),
                    Style::default().fg(Color::Green),
                ),
                Span::raw("  Dirs: "),
                Span::styled(
                    crate::util::format::format_count(p.dirs_scanned),
                    Style::default().fg(Color::Blue),
                ),
                Span::raw("  Size: "),
                Span::styled(
                    crate::util::format::format_size(p.total_size),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("  Time: "),
                Span::styled(elapsed, Style::default().fg(Color::Magenta)),
                Span::raw("  Speed: "),
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
                " ↑↓: Navigate  Enter: Open  Tab: View  q: Cancel scan",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        vec![
            Line::from(Span::styled(
                " ● Starting scan...",
                Style::default().fg(Color::Yellow),
            )),
        ]
    };

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Scanning in Progress ")
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
    let mode_str = match app.view_mode {
        ViewMode::Treemap => "TREEMAP",
        ViewMode::List => "LIST",
        ViewMode::Split => "SPLIT",
    };

    let selected_count = app.tree.get_selected().len();
    let selected_str = if selected_count > 0 {
        format!(" | {} selected", selected_count)
    } else {
        String::new()
    };

    let message = app.message.clone().unwrap_or_default();

    let help_text = if app.config.read_only {
        "q:Quit ?:Help a:About s:Settings g:Drives Tab:View [READ-ONLY]"
    } else {
        "q:Quit ?:Help a:About s:Settings g:Drives Tab:View Space:Select d:Delete"
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" [{}] ", mode_str),
            Style::default().fg(Color::Black).bg(Color::Cyan),
        ),
        Span::raw(" "),
        Span::styled(help_text, Style::default().fg(Color::DarkGray)),
        Span::styled(selected_str, Style::default().fg(Color::Yellow)),
        if !message.is_empty() {
            Span::styled(format!(" | {} ", message), Style::default().fg(Color::Green))
        } else {
            Span::raw("")
        },
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(super::theme::Theme::border_color())),
    );

    frame.render_widget(footer, area);
}

fn render_help_overlay(frame: &mut ratatui::Frame, area: Rect) {
    let help = HelpWidget::new();
    let help_area = centered_rect(60, 70, area);
    frame.render_widget(Clear, help_area);
    frame.render_widget(help, help_area);
}

fn render_about_overlay(frame: &mut ratatui::Frame, area: Rect) {
    let about = AboutWidget::new();
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
    let error = ErrorWidget::new(message);
    let error_area = centered_rect(60, 65, area);
    frame.render_widget(Clear, error_area);
    frame.render_widget(error, error_area);
}

fn render_delete_confirm(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let selected = app.get_selected_for_deletion();
    let total_size: u64 = selected.iter().map(|(_, _, s)| s).sum();

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Confirm Deletion",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Items to delete: {}", selected.len())),
        Line::from(format!(
            "Total size: {}",
            crate::util::format::format_size(total_size)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This action cannot be undone!",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Y]", Style::default().fg(Color::Green)),
            Span::raw(" Confirm  "),
            Span::styled("[N]", Style::default().fg(Color::Red)),
            Span::raw(" Cancel"),
        ]),
    ];

    let dialog = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Delete Confirmation ")
                .border_style(Style::default().fg(Color::Red)),
        )
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });

    let dialog_area = centered_rect(50, 40, area);
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
