use std::io;
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

use folder_usage_view::app::{load_last_location, save_last_location, App, Config};
use folder_usage_view::scanner::ParallelScanner;
use folder_usage_view::ui::run_app;

#[derive(Parser, Debug, Clone)]
#[command(name = "folder-usage-view")]
#[command(author = "Developer")]
#[command(version = "0.1.0")]
#[command(about = "Ultra high-performance disk usage analyzer with TUI", long_about = None)]
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
    let args = Args::parse();

    // Check if user specified a path or using default
    let is_default_path = args.path == PathBuf::from(".");

    // Resolve path - try last location if using default
    let mut target_path = if is_default_path {
        // Try to load last location first
        load_last_location().unwrap_or_else(|| {
            args.path.canonicalize().unwrap_or(args.path.clone())
        })
    } else {
        args.path.canonicalize().unwrap_or(args.path.clone())
    };

    // Validate path exists
    if !target_path.exists() {
        // If last location doesn't exist, fall back to current directory
        if is_default_path {
            target_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        } else {
            eprintln!("Error: Path does not exist: {}", target_path.display());
            std::process::exit(1);
        }
    }

    if !target_path.is_dir() {
        eprintln!("Error: Path is not a directory: {}", target_path.display());
        std::process::exit(1);
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut final_scan_result: Option<folder_usage_view::scanner::ScanResult> = None;

    // Main loop - allows rescanning when navigating to parent directory
    loop {
        // Create config
        let config = Config::new(target_path.clone())
            .with_read_only(!args.allow_delete)
            .with_follow_symlinks(args.follow_symlinks)
            .with_show_hidden(args.show_hidden);

        // Create app
        let mut app = App::new(config);

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
                let _ = tx.send(folder_usage_view::scanner::ScanMessage::Error(e.to_string()));
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
            folder_usage_view::util::format::format_size(scan_result.total_size)
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
