pub mod duplicates;
pub mod entry;
pub mod ignore;
pub mod parallel;

pub use duplicates::{find_duplicate_files, DuplicateGroup};
pub use entry::{ScanMessage, ScanProgress, ScanResult, ScannedEntry};
pub use ignore::{IgnoreMatcher, IgnorePreset};
pub use parallel::ParallelScanner;
