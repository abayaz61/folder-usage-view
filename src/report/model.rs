use std::path::Path;

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::model::FileTree;
use crate::scanner::ScanResult;
use crate::scanner::DuplicateGroup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Json,
    Csv,
    Markdown,
}

impl ReportFormat {
    pub fn from_cli_value(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            "md" | "markdown" => Ok(Self::Markdown),
            other => Err(anyhow!("Unsupported export format: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryReportRow {
    pub name: String,
    pub size: u64,
    pub percentage: f64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargestFileRow {
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub scanned_path: String,
    pub generated_at: String,
    pub total_size: u64,
    pub total_files: u64,
    pub total_dirs: u64,
    pub error_count: usize,
    pub duration_secs: f64,
    pub categories: Vec<CategoryReportRow>,
    pub largest_files: Vec<LargestFileRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDiffRow {
    pub name: String,
    pub previous_size: u64,
    pub current_size: u64,
    pub size_delta: i64,
    pub previous_count: u64,
    pub current_count: u64,
    pub count_delta: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareReport {
    pub baseline_path: String,
    pub current_path: String,
    pub baseline_generated_at: String,
    pub current_generated_at: String,
    pub total_size_delta: i64,
    pub total_files_delta: i64,
    pub total_dirs_delta: i64,
    pub error_count_delta: i64,
    pub category_diffs: Vec<ReportDiffRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupSuggestion {
    pub path: String,
    pub reason: String,
    pub estimated_reclaim_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileCleanupReport {
    pub scanned_path: String,
    pub generated_at: String,
    pub threshold_bytes: u64,
    pub large_files: Vec<LargestFileRow>,
    pub suggestions: Vec<CleanupSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateFilesReport {
    pub scanned_path: String,
    pub generated_at: String,
    pub duplicate_group_count: usize,
    pub duplicate_file_count: usize,
    pub reclaimable_bytes: u64,
    pub groups: Vec<DuplicateGroup>,
}

impl ScanReport {
    pub fn from_scan(path: &Path, tree: &FileTree, result: &ScanResult) -> Self {
        let total_size = result.total_size.max(1);
        let mut categories: Vec<_> = tree
            .statistics
            .size_by_category
            .iter()
            .map(|(category, size)| CategoryReportRow {
                name: category.name().to_string(),
                size: *size,
                percentage: (*size as f64 / total_size as f64) * 100.0,
                count: tree
                    .statistics
                    .count_by_category
                    .get(category)
                    .copied()
                    .unwrap_or(0),
            })
            .collect();
        categories.sort_by(|a, b| b.size.cmp(&a.size));

        let mut largest_files: Vec<_> = tree
            .statistics
            .largest_files
            .iter()
            .take(20)
            .map(|(size, node_id, name)| LargestFileRow {
                path: tree
                    .get_path(*node_id)
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| name.clone()),
                size: *size,
            })
            .collect();
        largest_files.sort_by(|a, b| b.size.cmp(&a.size));

        Self {
            scanned_path: path.display().to_string(),
            generated_at: Utc::now().to_rfc3339(),
            total_size: result.total_size,
            total_files: result.total_files,
            total_dirs: result.total_dirs,
            error_count: result.error_count,
            duration_secs: result.duration.as_secs_f64(),
            categories,
            largest_files,
        }
    }
}

pub fn compare_reports(baseline: &ScanReport, current: &ScanReport) -> CompareReport {
    let mut category_names: Vec<String> = baseline
        .categories
        .iter()
        .map(|row| row.name.clone())
        .chain(current.categories.iter().map(|row| row.name.clone()))
        .collect();
    category_names.sort();
    category_names.dedup();

    let mut category_diffs = Vec::new();
    for name in category_names {
        let previous = baseline.categories.iter().find(|row| row.name == name);
        let current_row = current.categories.iter().find(|row| row.name == name);

        let previous_size = previous.map(|row| row.size).unwrap_or(0);
        let current_size = current_row.map(|row| row.size).unwrap_or(0);
        let previous_count = previous.map(|row| row.count).unwrap_or(0);
        let current_count = current_row.map(|row| row.count).unwrap_or(0);

        category_diffs.push(ReportDiffRow {
            name,
            previous_size,
            current_size,
            size_delta: current_size as i64 - previous_size as i64,
            previous_count,
            current_count,
            count_delta: current_count as i64 - previous_count as i64,
        });
    }

    category_diffs.sort_by(|a, b| b.current_size.cmp(&a.current_size));

    CompareReport {
        baseline_path: baseline.scanned_path.clone(),
        current_path: current.scanned_path.clone(),
        baseline_generated_at: baseline.generated_at.clone(),
        current_generated_at: current.generated_at.clone(),
        total_size_delta: current.total_size as i64 - baseline.total_size as i64,
        total_files_delta: current.total_files as i64 - baseline.total_files as i64,
        total_dirs_delta: current.total_dirs as i64 - baseline.total_dirs as i64,
        error_count_delta: current.error_count as i64 - baseline.error_count as i64,
        category_diffs,
    }
}

pub fn build_large_file_report(
    report: &ScanReport,
    threshold_bytes: u64,
) -> LargeFileCleanupReport {
    let large_files: Vec<LargestFileRow> = report
        .largest_files
        .iter()
        .filter(|file| file.size >= threshold_bytes)
        .cloned()
        .collect();

    let suggestions = large_files
        .iter()
        .filter_map(|file| build_cleanup_suggestion(file))
        .collect();

    LargeFileCleanupReport {
        scanned_path: report.scanned_path.clone(),
        generated_at: report.generated_at.clone(),
        threshold_bytes,
        large_files,
        suggestions,
    }
}

fn build_cleanup_suggestion(file: &LargestFileRow) -> Option<CleanupSuggestion> {
    let normalized = file.path.replace('\\', "/").to_ascii_lowercase();
    let reason = if normalized.contains("/target/") || normalized.contains("/build/") {
        Some("Large build artifact")
    } else if normalized.contains("/node_modules/") {
        Some("Large dependency directory file")
    } else if normalized.contains("/.cache/") || normalized.contains("/cache/") {
        Some("Large cache file")
    } else if normalized.ends_with(".log") || normalized.contains("/logs/") {
        Some("Large log file")
    } else {
        None
    }?;

    Some(CleanupSuggestion {
        path: file.path.clone(),
        reason: reason.to_string(),
        estimated_reclaim_bytes: file.size,
    })
}

pub fn build_duplicate_files_report(
    scanned_path: &str,
    groups: Vec<DuplicateGroup>,
) -> DuplicateFilesReport {
    let duplicate_group_count = groups.len();
    let duplicate_file_count = groups.iter().map(|group| group.files.len()).sum();
    let reclaimable_bytes = groups.iter().map(|group| group.wasted_bytes).sum();

    DuplicateFilesReport {
        scanned_path: scanned_path.to_string(),
        generated_at: Utc::now().to_rfc3339(),
        duplicate_group_count,
        duplicate_file_count,
        reclaimable_bytes,
        groups,
    }
}
