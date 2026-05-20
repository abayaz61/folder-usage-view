use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use ratatui::DefaultTerminal;

use disk_usage_analyzer::app::{load_last_location, save_last_location, App, Config, Settings, StartupLocation};
use disk_usage_analyzer::app::settings::windows as settings_windows;
use disk_usage_analyzer::report::{
    build_large_file_report, compare_saved_reports, write_large_file_report, write_report,
    build_duplicate_files_report, write_duplicate_files_report, ExportRequest, ReportFormat,
    ScanReport,
};
use disk_usage_analyzer::scanner::{find_duplicate_files, IgnorePreset, ParallelScanner};
use disk_usage_analyzer::ui::run_app;

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

    /// Ignore a path segment or nested path (repeatable)
    #[arg(long = "ignore")]
    ignore_patterns: Vec<String>,

    /// Built-in ignore preset: build, dependencies, system (repeatable)
    #[arg(long = "ignore-preset")]
    ignore_presets: Vec<String>,

    /// Internal flag: already attempted admin elevation (prevents infinite loop)
    #[arg(long, hide = true, default_value = "false")]
    elevated: bool,

    /// Export scan summary to a file after completion
    #[arg(long)]
    export: Option<PathBuf>,

    /// Export format: json, csv, md
    #[arg(long, default_value = "json")]
    export_format: String,

    /// Compare the current scan with a previously exported JSON snapshot
    #[arg(long)]
    compare_with: Option<PathBuf>,

    /// Output path for comparison report
    #[arg(long)]
    compare_output: Option<PathBuf>,

    /// Save a Markdown report for large files and cleanup suggestions
    #[arg(long)]
    large_files_output: Option<PathBuf>,

    /// Large file threshold in megabytes for cleanup report
    #[arg(long, default_value_t = 100)]
    large_file_threshold_mb: u64,

    /// Save duplicate file report as Markdown
    #[arg(long)]
    duplicates_output: Option<PathBuf>,

    /// Minimum file size in kilobytes for duplicate scanning
    #[arg(long, default_value_t = 1)]
    duplicates_min_kb: u64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.compare_with.is_some() && args.export.is_none() {
        return Err(anyhow::anyhow!(
            "`--compare-with` kullanırken mevcut taramayi snapshot olarak kaydetmek için `--export` zorunludur"
        ));
    }

    if args.compare_with.is_some() && args.compare_output.is_none() {
        return Err(anyhow::anyhow!(
            "`--compare-with` kullanırken karşılaştırma dosyası için `--compare-output` zorunludur"
        ));
    }

    if args.compare_with.is_some() && !matches!(ReportFormat::from_cli_value(&args.export_format)?, ReportFormat::Json) {
        return Err(anyhow::anyhow!(
            "Karşılaştırma için `--export-format json` kullanılmalıdır"
        ));
    }

    // Safely load settings
    let settings = Settings::load();

    // Check if we need to restart as admin (only if not already attempted)
    let mut settings = settings;
    if settings.run_as_admin && !args.elevated && !settings_windows::is_running_as_admin() {
        // Restart with admin privileges
        if settings_windows::relaunch_as_admin_with_flag().is_ok() {
            // Exit current instance
            return Ok(());
        }
        // If relaunch failed, continue normally
    }

    // If we have --elevated flag but still not admin, user declined UAC - reset the setting
    if args.elevated && settings.run_as_admin && !settings_windows::is_running_as_admin() {
        settings.run_as_admin = false;
        let _ = settings.save();
    }

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
    let (target_path, start_in_computer_view) = match find_valid_path(requested_path.clone()) {
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

    // Apply saved font settings before terminal init
    let _ = settings_windows::set_console_font(&settings.font_name, settings.font_size);

    // Initialize terminal with ratatui (includes panic hook automatically)
    let mut terminal = ratatui::init();

    // Enable mouse capture
    execute!(io::stdout(), EnableMouseCapture)?;

    // Run the application
    let result = run_main_loop(&mut terminal, &args, &settings, target_path, start_in_computer_view);

    // Disable mouse capture before restore
    let _ = execute!(io::stdout(), DisableMouseCapture);

    // Restore terminal (handles raw mode, alternate screen, cursor)
    ratatui::restore();

    // Handle admin restart if needed
    if let Ok((true, _, _, _, _, _)) = &result {
        let current_settings = Settings::load();
        if current_settings.run_as_admin && !args.elevated && !settings_windows::is_running_as_admin() {
            let _ = settings_windows::relaunch_as_admin_with_flag();
            return Ok(());
        }
    }

    // Return result or print final stats
    match result {
        Ok((_, final_scan_result, exported_path, compare_output, large_files_output, duplicates_output)) => {
            if let Some(scan_result) = final_scan_result {
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
                if let Some(exported_path) = exported_path {
                    println!("  Exported report: {}", exported_path.display());
                }
                if let Some(compare_output) = compare_output {
                    println!("  Compare report: {}", compare_output.display());
                }
                if let Some(large_files_output) = large_files_output {
                    println!("  Large files report: {}", large_files_output.display());
                }
                if let Some(duplicates_output) = duplicates_output {
                    println!("  Duplicates report: {}", duplicates_output.display());
                }
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Main application loop - returns (should_check_admin_restart, final_scan_result, exported_report_path, compare_report_path, large_files_report_path, duplicates_report_path)
fn run_main_loop(
    terminal: &mut DefaultTerminal,
    args: &Args,
    settings: &Settings,
    mut target_path: PathBuf,
    start_in_computer_view: bool,
) -> anyhow::Result<(
    bool,
    Option<disk_usage_analyzer::scanner::ScanResult>,
    Option<PathBuf>,
    Option<PathBuf>,
    Option<PathBuf>,
    Option<PathBuf>,
)> {
    let mut final_scan_result: Option<disk_usage_analyzer::scanner::ScanResult>;
    let mut first_run = true;
    let parsed_ignore_presets = args
        .ignore_presets
        .iter()
        .map(|value| IgnorePreset::from_cli_value(value))
        .collect::<anyhow::Result<Vec<_>>>()?;

    // Main loop - allows rescanning when navigating to parent directory
    loop {
        // Create config - allow delete if either command line arg or settings allow it
        let allow_delete = args.allow_delete || settings.allow_delete;
        let config = Config::new(target_path.clone())
            .with_read_only(!allow_delete)
            .with_follow_symlinks(args.follow_symlinks)
            .with_show_hidden(args.show_hidden)
            .with_ignore_patterns(args.ignore_patterns.clone())
            .with_ignore_presets(parsed_ignore_presets.clone());

        // Create app
        let mut app = App::new(config);

        // Check if we should start in computer view
        if first_run && start_in_computer_view {
            app.open_computer_view();
            first_run = false;

            // Run the app without starting a scan
            let result = run_app(terminal, &mut app);

            match result {
                Ok(Some(new_path)) => {
                    target_path = new_path;
                    let _ = save_last_location(&target_path);
                    continue;
                }
                Ok(None) => {
                    return Ok((false, None, None, None, None, None));
                }
                Err(e) => {
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
        let ignore_matcher = app.config.ignore_matcher();

        thread::spawn(move || {
            let scanner = ParallelScanner::new()
                .follow_symlinks(follow_symlinks)
                .skip_hidden(!show_hidden)
                .with_ignore_matcher(ignore_matcher);

            if let Err(e) = scanner.scan(scanner_path, tx.clone(), cancel_flag) {
                let _ = tx.send(disk_usage_analyzer::scanner::ScanMessage::Error(e.to_string()));
            }
        });

        // Run the app
        let result = run_app(terminal, &mut app);

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
                let exported_path = if let (Some(scan_result), Some(export_path)) =
                    (final_scan_result.as_ref(), args.export.as_ref())
                {
                    let format = ReportFormat::from_cli_value(&args.export_format)?;
                    let report = ScanReport::from_scan(&target_path, &app.tree, scan_result);
                    let request = ExportRequest {
                        output_path: export_path.clone(),
                        format,
                    };
                    write_report(&request, &report)?;
                    Some(export_path.clone())
                } else {
                    None
                };
                let compare_output = if let (Some(compare_with), Some(compare_output), Some(exported_path)) =
                    (&args.compare_with, &args.compare_output, &exported_path)
                {
                    let compare = compare_saved_reports(compare_with, exported_path, compare_output)?;
                    println!("  Compare delta (size): {}", compare.total_size_delta);
                    Some(compare_output.clone())
                } else {
                    None
                };
                let large_files_output = if let (Some(scan_result), Some(output_path)) =
                    (final_scan_result.as_ref(), args.large_files_output.as_ref())
                {
                    let threshold_bytes = args.large_file_threshold_mb * 1024 * 1024;
                    let cleanup_report = build_large_file_report(
                        &ScanReport::from_scan(&target_path, &app.tree, scan_result),
                        threshold_bytes,
                    );
                    write_large_file_report(output_path, &cleanup_report)?;
                    Some(output_path.clone())
                } else {
                    None
                };
                let duplicates_output = if let Some(output_path) = args.duplicates_output.as_ref() {
                    let min_size = args.duplicates_min_kb * 1024;
                    let duplicates = find_duplicate_files(
                        &target_path,
                        &app.config.ignore_matcher(),
                        min_size,
                    )?;
                    let report = build_duplicate_files_report(
                        &target_path.display().to_string(),
                        duplicates,
                    );
                    write_duplicate_files_report(output_path, &report)?;
                    Some(output_path.clone())
                } else {
                    None
                };
                return Ok((
                    true,
                    final_scan_result,
                    exported_path,
                    compare_output,
                    large_files_output,
                    duplicates_output,
                ));
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
}
