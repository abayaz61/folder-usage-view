use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use jwalk::WalkDir;
use rayon::prelude::*;
use sha2::{Digest, Sha256};

use super::ignore::IgnoreMatcher;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DuplicateGroup {
    pub size: u64,
    pub wasted_bytes: u64,
    pub files: Vec<String>,
}

pub fn find_duplicate_files(
    root: &Path,
    ignore_matcher: &IgnoreMatcher,
    min_size: u64,
) -> Result<Vec<DuplicateGroup>> {
    let mut candidates: HashMap<u64, Vec<String>> = HashMap::new();

    for entry in WalkDir::new(root) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = entry.path();
        if ignore_matcher.matches(&path) || entry.file_type().is_dir() {
            continue;
        }

        let size = match entry.metadata() {
            Ok(metadata) => metadata.len(),
            Err(_) => continue,
        };

        if size < min_size {
            continue;
        }

        candidates
            .entry(size)
            .or_default()
            .push(path.display().to_string());
    }

    // Only groups with more than one file of the same size can contain duplicates.
    let size_groups: Vec<(u64, Vec<String>)> = candidates
        .into_iter()
        .filter(|(_, files)| files.len() > 1)
        .collect();

    // Hash files in parallel (rayon). Files that fail to hash are skipped rather
    // than aborting the whole report.
    let hashed: Vec<(u64, String, String)> = size_groups
        .into_par_iter()
        .flat_map(|(size, files)| {
            files
                .into_par_iter()
                .filter_map(|path| {
                    let hash = hash_file(Path::new(&path)).ok()?;
                    Some((size, hash, path))
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let mut hashed_groups: HashMap<(u64, String), Vec<String>> = HashMap::new();
    for (size, hash, path) in hashed {
        hashed_groups
            .entry((size, hash))
            .or_default()
            .push(path);
    }

    let mut groups = Vec::new();
    for ((size, _hash), files) in hashed_groups {
        if files.len() < 2 {
            continue;
        }

        groups.push(DuplicateGroup {
            size,
            wasted_bytes: size.saturating_mul(files.len().saturating_sub(1) as u64),
            files,
        });
    }

    groups.sort_by_key(|b| std::cmp::Reverse(b.wasted_bytes));
    Ok(groups)
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
