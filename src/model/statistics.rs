use std::collections::HashMap;
use super::node::{FileCategory, NodeId};

#[derive(Debug, Default)]
pub struct TreeStatistics {
    pub size_by_category: HashMap<FileCategory, u64>,
    pub count_by_category: HashMap<FileCategory, u64>,
    pub largest_files: Vec<(u64, NodeId, String)>,
    pub total_files: u64,
    pub total_dirs: u64,
    pub total_size: u64,
}

impl TreeStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(&mut self, category: FileCategory, size: u64, node_id: NodeId, name: String) {
        *self.size_by_category.entry(category).or_insert(0) += size;
        *self.count_by_category.entry(category).or_insert(0) += 1;
        self.total_files += 1;
        self.total_size += size;

        // Maintain top 100 largest files
        const TOP_N: usize = 100;
        if self.largest_files.len() < TOP_N {
            self.largest_files.push((size, node_id, name));
            self.largest_files.sort_by_key(|b| std::cmp::Reverse(b.0));
        } else if size > self.largest_files.last().map(|x| x.0).unwrap_or(0) {
            self.largest_files.pop();
            self.largest_files.push((size, node_id, name));
            self.largest_files.sort_by_key(|b| std::cmp::Reverse(b.0));
        }
    }

    pub fn add_directory(&mut self) {
        self.total_dirs += 1;
    }

    pub fn get_category_percentages(&self) -> Vec<(FileCategory, f64, u64)> {
        if self.total_size == 0 {
            return Vec::new();
        }

        let mut result: Vec<_> = self
            .size_by_category
            .iter()
            .map(|(&cat, &size)| {
                let percentage = (size as f64 / self.total_size as f64) * 100.0;
                (cat, percentage, size)
            })
            .collect();

        result.sort_by_key(|b| std::cmp::Reverse(b.2));
        result
    }
}
