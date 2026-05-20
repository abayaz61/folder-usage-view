use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::model::{
    CompareReport, DuplicateFilesReport, LargeFileCleanupReport, ReportFormat, ScanReport,
};

#[derive(Debug, Clone)]
pub struct ExportRequest {
    pub output_path: PathBuf,
    pub format: ReportFormat,
}

pub fn write_report(request: &ExportRequest, report: &ScanReport) -> Result<()> {
    if let Some(parent) = request.output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let content = match request.format {
        ReportFormat::Json => serde_json::to_string_pretty(report)?,
        ReportFormat::Csv => render_csv(report),
        ReportFormat::Markdown => render_markdown(report),
    };

    fs::write(&request.output_path, content)?;
    Ok(())
}

pub fn load_report(path: &Path) -> Result<ScanReport> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

pub fn write_compare_report(path: &Path, compare: &CompareReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(path, render_compare_markdown(compare))?;
    Ok(())
}

pub fn compare_saved_reports(
    baseline_path: &Path,
    current_path: &Path,
    output_path: &Path,
) -> Result<CompareReport> {
    let baseline = load_report(baseline_path)?;
    let current = load_report(current_path)?;
    let compare = super::model::compare_reports(&baseline, &current);
    write_compare_report(output_path, &compare)?;
    Ok(compare)
}

pub fn write_large_file_report(path: &Path, report: &LargeFileCleanupReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(path, render_large_file_markdown(report))?;
    Ok(())
}

pub fn write_duplicate_files_report(path: &Path, report: &DuplicateFilesReport) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(path, render_duplicate_files_markdown(report))?;
    Ok(())
}

fn render_csv(report: &ScanReport) -> String {
    let mut lines = vec!["section,name,size,percentage,count,path".to_string()];
    lines.push(format!(
        "summary,total_size,{},{},,",
        report.total_size, 100.0
    ));
    lines.push(format!(
        "summary,total_files,{0},,,",
        report.total_files
    ));
    lines.push(format!(
        "summary,total_dirs,{0},,,",
        report.total_dirs
    ));
    lines.push(format!(
        "summary,error_count,{0},,,",
        report.error_count
    ));
    lines.push(format!(
        "summary,duration_secs,{:.2},,,",
        report.duration_secs
    ));

    for category in &report.categories {
        lines.push(format!(
            "category,{},{},{:.2},{},",
            escape_csv(&category.name),
            category.size,
            category.percentage,
            category.count
        ));
    }

    for file in &report.largest_files {
        lines.push(format!(
            "largest,,,,,{}",
            escape_csv(&file.path)
        ));
    }

    lines.join("\n")
}

fn render_markdown(report: &ScanReport) -> String {
    let mut content = String::new();
    content.push_str("# Disk Usage Report\n\n");
    content.push_str(&format!("- Scanned Path: `{}`\n", report.scanned_path));
    content.push_str(&format!("- Generated At: `{}`\n", report.generated_at));
    content.push_str(&format!("- Total Size: `{}` bytes\n", report.total_size));
    content.push_str(&format!("- Total Files: `{}`\n", report.total_files));
    content.push_str(&format!("- Total Directories: `{}`\n", report.total_dirs));
    content.push_str(&format!("- Error Count: `{}`\n", report.error_count));
    content.push_str(&format!("- Duration: `{:.2}` seconds\n\n", report.duration_secs));

    content.push_str("## Category Breakdown\n\n");
    content.push_str("| Category | Size (bytes) | Percentage | File Count |\n");
    content.push_str("| --- | ---: | ---: | ---: |\n");
    for category in &report.categories {
        content.push_str(&format!(
            "| {} | {} | {:.2}% | {} |\n",
            category.name, category.size, category.percentage, category.count
        ));
    }

    content.push_str("\n## Largest Files\n\n");
    content.push_str("| Path | Size (bytes) |\n");
    content.push_str("| --- | ---: |\n");
    for file in &report.largest_files {
        content.push_str(&format!("| `{}` | {} |\n", file.path, file.size));
    }

    content
}

