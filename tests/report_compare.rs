use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use disk_usage_analyzer::report::{
    compare_reports, compare_saved_reports, load_report, write_compare_report, write_report,
    CategoryReportRow, CompareReport, ExportRequest, LargestFileRow, ReportDiffRow, ReportFormat,
    ScanReport,
};

fn temp_output_path(extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dua-compare-test-{}.{}", unique, extension))
}

fn baseline_report() -> ScanReport {
    ScanReport {
        scanned_path: "E:/repo".to_string(),
        generated_at: "2026-05-20T10:00:00Z".to_string(),
        total_size: 1000,
        total_files: 10,
        total_dirs: 3,
        error_count: 0,
        duration_secs: 1.0,
        categories: vec![
            CategoryReportRow {
                name: "Code".to_string(),
                size: 700,
                percentage: 70.0,
                count: 7,
            },
            CategoryReportRow {
                name: "Other".to_string(),
                size: 300,
                percentage: 30.0,
                count: 3,
            },
        ],
        largest_files: vec![LargestFileRow {
            path: "E:/repo/src/main.rs".to_string(),
            size: 300,
        }],
    }
}

fn current_report() -> ScanReport {
    ScanReport {
        scanned_path: "E:/repo".to_string(),
        generated_at: "2026-05-21T10:00:00Z".to_string(),
        total_size: 1300,
        total_files: 12,
        total_dirs: 4,
        error_count: 1,
        duration_secs: 1.4,
        categories: vec![
            CategoryReportRow {
                name: "Code".to_string(),
                size: 900,
                percentage: 69.2,
                count: 8,
            },
            CategoryReportRow {
                name: "Other".to_string(),
                size: 100,
                percentage: 7.7,
                count: 1,
            },
            CategoryReportRow {
                name: "Data".to_string(),
                size: 300,
                percentage: 23.1,
                count: 3,
            },
        ],
        largest_files: vec![LargestFileRow {
            path: "E:/repo/target/app.exe".to_string(),
            size: 600,
        }],
    }
}

#[test]
fn saves_and_loads_report_snapshot() {
    let output_path = temp_output_path("json");
    let request = ExportRequest {
        output_path: output_path.clone(),
        format: ReportFormat::Json,
    };

    write_report(&request, &baseline_report()).unwrap();
    let loaded = load_report(&output_path).unwrap();

    assert_eq!(loaded.scanned_path, "E:/repo");
    assert_eq!(loaded.total_size, 1000);
    assert_eq!(loaded.categories.len(), 2);

    let _ = fs::remove_file(output_path);
}

#[test]
fn compares_two_snapshots_and_reports_deltas() {
    let compare = compare_reports(&baseline_report(), &current_report());

    assert_eq!(compare.total_size_delta, 300);
    assert_eq!(compare.total_files_delta, 2);
    assert_eq!(compare.total_dirs_delta, 1);
    assert_eq!(compare.error_count_delta, 1);
    assert!(compare
        .category_diffs
        .iter()
        .any(|diff| diff.name == "Code" && diff.size_delta == 200));
    assert!(compare
        .category_diffs
        .iter()
        .any(|diff| diff.name == "Data" && diff.size_delta == 300));
}

#[test]
fn writes_compare_report_as_markdown() {
    let output_path = temp_output_path("md");
    let compare = CompareReport {
        baseline_path: "before.json".to_string(),
        current_path: "after.json".to_string(),
        baseline_generated_at: "2026-05-20T10:00:00Z".to_string(),
        current_generated_at: "2026-05-21T10:00:00Z".to_string(),
        total_size_delta: 300,
        total_files_delta: 2,
        total_dirs_delta: 1,
        error_count_delta: 1,
        category_diffs: vec![ReportDiffRow {
            name: "Code".to_string(),
            previous_size: 700,
            current_size: 900,
            size_delta: 200,
            previous_count: 7,
            current_count: 8,
            count_delta: 1,
        }],
    };

    write_compare_report(&output_path, &compare).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# Scan Comparison Report"));
    assert!(content.contains("before.json"));
    assert!(content.contains("| Category | Previous Size | Current Size | Delta |"));

    let _ = fs::remove_file(output_path);
}

#[test]
fn compares_saved_reports_and_writes_output_file() {
    let baseline_path = temp_output_path("baseline.json");
    let current_path = temp_output_path("current.json");
    let output_path = temp_output_path("compare.md");

    write_report(
        &ExportRequest {
            output_path: baseline_path.clone(),
            format: ReportFormat::Json,
        },
        &baseline_report(),
    )
    .unwrap();
    write_report(
        &ExportRequest {
            output_path: current_path.clone(),
            format: ReportFormat::Json,
        },
        &current_report(),
    )
    .unwrap();

    let compare = compare_saved_reports(&baseline_path, &current_path, &output_path).unwrap();
    let content = fs::read_to_string(&output_path).unwrap();

    assert_eq!(compare.total_size_delta, 300);
    assert!(content.contains("# Scan Comparison Report"));
    assert!(content.contains("Total Size Delta"));

    let _ = fs::remove_file(baseline_path);
    let _ = fs::remove_file(current_path);
    let _ = fs::remove_file(output_path);
}
