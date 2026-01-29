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
use crate::ui::widgets::{ComputerViewWidget, DriveListWidget, FileListWidget, HelpWidget, StatsWidget, TreemapWidget};

const TICK_RATE: Duration = Duration::from_millis(16); // ~60 FPS

/// Result of run_app: None means quit, Some(path) means rescan with new path
pub fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<Option<PathBuf>> {
    let mut last_tick = Instant::now();

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
            render_ui(frame, app, area);
        })?;

        // Handle events with timeout
        let timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(app, key.code, key.modifiers);
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

fn handle_key_event(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    // Clear any existing message on key press
    app.clear_message();

    match app.mode {
        AppMode::Scanning => match key {
            KeyCode::Char('q') | KeyCode::Esc => app.quit(),
            _ => {}
        },
        AppMode::Browsing => match key {
            KeyCode::Char('q') | KeyCode::Esc => app.quit(),
            KeyCode::Char('?') | KeyCode::Char('h') => app.toggle_help(),
            KeyCode::Char('g') => app.open_drive_selector(),
            KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => app.navigate_into(),
            KeyCode::Backspace | KeyCode::Left => app.navigate_back(),
            KeyCode::Tab => app.toggle_view(),
            KeyCode::Char(' ') => app.toggle_selection(),
            KeyCode::Char('d') => app.confirm_delete(),
            KeyCode::PageUp => app.move_selection(-10),
            KeyCode::PageDown => app.move_selection(10),
            KeyCode::Home => app.selected_index = 0,
            KeyCode::End => {
                let count = app.get_current_children().len();
                if count > 0 {
                    app.selected_index = count - 1;
                }
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
            _ => {}
        },
        AppMode::Help => match key {
            KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('h') => {
                app.toggle_help()
            }
            _ => {}
        },
        AppMode::DeleteConfirm => match key {
            KeyCode::Char('y') | KeyCode::Enter => {
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
            KeyCode::Char('n') | KeyCode::Esc => app.cancel_delete(),
            _ => {}
        },
        AppMode::DriveSelect => match key {
            KeyCode::Esc | KeyCode::Char('q') => app.close_drive_selector(),
            KeyCode::Up | KeyCode::Char('k') => app.move_drive_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => app.move_drive_selection(1),
            KeyCode::Enter => app.select_drive(),
            KeyCode::Char('g') => {
                // Refresh drive list
                app.drives = get_all_drives();
                app.message = Some("Drive list refreshed".to_string());
            }
            _ => {}
        },
        AppMode::ComputerView => match key {
            KeyCode::Char('q') | KeyCode::Esc => app.quit(),
            KeyCode::Char('?') | KeyCode::Char('h') => app.toggle_help(),
            KeyCode::Up | KeyCode::Char('k') => app.move_drive_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => app.move_drive_selection(1),
            KeyCode::Enter => app.navigate_into(),
            KeyCode::Char('g') => {
                // Refresh drive list
                app.refresh_drives();
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
            _ => {}
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
        AppMode::DeleteConfirm => render_delete_confirm(frame, app, area),
        AppMode::DriveSelect => render_drive_selector(frame, app, area),
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
        " Folder Usage View - {} ",
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
        AppMode::Scanning => render_scanning(frame, app, area),
        AppMode::ComputerView => {
            let computer_view = ComputerViewWidget::new(app);
            frame.render_widget(computer_view, area);
        }
        _ => render_main_view(frame, app, area),
    }
}

fn render_scanning(frame: &mut ratatui::Frame, app: &App, area: Rect) {
    let progress = app.scan_progress.as_ref();

    let text = if let Some(p) = progress {
        let elapsed = format!("{:.1}s", p.elapsed.as_secs_f64());
        let speed = format!("{:.0} items/s", p.entries_per_second);

        vec![
            Line::from(""),
            Line::from(Span::styled(
                "Scanning...",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
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
            ]),
            Line::from(vec![
                Span::raw("Size: "),
                Span::styled(
                    crate::util::format::format_size(p.total_size),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::raw("Time: "),
                Span::styled(elapsed, Style::default().fg(Color::Magenta)),
                Span::raw("  Speed: "),
                Span::styled(speed, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                crate::util::format::truncate_path(
                    &p.current_path.display().to_string(),
                    area.width.saturating_sub(4) as usize,
                ),
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'q' to cancel",
                Style::default().fg(Color::DarkGray),
            )),
        ]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "Starting scan...",
                Style::default().fg(Color::Yellow),
            )),
        ]
    };

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Progress ")
                .border_style(Style::default().fg(super::theme::Theme::border_color())),
        )
        .alignment(ratatui::layout::Alignment::Center);

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
        "q:Quit ?:Help g:Drives Tab:View Enter:Open Backspace:Back [READ-ONLY]"
    } else {
        "q:Quit ?:Help g:Drives Tab:View Enter:Open Backspace:Back Space:Select d:Delete"
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
