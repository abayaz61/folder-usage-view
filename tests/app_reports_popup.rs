use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use disk_usage_analyzer::app::{App, AppMode, Config};
use disk_usage_analyzer::scanner::ScanResult;

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dua-popup-test-{}-{}", name, unique))
}

fn build_scanned_app(root: &std::path::Path) -> App {
    let config = Config::new(root.to_path_buf());
    let mut app = App::new(config);

    let root_id = app.tree.set_root(root);
    app.current_node = Some(root_id);

    let folder = root.join("target");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("a.bin"), b"same-content").unwrap();
    fs::write(folder.join("b.bin"), b"same-content").unwrap();

    app.tree.insert_entry(&folder, root, 0, true, None);
    app.tree
        .insert_entry(&folder.join("a.bin"), &folder, 12, false, None);
    app.tree
        .insert_entry(&folder.join("b.bin"), &folder, 12, false, None);

    app.scan_result = Some(ScanResult {
        total_files: 2,
        total_dirs: 2,
        total_size: 24,
        duration: Duration::from_secs(1),
        error_count: 0,
    });

    app
}

#[test]
fn reports_popup_opens_and_closes() {
    let temp_dir = temp_dir("open-close");
    fs::create_dir_all(&temp_dir).unwrap();
    let mut app = build_scanned_app(&temp_dir);

    app.open_reports_popup();
    assert_eq!(app.mode, AppMode::Reports);
    assert_eq!(app.reports_selected_index, 0);

    app.close_reports_popup();
    assert_eq!(app.mode, AppMode::Browsing);

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn reports_popup_selection_wraps() {
    let temp_dir = temp_dir("selection");
    fs::create_dir_all(&temp_dir).unwrap();
    let mut app = build_scanned_app(&temp_dir);

    app.open_reports_popup();
    app.move_reports_selection(1);
    assert_eq!(app.reports_selected_index, 1);
    app.move_reports_selection(10);
    assert_eq!(app.reports_selected_index, 2);
    app.move_reports_selection(1);
    assert_eq!(app.reports_selected_index, 0);

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn reports_popup_executes_selected_action() {
    let temp_dir = temp_dir("execute");
    fs::create_dir_all(&temp_dir).unwrap();
    let mut app = build_scanned_app(&temp_dir);

    app.open_reports_popup();
    app.move_reports_selection(2);
    let output = app.execute_selected_report_action().unwrap();

    assert!(output.exists());
    assert!(output.ends_with("reports\\duplicates.md") || output.ends_with("reports/duplicates.md"));
    assert_eq!(app.mode, AppMode::Browsing);

    let _ = fs::remove_dir_all(temp_dir);
}
