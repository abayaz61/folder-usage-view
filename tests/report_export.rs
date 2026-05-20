use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use disk_usage_analyzer::report::{
    write_report, CategoryReportRow, ExportRequest, LargestFileRow, ReportFormat, ScanReport,
};

fn temp_output_path(extension: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dua-report-test-{}.{}", unique, extension))
}

fn sample_report() -> ScanReport {
    ScanReport {
        scanned_path: "E:/AI Projects/folder-usage-view".to_string(),
        generated_at: "2026-05-20T12:00:00Z".to_string(),
        total_size: 1024 * 1024 * 42,
        total_files: 120,
        total_dirs: 18,
        error_count: 1,
        duration_secs: 2.5,
        categories: vec![
            CategoryReportRow {
                name: "Code".to_string(),
                size: 1024 * 1024 * 20,
                percentage: 47.6,
                count: 64,
            },
            CategoryReportRow {
                name: "Other".to_string(),
                size: 1024 * 1024 * 22,
                percentage: 52.4,
                count: 56,
            },
        ],
        largest_files: vec![
            LargestFileRow {
                path: "E:/AI Projects/folder-usage-view/target/release/dua.exe".to_string(),
                size: 1024 * 1024 * 9,
            },
            LargestFileRow {
                path: "E:/AI Projects/folder-usage-view/screenshots/banner.png".to_string(),
                size: 1024 * 1024 * 2,
            },
        ],
    }
}

#[test]
fn writes_json_report_with_summary_and_rows() {
    let output_path = temp_output_path("json");
    let request = ExportRequest {
        output_path: output_path.clone(),
        format: ReportFormat::Json,
    };

    write_report(&request, &sample_report()).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("\"scanned_path\""));
    assert!(content.contains("\"categories\""));
    assert!(content.contains("\"largest_files\""));

    let _ = fs::remove_file(output_path);
}

#[test]
fn writes_csv_report_with_category_and_largest_sections() {
    let output_path = temp_output_path("csv");
    let request = ExportRequest {
        output_path: output_path.clone(),
        format: ReportFormat::Csv,
    };

    write_report(&request, &sample_report()).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("section,name,size,percentage,count,path"));
    assert!(content.contains("summary,total_files,120,,,"));
    assert!(content.contains("category,Code,20971520,47.60,64,"));
    assert!(content.contains("largest,,,,,E:/AI Projects/folder-usage-view/target/release/dua.exe"));

    let _ = fs::remove_file(output_path);
}

#[test]
fn writes_markdown_report_with_tables() {
    let output_path = temp_output_path("md");
    let request = ExportRequest {
        output_path: output_path.clone(),
        format: ReportFormat::Markdown,
    };

    write_report(&request, &sample_report()).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("# Disk Usage Report"));
    assert!(content.contains("## Category Breakdown"));
    assert!(content.contains("| Category | Size (bytes) | Percentage | File Count |"));
    assert!(content.contains("## Largest Files"));

    let _ = fs::remove_file(output_path);
}
