use anyhow::{anyhow, Result};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnorePreset {
    Build,
    Dependencies,
    System,
}

impl IgnorePreset {
    pub fn from_cli_value(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "build" => Ok(Self::Build),
            "deps" | "dependencies" => Ok(Self::Dependencies),
            "system" => Ok(Self::System),
            other => Err(anyhow!("Unsupported ignore preset: {}", other)),
        }
    }

    fn patterns(self) -> &'static [&'static str] {
        match self {
            Self::Build => &["target", "dist", "build", "out", ".next", "coverage"],
            Self::Dependencies => &["node_modules", ".pnpm-store", ".yarn", ".turbo"],
            Self::System => &[".git", ".cache", ".DS_Store", "Thumbs.db"],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IgnoreMatcher {
    segment_patterns: Vec<String>,
    path_patterns: Vec<String>,
}

impl IgnoreMatcher {
    pub fn from_inputs(patterns: &[String], presets: &[IgnorePreset]) -> Self {
        let mut all_patterns = patterns.to_vec();
        for preset in presets {
            all_patterns.extend(preset.patterns().iter().map(|pattern| (*pattern).to_string()));
        }

        let mut segment_patterns = Vec::new();
        let mut path_patterns = Vec::new();

        for pattern in all_patterns {
            let normalized = normalize_pattern(&pattern);
            if normalized.is_empty() {
                continue;
            }

            if normalized.contains('/') {
                path_patterns.push(normalized);
            } else {
                segment_patterns.push(normalized);
            }
        }

        Self {
            segment_patterns,
            path_patterns,
        }
    }

    pub fn matches(&self, path: &Path) -> bool {
        let normalized = normalize_path(path);
        if normalized.is_empty() {
            return false;
        }

        let segments: Vec<&str> = normalized.split('/').filter(|segment| !segment.is_empty()).collect();

        if self
            .segment_patterns
            .iter()
            .any(|pattern| segments.iter().any(|segment| segment.eq(pattern)))
        {
            return true;
        }

        self.path_patterns.iter().any(|pattern| {
            normalized == *pattern
                || normalized.ends_with(pattern)
                || normalized.contains(&format!("/{}/", pattern))
        })
    }
}

fn normalize_pattern(value: &str) -> String {
    value
        .trim()
        .replace('\\', "/")
        .trim_matches('/')
        .to_ascii_lowercase()
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_matches('/')
        .to_ascii_lowercase()
}
