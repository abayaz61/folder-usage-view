use std::path::PathBuf;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct ScannedEntry {
    pub path: PathBuf,
    pub parent_path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub modified: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct ScanProgress {
    pub files_scanned: u64,
    pub dirs_scanned: u64,
    pub total_size: u64,
    pub current_path: PathBuf,
    pub elapsed: Duration,
    pub entries_per_second: f64,
}

#[derive(Debug)]
pub struct ScanResult {
    pub total_files: u64,
    pub total_dirs: u64,
    pub total_size: u64,
    pub duration: Duration,
    pub error_count: usize,
}

#[derive(Debug)]
pub enum ScanMessage {
    Entry(ScannedEntry),
    Progress(ScanProgress),
    Completed(ScanResult),
    Error(String),
}
