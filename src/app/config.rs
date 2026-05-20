use std::path::PathBuf;

use crate::scanner::{IgnoreMatcher, IgnorePreset};

#[derive(Debug, Clone)]
pub struct Config {
    pub target_path: PathBuf,
    pub read_only: bool,
    pub follow_symlinks: bool,
    pub show_hidden: bool,
    pub ignore_patterns: Vec<String>,
    pub ignore_presets: Vec<IgnorePreset>,
}

impl Config {
    pub fn new(target_path: PathBuf) -> Self {
        Self {
            target_path,
            read_only: true,
            follow_symlinks: false,
            show_hidden: false,
            ignore_patterns: Vec::new(),
            ignore_presets: Vec::new(),
        }
    }

    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    pub fn with_follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = follow;
        self
    }

    pub fn with_show_hidden(mut self, show: bool) -> Self {
        self.show_hidden = show;
        self
    }

    pub fn with_ignore_patterns(mut self, patterns: Vec<String>) -> Self {
        self.ignore_patterns = patterns;
        self
    }

    pub fn with_ignore_presets(mut self, presets: Vec<IgnorePreset>) -> Self {
        self.ignore_presets = presets;
        self
    }

    pub fn ignore_matcher(&self) -> IgnoreMatcher {
        IgnoreMatcher::from_inputs(&self.ignore_patterns, &self.ignore_presets)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}
