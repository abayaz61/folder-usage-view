use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use disk_usage_analyzer::report::{
    build_large_file_report, write_large_file_report, CategoryReportRow, CleanupSuggestion,
    LargeFileCleanupReport, LargestFileRow, ScanReport,
};

fn temp_output_path(extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dua-cleanup-test-{}.{}", unique, extension))
}

fn sample_scan_report() -> ScanReport {
    ScanReport {
        scanned_path: "E:/repo".to_string(),
        generated_at: "2026-05-21T12:00:00Z".to_string(),
        total_size: 2_000_000_000,
        total_files: 100,
        total_dirs: 20,
        error_count: 0,
        duration_secs: 3.2,
        categories: vec![CategoryReportRow {
            name: "Code".to_string(),
            size: 500_000_000,
            percentage: 25.0,
            count: 40,
        }],
        largest_files: vec![
            LargestFileRow {
                path: "E:/repo/target/release/app.exe".to_string(),
                size: 800_000_000,
            },
            LargestFileRow {
                path: "E:/repo/node_modules/pkg/index.js".to_string(),
                size: 400_000_000,
            },
            LargestFileRow {
                path: "E:/repo/src/main.rs".to_string(),
                size: 1_000,
            },
        ],
    }
}

#[test]
fn builds_large_file_report_with_threshold_filter() {
    let report = build_large_file_report(&sample_scan_report(), 100_000_000);

    assert_eq!(report.threshold_bytes, 100_000_000);
    assert_eq!(report.large_files.len(), 2);
    assert!(report
        .large_files
        .iter()
        .all(|file| file.size >= 100_000_000));
}

#[test]
fn cleanup_suggestions_detect_common_safe_targets() {
    let report = build_large_file_report(&sample_scan_report(), 100_000_000);

    assert!(report.suggestions.iter().any(|item| {
        item.path.contains("/target/")
            && item.reason.contains("build")
            && item.estimated_reclaim_bytes == 800_000_000
    }));
    assert!(report.suggestions.iter().any(|item| {
        item.path.contains("/node_modules/")
            && item.reason.contains("dependency")
            && item.estimated_reclaim_bytes == 400_000_000
    }));
}

#[test]
fn writes_large_file_report_as_markdown() {
    let output_path = temp_output_path("md");
    let report = LargeFileCleanupReport {
        scanned_path: "E:/repo".to_string(),
        generated_at: "2026-05-21T12:00:00Z".to_string(),
        threshold_bytes: 100_000_000,
        large_files: vec![LargestFileRow {
            path: "E:/repo/target/release/app.exe".to_string(),
            size: 800_000_000,
        }],
        suggestions: vec![CleanupSuggestion {
            path: "E:/repo/target/release/app.exe".to_string(),
            reason: "Large build artifact".to_string(),
            estimated_reclaim_bytes: 800_000_000,
        }],
    };

    write_large_file_report(&output_path, &report).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# Large File Cleanup Report"));
    assert!(content.contains("## Large Files"));
    assert!(content.contains("## Cleanup Suggestions"));

    let _ = fs::remove_file(output_path);
}