fn escape_csv(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn render_compare_markdown(compare: &CompareReport) -> String {
    let mut content = String::new();
    content.push_str("# Scan Comparison Report\n\n");
    content.push_str(&format!("- Baseline Path: `{}`\n", compare.baseline_path));
    content.push_str(&format!("- Current Path: `{}`\n", compare.current_path));
    content.push_str(&format!(
        "- Baseline Generated At: `{}`\n",
        compare.baseline_generated_at
    ));
    content.push_str(&format!(
        "- Current Generated At: `{}`\n",
        compare.current_generated_at
    ));
    content.push_str(&format!("- Total Size Delta: `{}` bytes\n", compare.total_size_delta));
    content.push_str(&format!("- Total Files Delta: `{}`\n", compare.total_files_delta));
    content.push_str(&format!("- Total Directories Delta: `{}`\n", compare.total_dirs_delta));
    content.push_str(&format!("- Error Count Delta: `{}`\n\n", compare.error_count_delta));

    content.push_str("## Category Deltas\n\n");
    content.push_str("| Category | Previous Size | Current Size | Delta | Previous Count | Current Count | Count Delta |\n");
    content.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for diff in &compare.category_diffs {
        content.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n",
            diff.name,
            diff.previous_size,
            diff.current_size,
            diff.size_delta,
            diff.previous_count,
            diff.current_count,
            diff.count_delta
        ));
    }

    content
}

fn render_large_file_markdown(report: &LargeFileCleanupReport) -> String {
    let mut content = String::new();
    content.push_str("# Large File Cleanup Report\n\n");
    content.push_str(&format!("- Scanned Path: `{}`\n", report.scanned_path));
    content.push_str(&format!("- Generated At: `{}`\n", report.generated_at));
    content.push_str(&format!(
        "- Threshold: `{}` bytes\n\n",
        report.threshold_bytes
    ));

    content.push_str("## Large Files\n\n");
    content.push_str("| Path | Size (bytes) |\n");
    content.push_str("| --- | ---: |\n");
    for file in &report.large_files {
        content.push_str(&format!("| `{}` | {} |\n", file.path, file.size));
    }

    content.push_str("\n## Cleanup Suggestions\n\n");
    content.push_str("| Path | Reason | Estimated Reclaim |\n");
    content.push_str("| --- | --- | ---: |\n");
    for item in &report.suggestions {
        content.push_str(&format!(
            "| `{}` | {} | {} |\n",
            item.path, item.reason, item.estimated_reclaim_bytes
        ));
    }

    content
}

fn render_duplicate_files_markdown(report: &DuplicateFilesReport) -> String {
    let mut content = String::new();
    content.push_str("# Duplicate Files Report\n\n");
    content.push_str("## Summary\n\n");
    content.push_str(&format!("- Scanned Path: `{}`\n", report.scanned_path));
    content.push_str(&format!("- Generated At: `{}`\n", report.generated_at));
    content.push_str(&format!(
        "- Duplicate Groups: `{}`\n",
        report.duplicate_group_count
    ));
    content.push_str(&format!(
        "- Duplicate Files: `{}`\n",
        report.duplicate_file_count
    ));
    content.push_str(&format!(
        "- Reclaimable Bytes: `{}`\n\n",
        report.reclaimable_bytes
    ));

    content.push_str("## Duplicate Groups\n\n");
    for (index, group) in report.groups.iter().enumerate() {
        content.push_str(&format!(
            "### Group {}\n\n- File Size: `{}` bytes\n- Wasted Bytes: `{}`\n\n",
            index + 1,
            group.size,
            group.wasted_bytes
        ));
        for file in &group.files {
            content.push_str(&format!("- `{}`\n", file));
        }
        content.push('\n');
    }

    content
}
