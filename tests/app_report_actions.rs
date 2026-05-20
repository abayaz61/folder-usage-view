use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use disk_usage_analyzer::app::{App, Config};
use disk_usage_analyzer::scanner::ScanResult;

fn build_scanned_app(root: &std::path::Path) -> App {
    let config = Config::new(root.to_path_buf());
    let mut app = App::new(config);

    let root_id = app.tree.set_root(root);
    app.current_node = Some(root_id);

    let folder = root.join("target");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("a.bin"), b"same-content").unwrap();
    fs::write(folder.join("b.bin"), b"same-content").unwrap();
    fs::write(root.join("main.log"), vec![b'x'; 2048]).unwrap();

    app.tree.insert_entry(&folder, root, 0, true);
    app.tree
        .insert_entry(&folder.join("a.bin"), &folder, 12, false);
    app.tree
        .insert_entry(&folder.join("b.bin"), &folder, 12, false);
    app.tree
        .insert_entry(&root.join("main.log"), root, 2048, false);

    app.scan_result = Some(ScanResult {
        total_files: 3,
        total_dirs: 2,
        total_size: 2072,
        duration: Duration::from_secs(1),
        error_count: 0,
    });

    app
}

fn temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dua-app-report-{}-{}", name, unique))
}

#[test]
fn app_exports_snapshot_report_to_default_reports_folder() {
    let temp_dir = temp_dir("snapshot");
    fs::create_dir_all(&temp_dir).unwrap();
    let mut app = build_scanned_app(&temp_dir);

    let output = app.export_snapshot_report().unwrap();

    assert!(output.exists());
    assert!(output.ends_with(".dua-reports\\snapshot.json") || output.ends_with(".dua-reports/snapshot.json"));
    assert!(app.message.as_deref().unwrap_or_default().contains("snapshot"));
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn app_exports_cleanup_report_to_default_reports_folder() {
    let temp_dir = temp_dir("cleanup");
    fs::create_dir_all(&temp_dir).unwrap();
    let mut app = build_scanned_app(&temp_dir);

    let output = app.export_cleanup_report(1).unwrap();

    assert!(output.exists());
    assert!(output.ends_with(".dua-reports\\cleanup.md") || output.ends_with(".dua-reports/cleanup.md"));
    assert!(app.message.as_deref().unwrap_or_default().contains("cleanup"));
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn app_exports_duplicate_report_to_default_reports_folder() {
    let temp_dir = temp_dir("duplicates");
    fs::create_dir_all(&temp_dir).unwrap();
    let mut app = build_scanned_app(&temp_dir);

    let output = app.export_duplicate_report(1).unwrap();

    assert!(output.exists());
    assert!(output.ends_with(".dua-reports\\duplicates.md") || output.ends_with(".dua-reports/duplicates.md"));
    assert!(app.message.as_deref().unwrap_or_default().contains("duplicate"));
    let _ = fs::remove_dir_all(temp_dir);
}
