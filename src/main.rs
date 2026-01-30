use std::io;
use std::panic;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use disk_usage_analyzer::app::{load_last_location, save_last_location, App, Config, Settings, StartupLocation};
use disk_usage_analyzer::scanner::ParallelScanner;
use disk_usage_analyzer::ui::run_app;

fn setup_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);

        // Call original panic hook
        original_hook(panic_info);
    }));
}

/// Try to find a valid directory by walking up the path tree
/// Returns None if no valid directory found (should switch to Computer view)
fn find_valid_path(path: PathBuf) -> Option<PathBuf> {
    let mut current = path;

    // Try the path itself first
    if current.exists() && current.is_dir() {
        return Some(current);
    }

    // Walk up parent directories
    while let Some(parent) = current.parent() {
        let parent_buf = parent.to_path_buf();
        if parent_buf.as_os_str().is_empty() {
            break;
        }
        if parent_buf.exists() && parent_buf.is_dir() {
            return Some(parent_buf);
        }
        current = parent_buf;
    }

    // Try current directory as last resort
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.exists() && cwd.is_dir() {
            return Some(cwd);
        }
    }

    None
}

#[derive(Parser, Debug, Clone)]
#[command(name = "dua")]
#[command(author = "Codegen <abayaz61@gmail.com>")]
#[command(version)]
#[command(about = "Disk Usage Analyzer - Ultra high-performance disk usage analyzer with TUI", long_about = None)]
struct Args {
    /// Path to analyze (defaults to current directory)
    #[arg(short, long, default_value = ".")]
    path: PathBuf,

    /// Enable delete functionality (disabled by default for safety)
    #[arg(long, default_value = "false")]
    allow_delete: bool,

    /// Follow symbolic links
    #[arg(long, default_value = "false")]
    follow_symlinks: bool,

    /// Show hidden files and directories
    #[arg(long, default_value = "false")]
    show_hidden: bool,
}

fn main() -> anyhow::Result<()> {
    // Setup panic hook to restore terminal on crash
    setup_panic_hook();

    let args = Args::parse();

    // Safely load settings
    let settings = Settings::load();

    // Determine initial path based on settings
    let requested_path = if args.path == PathBuf::from(".") {
        match settings.startup_location {
            StartupLocation::LastLocation => {
                load_last_location().unwrap_or_else(|| {
                    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                })
            }
            StartupLocation::ComputerView => {
                // Will switch to computer view, but need a fallback path
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            }
            StartupLocation::CurrentFolder => {
                std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
            }
        }
    } else {
        args.path.clone()
    };

    // Try to find a valid path, walking up if needed
    let (mut target_path, start_in_computer_view) = match find_valid_path(requested_path.clone()) {
        Some(valid_path) => {
            // Canonicalize if possible
            let path = valid_path.canonicalize().unwrap_or(valid_path);
            let computer_view = settings.startup_location == StartupLocation::ComputerView;
            (path, computer_view)
        }
        None => {
            // No valid path found - start in computer view
            let fallback = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("C:\\"));
            (fallback, true)
        }
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut final_scan_result: Option<disk_usage_analyzer::scanner::ScanResult> = None;
    let mut first_run = true;

    // Main loop - allows rescanning when navigating to parent directory
    loop {
        // Create config - allow delete if either command line arg or settings allow it
        let allow_delete = args.allow_delete || settings.allow_delete;
        let config = Config::new(target_path.clone())
            .with_read_only(!allow_delete)
            .with_follow_symlinks(args.follow_symlinks)
            .with_show_hidden(args.show_hidden);

        // Create app
        let mut app = App::new(config);

        // Check if we should start in computer view
        if first_run && start_in_computer_view {
            app.open_computer_view();
            first_run = false;

            // Run the app without starting a scan
            let result = run_app(&mut terminal, &mut app);

            match result {
                Ok(Some(new_path)) => {
                    target_path = new_path;
                    let _ = save_last_location(&target_path);
                    continue;
                }
                Ok(None) => {
                    break;
                }
                Err(e) => {
                    let _ = disable_raw_mode();
                    let _ = execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    );
                    let _ = terminal.show_cursor();
                    return Err(e.into());
                }
            }
        }
        first_run = false;

        // Start scanner in background thread
        let tx = app.start_scan();
        let cancel_flag = Arc::clone(&app.cancel_flag);
        let scanner_path = target_path.clone();
        let follow_symlinks = args.follow_symlinks;
        let show_hidden = args.show_hidden;

        thread::spawn(move || {
            let scanner = ParallelScanner::new()
                .follow_symlinks(follow_symlinks)
                .skip_hidden(!show_hidden);

            if let Err(e) = scanner.scan(scanner_path, tx.clone(), cancel_flag) {
                let _ = tx.send(disk_usage_analyzer::scanner::ScanMessage::Error(e.to_string()));
            }
        });

        // Run the app
        let result = run_app(&mut terminal, &mut app);

        // Store scan result for final output
        final_scan_result = app.scan_result.take();

        match result {
            Ok(Some(new_path)) => {
                // Rescan requested - update target path and continue loop
                target_path = new_path;
                // Save location for next session
                let _ = save_last_location(&target_path);
                continue;
            }
            Ok(None) => {
                // Normal quit - save current location
                let _ = save_last_location(&target_path);
                break;
            }
            Err(e) => {
                // Restore terminal before returning error
                let _ = disable_raw_mode();
                let _ = execute!(
                    terminal.backend_mut(),
                    LeaveAlternateScreen,
                    DisableMouseCapture
                );
                let _ = terminal.show_cursor();
                return Err(e.into());
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Print final stats if scan completed
    if let Some(scan_result) = &final_scan_result {
        println!("\nScan completed:");
        println!(
            "  Total size: {}",
            disk_usage_analyzer::util::format::format_size(scan_result.total_size)
        );
        println!("  Files: {}", scan_result.total_files);
        println!("  Directories: {}", scan_result.total_dirs);
        println!("  Duration: {:.2}s", scan_result.duration.as_secs_f64());
        if scan_result.error_count > 0 {
            println!("  Errors: {}", scan_result.error_count);
        }
    }

    Ok(())
}
