use slotmap::SlotMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::node::{FileCategory, NodeId, TreeNode};
use super::statistics::TreeStatistics;
use crate::app::SortMode;
use crate::scanner::IgnoreMatcher;

pub struct FileTree {
    arena: SlotMap<NodeId, TreeNode>,
    root: Option<NodeId>,
    path_index: HashMap<PathBuf, NodeId>,
    node_paths: HashMap<NodeId, PathBuf>,
    pub statistics: TreeStatistics,
    /// Number of nodes currently flagged `selected`. Maintained alongside
    /// `toggle_selection` / `clear_all_selections` / `remove` so callers that
    /// only need the count (e.g. the footer, rendered every tick) can avoid a
    /// full arena scan via `get_selected()`.
    selected_count: usize,
    /// Bumped on every structural mutation (insert/remove/set_root). Used to
    /// invalidate per-frame caches in `App` that key off the current node.
    version: u64,
}

impl FileTree {
    pub fn new() -> Self {
        Self {
            arena: SlotMap::with_key(),
            root: None,
            path_index: HashMap::new(),
            node_paths: HashMap::new(),
            statistics: TreeStatistics::new(),
            selected_count: 0,
            version: 0,
        }
    }

    /// Reset the tree to an empty state, dropping all nodes, indexes, and
    /// statistics. Used before starting a fresh scan on an existing tree so it
    /// can be reused across rescans without recreating the `App`.
    pub fn clear(&mut self) {
        self.arena.clear();
        self.root = None;
        self.path_index.clear();
        self.node_paths.clear();
        self.statistics = TreeStatistics::new();
        self.selected_count = 0;
        self.version = self.version.wrapping_add(1);
    }

