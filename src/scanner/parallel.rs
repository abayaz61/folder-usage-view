use crossbeam_channel::Sender;
use jwalk::WalkDir;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::entry::{ScanMessage, ScanProgress, ScanResult, ScannedEntry};

// Batch size for sending entries - larger = less overhead, but less responsive UI
const BATCH_SIZE: usize = 256;

// Progress update interval
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

pub struct ParallelScanner {
    follow_symlinks: bool,
    skip_hidden: bool,
}

impl ParallelScanner {
    pub fn new() -> Self {
        Self {
            follow_symlinks: false,
            skip_hidden: false,
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

        // Optimized walker configuration:
        // 1. sort(false) - Skip sorting for faster iteration
        // 2. RayonDefaultPool - Reuse existing thread pool (less overhead)
        // 3. process_read_dir - Pre-fetch metadata during directory read
        let walker = WalkDir::new(&root)
            .skip_hidden(self.skip_hidden)
            .follow_links(self.follow_symlinks)
            .sort(false)  // Optimization: Don't sort entries
            .parallelism(jwalk::Parallelism::RayonDefaultPool {
                busy_timeout: Duration::from_secs(5),
            })
            .process_read_dir(|_depth, _path, _read_dir_state, children| {
                // Optimization: Pre-fetch metadata for all children at once
                // This is more efficient than fetching one-by-one during iteration
                children.iter_mut().for_each(|dir_entry_result| {
                    if let Ok(dir_entry) = dir_entry_result {
                        // Access metadata here to cache it
                        // jwalk will reuse this cached metadata later
                        let _ = dir_entry.metadata();
                    }
                });
            });

        let mut last_progress = Instant::now();
        let mut batch: Vec<ScannedEntry> = Vec::with_capacity(BATCH_SIZE);
        let mut current_path: PathBuf = root.clone();

        for entry in walker {
            // Check cancellation
            if cancel_flag.load(Ordering::Relaxed) {
                break;
            }

            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    let is_dir = entry.file_type().is_dir();

                    // Get file size - use cached metadata from process_read_dir
                    let size = if is_dir {
                        0
                    } else {
                        entry.metadata().map(|m| m.len()).unwrap_or(0)
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

                    // Add to batch instead of sending immediately
                    batch.push(ScannedEntry {
                        path: path.clone(),
                        parent_path,
                        size,
                        is_dir,
                    });

                    current_path = path;

                    // Send batch when full
                    if batch.len() >= BATCH_SIZE {
                        if tx.send(ScanMessage::Batch(std::mem::replace(
                            &mut batch,
                            Vec::with_capacity(BATCH_SIZE),
                        ))).is_err() {
                            break;
                        }
                    }

                    // Send periodic progress
                    if last_progress.elapsed() >= PROGRESS_INTERVAL {
                        let elapsed = start.elapsed();
                        let files = files_scanned.load(Ordering::Relaxed);
                        let dirs = dirs_scanned.load(Ordering::Relaxed);
                        let total = total_size.load(Ordering::Relaxed);
                        let entries_per_second = (files + dirs) as f64 / elapsed.as_secs_f64();

                        let progress = ScanProgress {
                            files_scanned: files,
                            dirs_scanned: dirs,
                            total_size: total,
                            current_path: current_path.clone(),
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

        // Send remaining entries in batch
        if !batch.is_empty() {
            let _ = tx.send(ScanMessage::Batch(batch));
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
