use crossbeam_channel::Sender;
use jwalk::WalkDir;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use super::entry::{ScanMessage, ScanProgress, ScanResult, ScannedEntry};
use super::ignore::IgnoreMatcher;

pub struct ParallelScanner {
    follow_symlinks: bool,
    skip_hidden: bool,
    ignore_matcher: IgnoreMatcher,
}

impl ParallelScanner {
    pub fn new() -> Self {
        Self {
            follow_symlinks: false,
            skip_hidden: false,
            ignore_matcher: IgnoreMatcher::default(),
        }
    }

    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    pub fn skip_hidden(mut self, skip: bool) -> Self {
        self.skip_hidden = skip;
        self
    }

    pub fn with_ignore_matcher(mut self, ignore_matcher: IgnoreMatcher) -> Self {
        self.ignore_matcher = ignore_matcher;
        self
    }

    pub fn scan(
        &self,
        root: PathBuf,
        tx: Sender<ScanMessage>,
        cancel_flag: Arc<AtomicBool>,
    ) -> anyhow::Result<()> {
        let start = Instant::now();

        let files_scanned = Arc::new(AtomicU64::new(0));
        let dirs_scanned = Arc::new(AtomicU64::new(0));
        let total_size = Arc::new(AtomicU64::new(0));
        let error_count = Arc::new(AtomicU64::new(0));

        let walker = WalkDir::new(&root)
            .skip_hidden(self.skip_hidden)
            .follow_links(self.follow_symlinks)
            .parallelism(jwalk::Parallelism::RayonNewPool(num_cpus::get()));

        let mut last_progress = Instant::now();
        let progress_interval = std::time::Duration::from_millis(100);

        for entry in walker {
            // Check cancellation
            if cancel_flag.load(Ordering::Relaxed) {
                break;
            }

            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if self.ignore_matcher.matches(&path) {
                        continue;
                    }
                    let is_dir = entry.file_type().is_dir();

                    // Get file size and modified time
                    let metadata = entry.metadata();
                    let (size, modified) = match &metadata {
                        Ok(m) => {
                            let size = if is_dir { 0 } else { m.len() };
                            let modified = m.modified().ok();
                            (size, modified)
                        }
                        Err(_) => (0u64, None),
                    };

                    // Update counters
                    if is_dir {
                        dirs_scanned.fetch_add(1, Ordering::Relaxed);
                    } else {
                        files_scanned.fetch_add(1, Ordering::Relaxed);
                        total_size.fetch_add(size, Ordering::Relaxed);
                    }

                    // Get parent path
                    let parent_path = path.parent().unwrap_or(Path::new("")).to_path_buf();

                    // Send entry
                    let scanned = ScannedEntry {
                        path: path.clone(),
                        parent_path,
                        size,
                        is_dir,
                        modified,
                    };

                    if tx.send(ScanMessage::Entry(scanned)).is_err() {
                        break;
                    }

                    // Send periodic progress
                    if last_progress.elapsed() >= progress_interval {
                        let elapsed = start.elapsed();
                        let files = files_scanned.load(Ordering::Relaxed);
                        let dirs = dirs_scanned.load(Ordering::Relaxed);
                        let total = total_size.load(Ordering::Relaxed);
                        let entries_per_second = (files + dirs) as f64 / elapsed.as_secs_f64();

                        let progress = ScanProgress {
                            files_scanned: files,
                            dirs_scanned: dirs,
                            total_size: total,
                            current_path: path,
                            elapsed,
                            entries_per_second,
                        };

                        let _ = tx.send(ScanMessage::Progress(progress));
                        last_progress = Instant::now();
                    }
                }
                Err(_e) => {
                    error_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // Send completion
        let result = ScanResult {
            total_files: files_scanned.load(Ordering::Relaxed),
            total_dirs: dirs_scanned.load(Ordering::Relaxed),
            total_size: total_size.load(Ordering::Relaxed),
            duration: start.elapsed(),
            error_count: error_count.load(Ordering::Relaxed) as usize,
        };

        let _ = tx.send(ScanMessage::Completed(result));

        Ok(())
    }
}

impl Default for ParallelScanner {
    fn default() -> Self {
        Self::new()
    }
}