    pub fn set_root(&mut self, path: &Path) -> NodeId {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let node = TreeNode::new_directory(name, None, 0);
        let id = self.arena.insert(node);
        self.root = Some(id);
        let root_path = path.to_path_buf();
        self.path_index.insert(root_path.clone(), id);
        self.node_paths.insert(id, root_path);
        self.statistics.add_directory();
        self.version = self.version.wrapping_add(1);
        id
    }

    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    pub fn get(&self, id: NodeId) -> Option<&TreeNode> {
        self.arena.get(id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut TreeNode> {
        self.arena.get_mut(id)
    }

    pub fn get_by_path(&self, path: &Path) -> Option<NodeId> {
        self.path_index.get(path).copied()
    }

    pub fn insert_entry(
        &mut self,
        path: &Path,
        parent_path: &Path,
        size: u64,
        is_dir: bool,
        modified: Option<SystemTime>,
    ) -> Option<NodeId> {
        let parent_id = self.path_index.get(parent_path).copied()?;

        // Check if the entry already exists (e.g., from populate_children_from_fs)
        if let Some(&existing_id) = self.path_index.get(path) {
            // Update existing node with size info
            if !is_dir {
                if let Some(node) = self.arena.get_mut(existing_id) {
                    let old_size = node.size;
                    let size_diff = size as i64 - old_size as i64;
                    node.size = size;

                    // Fill metadata if missing (populate_children_from_fs nodes lack it)
                    if node.metadata.is_none() {
                        node.metadata = Some(super::node::NodeMetadata {
                            modified,
                            created: None,
                            accessed: None,
                            is_symlink: false,
                        });
                    }

                    // Update statistics
                    let category = node.category();
                    let name = node.name.clone();
                    self.statistics.add_file(category, size, existing_id, name);

                    // Propagate size difference up to ancestors
                    if size_diff > 0 {
                        self.propagate_size(parent_id, size_diff as u64);
                    }
                }
            }
            return Some(existing_id);
        }

        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let parent_depth = self.arena.get(parent_id).map(|n| n.depth).unwrap_or(0);
        let depth = parent_depth + 1;

        let node = if is_dir {
            self.statistics.add_directory();
            TreeNode::new_directory(name.clone(), Some(parent_id), depth)
        } else {
            let ext = path
                .extension()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let category = FileCategory::from_extension(&ext);
            let node = TreeNode::new_file(name.clone(), size, Some(parent_id), depth, category);

            let id = self.arena.insert(node);
            self.version = self.version.wrapping_add(1);
            let entry_path = path.to_path_buf();
            self.path_index.insert(entry_path.clone(), id);
            self.node_paths.insert(id, entry_path);

            // Attach metadata (modified time)
            if let Some(n) = self.arena.get_mut(id) {
                n.metadata = Some(super::node::NodeMetadata {
                    modified,
                    created: None,
                    accessed: None,
                    is_symlink: false,
                });
            }

            // Add to parent
            if let Some(parent) = self.arena.get_mut(parent_id) {
                parent.children.push(id);
            }

            // Update statistics
            self.statistics.add_file(category, size, id, name);

            // Propagate size up to ancestors
            self.propagate_size(parent_id, size);

            return Some(id);
        };

        let id = self.arena.insert(node);
        self.version = self.version.wrapping_add(1);
        let entry_path = path.to_path_buf();
        self.path_index.insert(entry_path.clone(), id);
        self.node_paths.insert(id, entry_path);

        // Attach metadata (modified time) for directories too
        if let Some(n) = self.arena.get_mut(id) {
            n.metadata = Some(super::node::NodeMetadata {
                modified,
                created: None,
                accessed: None,
                is_symlink: false,
            });
        }

        // Add to parent
        if let Some(parent) = self.arena.get_mut(parent_id) {
            parent.children.push(id);
        }

        Some(id)
    }

    fn propagate_size(&mut self, mut current: NodeId, size: u64) {
        while let Some(node) = self.arena.get_mut(current) {
            node.size += size;
            node.item_count += 1;
            match node.parent {
                Some(parent) => current = parent,
                None => break,
            }
        }
    }

    pub fn get_children(&self, id: NodeId) -> Vec<(NodeId, &TreeNode)> {
        self.arena
            .get(id)
            .map(|node| {
                node.children
                    .iter()
                    .filter_map(|&child_id| self.arena.get(child_id).map(|n| (child_id, n)))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_children_sorted_by_size(&self, id: NodeId) -> Vec<(NodeId, &TreeNode)> {
        let mut children = self.get_children(id);
        children.sort_by_key(|b| std::cmp::Reverse(b.1.size));
        children
    }

    pub fn get_children_sorted(&self, id: NodeId, sort_mode: SortMode) -> Vec<(NodeId, &TreeNode)> {
        let mut children = self.get_children(id);
        match sort_mode {
            SortMode::Size => {
                children.sort_by_key(|b| std::cmp::Reverse(b.1.size));
            }
            SortMode::Name => {
                children.sort_by_key(|a| a.1.name.to_lowercase());
            }
            SortMode::Type => {
                children.sort_by(|a, b| {
                    // Directories first
                    let a_is_dir = a.1.is_dir();
                    let b_is_dir = b.1.is_dir();
                    match (a_is_dir, b_is_dir) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => {
                            // Same type - sort by extension then name
                            let a_ext = a.1.name.rsplit('.').next().unwrap_or("").to_lowercase();
                            let b_ext = b.1.name.rsplit('.').next().unwrap_or("").to_lowercase();
                            match a_ext.cmp(&b_ext) {
                                std::cmp::Ordering::Equal => {
                                    a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())
                                }
                                other => other,
                            }
                        }
                    }
                });
            }
            SortMode::Date => {
                children.sort_by(|a, b| {
                    let a_time = a.1.metadata.as_ref().and_then(|m| m.modified);
                    let b_time = b.1.metadata.as_ref().and_then(|m| m.modified);
                    match (a_time, b_time) {
                        (Some(a), Some(b)) => b.cmp(&a), // Newest first
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()),
                    }
                });
            }
        }
        children
    }

    pub fn toggle_selection(&mut self, id: NodeId) {
        if let Some(node) = self.arena.get_mut(id) {
            node.selected = !node.selected;
            if node.selected {
                self.selected_count += 1;
            } else {
                self.selected_count -= 1;
            }
        }
    }

    pub fn get_selected(&self) -> Vec<NodeId> {
        self.arena
            .iter()
            .filter(|(_, node)| node.selected)
            .map(|(id, _)| id)
            .collect()
    }

    /// O(1) count of currently selected nodes. Prefer this over
    /// `get_selected().len()` when only the count is needed (e.g. rendering).
    pub fn selected_count(&self) -> usize {
        self.selected_count
    }

    pub fn clear_all_selections(&mut self) {
        for (_, node) in self.arena.iter_mut() {
            node.selected = false;
        }
        self.selected_count = 0;
    }

    pub fn get_path(&self, id: NodeId) -> Option<PathBuf> {
        self.node_paths.get(&id).cloned()
    }

    pub fn remove(&mut self, id: NodeId) -> bool {
        if let Some(node) = self.arena.remove(id) {
            // Account for this node's selection state.
            if node.selected {
                self.selected_count = self.selected_count.saturating_sub(1);
            }

            // Remove from parent's children
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.arena.get_mut(parent_id) {
                    parent.children.retain(|child| *child != id);
                }
            }

            // Remove from both path indexes before recursing into children
            if let Some(path) = self.node_paths.remove(&id) {
                self.path_index.remove(&path);
            }

            // Recursively remove children (each recursive call updates
            // selected_count for its own node).
            for child_id in node.children.iter() {
                self.remove(*child_id);
            }

            self.version = self.version.wrapping_add(1);
            true
        } else {
            false
        }
    }

    /// Current structural-mutation version. Incremented on insert/remove/
    /// set_root so caches keyed on it can detect staleness.
    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn total_nodes(&self) -> usize {
        self.arena.len()
    }

    /// Check if a node's children have been populated
    pub fn has_children_populated(&self, id: NodeId) -> bool {
        self.arena
            .get(id)
            .map(|node| !node.children.is_empty() || node.children_populated)
            .unwrap_or(false)
    }

    /// Mark a node as having its children populated (even if empty)
    pub fn mark_children_populated(&mut self, id: NodeId) {
        if let Some(node) = self.arena.get_mut(id) {
            node.children_populated = true;
        }
    }

    /// Populate a directory's children by reading the filesystem directly
    /// This is used when navigating into a directory during scanning
    pub fn populate_children_from_fs(&mut self, id: NodeId) -> bool {
        self.populate_children_from_fs_with_filter(id, &IgnoreMatcher::default())
    }

    /// Populate a directory's children by reading the filesystem directly while respecting ignore rules
    pub fn populate_children_from_fs_with_filter(
        &mut self,
        id: NodeId,
        ignore_matcher: &IgnoreMatcher,
    ) -> bool {
        // Get the path for this node
        let path = match self.get_path(id) {
            Some(p) => p,
            None => return false,
        };

        // Check if already populated
        if self.has_children_populated(id) {
            return true;
        }

        // Read directory contents
        let entries = match std::fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => {
                self.mark_children_populated(id);
                return false;
            }
        };

        // Get parent depth
        let parent_depth = self.arena.get(id).map(|n| n.depth).unwrap_or(0);

        for entry in entries.filter_map(|e| e.ok()) {
            let entry_path = entry.path();
            if ignore_matcher.matches(&entry_path) {
                continue;
            }
            let name = entry_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            // Skip if already exists
            if self.path_index.contains_key(&entry_path) {
                continue;
            }

            let is_dir = entry_path.is_dir();
            let depth = parent_depth + 1;

            let node = if is_dir {
                TreeNode::new_directory(name.clone(), Some(id), depth)
            } else {
                // Get file size and modified time
                let metadata = entry.metadata();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = metadata.ok().and_then(|m| m.modified().ok());
                let ext = entry_path
                    .extension()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let category = FileCategory::from_extension(&ext);
                let mut node = TreeNode::new_file(name.clone(), size, Some(id), depth, category);
                node.metadata = Some(super::node::NodeMetadata {
                    modified,
                    created: None,
                    accessed: None,
                    is_symlink: false,
                });
                node
            };

            let child_id = self.arena.insert(node);
            self.version = self.version.wrapping_add(1);
            self.path_index.insert(entry_path.clone(), child_id);
            self.node_paths.insert(child_id, entry_path);

            // Add to parent
            if let Some(parent) = self.arena.get_mut(id) {
                parent.children.push(child_id);
            }
        }

        self.mark_children_populated(id);
        true
    }
}

impl Default for FileTree {
    fn default() -> Self {
        Self::new()
    }
}
