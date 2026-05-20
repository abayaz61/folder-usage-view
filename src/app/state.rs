use crossbeam_channel::{Receiver, Sender};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::model::{DriveInfo, FileTree, NodeId, get_all_drives};
use crate::platform::get_platform_labels;
use crate::scanner::{ScanMessage, ScanProgress, ScanResult};
use crate::ui::theme::{Icons, Theme};

use super::config::Config;
use super::settings::Settings;

/// Cached results of expensive platform checks (registry queries, process spawns, file checks).
/// Refreshed only when the settings page opens or a setting is toggled.
#[derive(Debug, Clone, Default)]
pub struct SettingsCache {
    pub context_menu_registered: bool,
    pub path_registered: bool,
    pub start_menu_shortcut_exists: bool,
    pub desktop_shortcut_exists: bool,
    pub running_as_admin: bool,
    pub install_path: String,
    pub start_menu_path: String,
    pub desktop_path: String,
    pub current_font_name: String,
    pub current_font_size: u16,
    pub available_fonts: Vec<String>,
}

impl SettingsCache {
    pub fn refresh() -> Self {
        use super::settings::windows;
        let (font_name, font_size) = windows::get_console_font();
        Self {
            context_menu_registered: windows::is_context_menu_registered(),
            path_registered: windows::is_path_registered(),
            start_menu_shortcut_exists: windows::is_start_menu_shortcut_exists(),
            desktop_shortcut_exists: windows::is_desktop_shortcut_exists(),
            running_as_admin: windows::is_running_as_admin(),
            install_path: windows::get_install_path().display().to_string(),
            start_menu_path: windows::get_start_menu_path().display().to_string(),
            desktop_path: windows::get_desktop_path().display().to_string(),
            current_font_name: font_name,
            current_font_size: font_size,
            available_fonts: windows::get_available_fonts(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Scanning,
    Browsing,
    ComputerView,  // Root view showing all drives
    Help,
    About,
    Settings,
    DeleteConfirm,
    DriveSelect,
    Error,  // Error display mode
    Quitting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum ViewMode {
    #[default]
    Treemap,
    List,
    Split,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    Size,       // By size, largest first
    Name,       // Alphabetical A-Z
    Type,       // Directories first, then by extension
    Date,       // By modified date, newest first
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            SortMode::Size => SortMode::Name,
            SortMode::Name => SortMode::Type,
            SortMode::Type => SortMode::Date,
            SortMode::Date => SortMode::Size,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SortMode::Size => "SIZE",
            SortMode::Name => "NAME",
            SortMode::Type => "TYPE",
            SortMode::Date => "DATE",
        }
    }
}

pub struct App {
    pub config: Config,
    pub mode: AppMode,
    pub previous_mode: AppMode,
    pub view_mode: ViewMode,
    pub tree: FileTree,
    pub current_node: Option<NodeId>,
    pub selected_index: usize,
    pub parent_entry_selected: bool, // Whether ".." entry is selected
    pub scan_progress: Option<ScanProgress>,
    pub scan_result: Option<ScanResult>,
    pub scan_rx: Option<Receiver<ScanMessage>>,
    pub cancel_flag: Arc<AtomicBool>,
    pub message: Option<String>,
    pub navigation_stack: Vec<NodeId>,
    pub pending_rescan: Option<PathBuf>,
    // Drive selection
    pub drives: Vec<DriveInfo>,
    pub drive_selected_index: usize,
    // Computer view state
    pub in_computer_view: bool,
    // Settings
    pub settings: Settings,
    pub settings_selected_index: usize,
    pub settings_cache: SettingsCache,
    // Error handling
    pub error_message: Option<String>,
    // Sorting
    pub sort_mode: SortMode,
    // Admin restart flag
    pub pending_admin_restart: bool,
    // Font preview state (for revert on cancel)
    pub original_font_name: String,
    pub original_font_size: u16,
}

impl App {
    pub fn new(config: Config) -> Self {
        let settings = Settings::load();
        let view_mode = settings.view_mode;
        Self {
            config,
            mode: AppMode::Scanning,
            previous_mode: AppMode::Scanning,
            view_mode,
            tree: FileTree::new(),
            current_node: None,
            selected_index: 0,
            parent_entry_selected: false,
            scan_progress: None,
            scan_result: None,
            scan_rx: None,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            message: None,
            navigation_stack: Vec::new(),
            pending_rescan: None,
            drives: Vec::new(),
            drive_selected_index: 0,
            in_computer_view: false,
            settings,
            settings_selected_index: 0,
            settings_cache: SettingsCache::default(),
            error_message: None,
            sort_mode: SortMode::default(),
            pending_admin_restart: false,
            original_font_name: String::new(),
            original_font_size: 0,
        }
    }

    pub fn show_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.previous_mode = self.get_base_mode();
        self.mode = AppMode::Error;
    }

    pub fn dismiss_error(&mut self) {
        self.error_message = None;
        self.mode = self.previous_mode;
    }

    /// Get the current theme based on settings
    pub fn theme(&self) -> Theme {
        Theme::new(self.settings.color_palette)
    }

    /// Get the current icon set based on settings
    pub fn icons(&self) -> Icons {
        Icons::new(self.settings.use_ascii_icons)
    }

    pub fn start_scan(&mut self) -> Sender<ScanMessage> {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.scan_rx = Some(rx);
        self.mode = AppMode::Scanning;
        self.in_computer_view = false;

        // Initialize root
        let root_id = self.tree.set_root(&self.config.target_path);
        self.current_node = Some(root_id);

        // Immediately populate root's children for faster UI response
        self.tree.populate_children_from_fs(root_id);

        tx
    }

    pub fn process_scan_messages(&mut self) {
        if let Some(rx) = &self.scan_rx {
            // Process all pending messages
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    ScanMessage::Entry(entry) => {
                        self.tree.insert_entry(
                            &entry.path,
                            &entry.parent_path,
                            entry.size,
                            entry.is_dir,
                        );
                    }
                    ScanMessage::Progress(progress) => {
                        self.scan_progress = Some(progress);
                    }
                    ScanMessage::Completed(result) => {
                        self.scan_result = Some(result);
                        self.mode = AppMode::Browsing;
                        self.message = Some("Scan complete!".to_string());
                    }
                    ScanMessage::Error(err) => {
                        self.message = Some(format!("Error: {}", err));
                    }
                }
            }
        }
    }

    pub fn get_current_children(&self) -> Vec<(NodeId, String, u64, bool)> {
        if let Some(current) = self.current_node {
            self.tree
                .get_children_sorted(current, self.sort_mode)
                .into_iter()
                .map(|(id, node)| (id, node.name.clone(), node.size, node.is_dir()))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn cycle_sort_mode(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.message = Some(format!("Sort: {}", self.sort_mode.label()));
    }

    pub fn open_in_explorer(&mut self) {
        let path = if self.in_computer_view {
            // In computer view, open selected drive
            if self.drive_selected_index < self.drives.len() {
                self.drives[self.drive_selected_index].mount_point.clone()
            } else {
                return;
            }
        } else if let Some(current) = self.current_node {
            // Get current directory path
            if let Some(path) = self.tree.get_path(current) {
                path
            } else {
                self.config.target_path.clone()
            }
        } else {
            self.config.target_path.clone()
        };

        // Open in file manager (cross-platform)
        match open::that_detached(&path) {
            Ok(()) => self.message = Some(format!("Opened: {}", path.display())),
            Err(e) => self.message = Some(format!("Failed to open: {}", e)),
        }
    }

    /// Open the currently selected file/folder with the default system application
    pub fn open_selected_item(&mut self) {
        if self.in_computer_view {
            // In computer view, open selected drive
            if self.drive_selected_index < self.drives.len() {
                let path = self.drives[self.drive_selected_index].mount_point.clone();
                self.open_path_with_system(&path);
            }
            return;
        }

        // If ".." entry is selected, do nothing (it's not a real file)
        if self.parent_entry_selected {
            return;
        }

        let children = self.get_current_children();
        if self.selected_index < children.len() {
            let (child_id, _, _, _) = &children[self.selected_index];
            if let Some(path) = self.tree.get_path(*child_id) {
                self.open_path_with_system(&path);
            }
        }
    }

    /// Open a path with the default system application (cross-platform)
    fn open_path_with_system(&mut self, path: &PathBuf) {
        match open::that_detached(path) {
            Ok(()) => self.message = Some(format!("Opened: {}", path.display())),
            Err(e) => self.message = Some(format!("Failed to open: {}", e)),
        }
    }

    /// Navigate into directory. If open_files is true, also opens files with default app.
    pub fn navigate_into_ex(&mut self, open_files: bool) {
        if self.in_computer_view {
            // In computer view, select drive
            if self.drive_selected_index < self.drives.len() {
                let drive = &self.drives[self.drive_selected_index];
                let path = drive.mount_point.clone();
                self.pending_rescan = Some(path.clone());
                self.message = Some(format!("Opening: {}", path.display()));
                self.in_computer_view = false;
            }
            return;
        }

        // If ".." entry is selected, navigate back
        if self.parent_entry_selected {
            self.navigate_back();
            return;
        }

        let children = self.get_current_children();
        if self.selected_index < children.len() {
            let (child_id, _, _, is_dir) = &children[self.selected_index];
            if *is_dir {
                let child_id = *child_id;
                // Clear selections when navigating
                self.tree.clear_all_selections();
                // Directory - navigate into it
                if let Some(current) = self.current_node {
                    self.navigation_stack.push(current);
                }
                self.current_node = Some(child_id);
                self.selected_index = 0;
                self.parent_entry_selected = false;

                // Immediately populate children if not already done (for faster UI response)
                if self.is_scanning() {
                    self.tree.populate_children_from_fs(child_id);
                }
            } else if open_files {
                // File - open it with default application (only if open_files is true)
                self.open_selected_item();
            }
        }
    }

    /// Navigate into directory and open files (Enter key behavior)
    pub fn navigate_into(&mut self) {
        self.navigate_into_ex(true);
    }

    /// Navigate into directory only, don't open files (Arrow key behavior)
    pub fn navigate_into_dir_only(&mut self) {
        self.navigate_into_ex(false);
    }

    pub fn navigate_back(&mut self) {
        if self.in_computer_view {
            // Already at computer view, do nothing
            self.message = Some("Already at Computer view".to_string());
            return;
        }

        // Clear selections when navigating
        self.tree.clear_all_selections();

        // Reset parent entry selection
        self.parent_entry_selected = false;

        if let Some(parent_id) = self.navigation_stack.pop() {
            // Find index of current node in parent's children
            if let Some(current) = self.current_node {
                let parent_children = self.tree.get_children_sorted_by_size(parent_id);
                self.selected_index = parent_children
                    .iter()
                    .position(|(id, _)| *id == current)
                    .unwrap_or(0);
            }
            self.current_node = Some(parent_id);
        } else {
            // At root - try to go to parent directory or show computer view
            self.navigate_to_parent_directory();
        }
    }

    pub fn navigate_to_parent_directory(&mut self) {
        if let Some(parent_path) = self.config.target_path.parent() {
            // Check if parent is a drive root (e.g., C:\)
            let is_drive_root = parent_path.parent().is_none()
                || parent_path.to_string_lossy().ends_with(':')
                || parent_path.to_string_lossy() == "\\\\"
                || parent_path.as_os_str().is_empty();

            if is_drive_root {
                // Go to computer view instead
                self.open_computer_view();
            } else if parent_path.exists() && parent_path.is_dir() {
                self.pending_rescan = Some(parent_path.to_path_buf());
                self.message = Some(format!("Navigating to: {}", parent_path.display()));
            } else {
                self.open_computer_view();
            }
        } else {
            // No parent - show computer view
            self.open_computer_view();
        }
    }

    pub fn open_computer_view(&mut self) {
        self.drives = get_all_drives();
        self.drive_selected_index = 0;
        self.in_computer_view = true;
        self.mode = AppMode::ComputerView;

        // Try to select current drive
        for (i, drive) in self.drives.iter().enumerate() {
            if self.config.target_path.starts_with(&drive.mount_point) {
                self.drive_selected_index = i;
                break;
            }
        }

        self.message = Some("Computer - Select a drive".to_string());
    }

    pub fn close_computer_view(&mut self) {
        self.in_computer_view = false;
        self.mode = AppMode::Browsing;
    }

    pub fn refresh(&mut self) {
        let path = if let Some(current) = self.current_node {
            self.tree.get_path(current).unwrap_or_else(|| self.config.target_path.clone())
        } else {
            self.config.target_path.clone()
        };
        self.pending_rescan = Some(path);
    }

    pub fn take_pending_rescan(&mut self) -> Option<PathBuf> {
        self.pending_rescan.take()
    }

    pub fn reset_for_rescan(&mut self, new_path: PathBuf) {
        self.config.target_path = new_path;
        self.tree = FileTree::new();
        self.current_node = None;
        self.selected_index = 0;
        self.scan_progress = None;
        self.scan_result = None;
        self.navigation_stack.clear();
        self.cancel_flag = Arc::new(AtomicBool::new(false));
        self.in_computer_view = false;
    }

    pub fn move_selection(&mut self, delta: i32) {
        if self.in_computer_view {
            self.move_drive_selection(delta);
            return;
        }

        let children = self.get_current_children();
        // Always show ".." when browsing - either to go to parent folder or to ComputerView
        let has_parent = self.current_node.is_some() && !self.in_computer_view;

        if children.is_empty() && !has_parent {
            return;
        }

        // Total items including ".." entry
        let total_items = children.len() + if has_parent { 1 } else { 0 };
        if total_items == 0 {
            return;
        }

        // Calculate current effective position
        let current_pos = if has_parent && self.parent_entry_selected {
            0i32
        } else if has_parent {
            self.selected_index as i32 + 1
        } else {
            self.selected_index as i32
        };

        // Calculate new position with wrapping
        let new_pos = (current_pos + delta).rem_euclid(total_items as i32) as usize;

        // Update selection state
        if has_parent {
            if new_pos == 0 {
                self.parent_entry_selected = true;
                self.selected_index = 0;
            } else {
                self.parent_entry_selected = false;
                self.selected_index = new_pos - 1;
            }
        } else {
            self.parent_entry_selected = false;
            self.selected_index = new_pos;
        }
    }

    /// Move selection while toggling selection state (for Shift+Arrow multi-select)
    pub fn move_selection_with_select(&mut self, delta: i32) {
        if self.in_computer_view || self.config.read_only {
            self.move_selection(delta);
            return;
        }

        let children = self.get_current_children();
        let has_parent = self.current_node.is_some() && !self.in_computer_view;

        if children.is_empty() && !has_parent {
            return;
        }

        // Toggle selection on current item before moving (skip ".." entry)
        if !self.parent_entry_selected && self.selected_index < children.len() {
            let (child_id, _, _, _) = &children[self.selected_index];
            self.tree.toggle_selection(*child_id);
        }

        // Move to next position
        self.move_selection(delta);
    }

    pub fn toggle_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Treemap => ViewMode::List,
            ViewMode::List => ViewMode::Split,
            ViewMode::Split => ViewMode::Treemap,
        };
        // Save view mode preference
        self.settings.view_mode = self.view_mode;
        let _ = self.settings.save();
    }

    pub fn toggle_selection(&mut self) {
        if self.in_computer_view {
            return; // No selection in computer view
        }

        let children = self.get_current_children();
        if self.selected_index < children.len() {
            let (child_id, _, _, _) = &children[self.selected_index];
            self.tree.toggle_selection(*child_id);
        }
    }

    pub fn get_selected_for_deletion(&self) -> Vec<(NodeId, PathBuf, u64)> {
        self.tree
            .get_selected()
            .into_iter()
            .filter_map(|id| {
                let node = self.tree.get(id)?;
                let path = self.tree.get_path(id)?;
                Some((id, path, node.size))
            })
            .collect()
    }

    pub fn confirm_delete(&mut self) {
        if self.in_computer_view {
            self.message = Some("Cannot delete drives".to_string());
            return;
        }

        if !self.config.read_only {
            self.mode = AppMode::DeleteConfirm;
        } else {
            self.message = Some("Delete disabled in read-only mode".to_string());
        }
    }

    pub fn cancel_delete(&mut self) {
        self.mode = AppMode::Browsing;
    }

    /// Delete selected item directly (used with Delete key)
    pub fn delete_selected_item(&mut self) {
        if self.in_computer_view {
            self.message = Some("Cannot delete drives".to_string());
            return;
        }

        if self.config.read_only {
            self.message = Some("Delete disabled in read-only mode".to_string());
            return;
        }

        if self.parent_entry_selected {
            return; // Can't delete ".."
        }

        let children = self.get_current_children();
        if self.selected_index >= children.len() {
            return;
        }

        let (child_id, name, _, _) = &children[self.selected_index];
        let child_id = *child_id;
        let name = name.clone();

        // Mark item for deletion
        self.tree.toggle_selection(child_id);

        if self.settings.show_delete_confirmation {
            // Show confirmation dialog
            self.mode = AppMode::DeleteConfirm;
        } else {
            // Delete directly without confirmation
            let results = self.execute_delete(self.settings.delete_to_trash);
            let success_count = results.iter().filter(|(_, r)| r.is_ok()).count();
            if success_count > 0 {
                if self.settings.delete_to_trash {
                    let labels = get_platform_labels();
                    self.message = Some(format!("Moved to {}: {}", labels.trash_name, name));
                } else {
                    self.message = Some(format!("Deleted: {}", name));
                }
            } else {
                self.message = Some(format!("Failed to delete: {}", name));
            }
        }
    }

    pub fn execute_delete(&mut self, to_trash: bool) -> Vec<(PathBuf, Result<(), std::io::Error>)> {
        let selected = self.get_selected_for_deletion();
        let mut results = Vec::new();

        for (id, path, _) in selected {
            let result = if to_trash {
                Self::move_to_trash(&path)
            } else if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            };

            if result.is_ok() {
                self.tree.remove(id);
            }

            results.push((path, result));
        }

        self.mode = AppMode::Browsing;
        results
    }

    /// Move a file or directory to the Recycle Bin/Trash (cross-platform)
    fn move_to_trash(path: &PathBuf) -> Result<(), std::io::Error> {
        trash::delete(path).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Trash error: {}", e))
        })
    }

    pub fn toggle_help(&mut self) {
        self.mode = match self.mode {
            AppMode::Help => {
                if self.in_computer_view {
                    AppMode::ComputerView
                } else if self.is_scanning() {
                    AppMode::Scanning
                } else {
                    AppMode::Browsing
                }
            }
            _ => AppMode::Help,
        };
    }

    pub fn is_scanning(&self) -> bool {
        self.scan_rx.is_some() && self.scan_result.is_none()
    }

    // Drive selection methods
    pub fn open_drive_selector(&mut self) {
        self.drives = get_all_drives();
        self.drive_selected_index = 0;

        // Try to select current drive
        for (i, drive) in self.drives.iter().enumerate() {
            if self.config.target_path.starts_with(&drive.mount_point) {
                self.drive_selected_index = i;
                break;
            }
        }

        self.mode = AppMode::DriveSelect;
    }

    pub fn close_drive_selector(&mut self) {
        if self.in_computer_view {
            self.mode = AppMode::ComputerView;
        } else {
            self.mode = AppMode::Browsing;
        }
    }

    pub fn move_drive_selection(&mut self, delta: i32) {
        if self.drives.is_empty() {
            return;
        }
        let len = self.drives.len() as i32;
        let new_index = (self.drive_selected_index as i32 + delta).rem_euclid(len) as usize;
        self.drive_selected_index = new_index;
    }

    /// Move drive selection vertically in grid (by column count)
    pub fn move_drive_selection_vertical(&mut self, direction: i32, cols: usize) {
        if self.drives.is_empty() || cols == 0 {
            return;
        }
        let len = self.drives.len() as i32;
        let delta = direction * cols as i32;
        let new_index = (self.drive_selected_index as i32 + delta).rem_euclid(len) as usize;
        self.drive_selected_index = new_index;
    }

    pub fn select_drive(&mut self) {
        if self.drive_selected_index < self.drives.len() {
            let drive = &self.drives[self.drive_selected_index];
            let path = drive.mount_point.clone();
            self.pending_rescan = Some(path.clone());
            self.message = Some(format!("Switching to: {}", path.display()));
            self.mode = AppMode::Browsing;
            self.in_computer_view = false;
        }
    }

    pub fn refresh_drives(&mut self) {
        self.drives = get_all_drives();
        self.message = Some("Drive list refreshed".to_string());
    }

    pub fn get_total_disk_stats(&self) -> (u64, u64, u64) {
        let total: u64 = self.drives.iter().map(|d| d.total_space).sum();
        let used: u64 = self.drives.iter().map(|d| d.used_space).sum();
        let free: u64 = self.drives.iter().map(|d| d.available_space).sum();
        (total, used, free)
    }

    pub fn quit(&mut self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        self.mode = AppMode::Quitting;
    }

    pub fn should_quit(&self) -> bool {
        self.mode == AppMode::Quitting
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    // About methods
    pub fn open_about(&mut self) {
        self.previous_mode = self.get_base_mode();
        self.mode = AppMode::About;
    }

    pub fn close_about(&mut self) {
        self.mode = self.previous_mode;
    }

    // Settings methods
    pub fn open_settings(&mut self) {
        self.previous_mode = self.get_base_mode();
        self.settings_selected_index = 0;
        self.settings_cache = SettingsCache::refresh();
        // Store original font for revert-on-cancel
        self.original_font_name = self.settings.font_name.clone();
        self.original_font_size = self.settings.font_size;
        self.mode = AppMode::Settings;
    }

    /// Close settings and save all changes (including font)
    pub fn close_settings(&mut self) {
        let _ = self.settings.save();
        self.mode = self.previous_mode;
    }

    /// Cancel settings: revert font to original and close
    pub fn cancel_settings(&mut self) {
        // Revert font if it was changed
        if self.settings.font_name != self.original_font_name
            || self.settings.font_size != self.original_font_size
        {
            use super::settings::windows;
            let _ = windows::set_console_font(&self.original_font_name, self.original_font_size);
            self.settings.font_name = self.original_font_name.clone();
            self.settings.font_size = self.original_font_size;
            let _ = self.settings.save();
        }
        self.mode = self.previous_mode;
    }

    pub fn move_settings_selection(&mut self, delta: i32) {
        const SETTINGS_COUNT: usize = 14; // Number of settings options
        let new_index = (self.settings_selected_index as i32 + delta).rem_euclid(SETTINGS_COUNT as i32) as usize;
        self.settings_selected_index = new_index;
    }

    pub fn toggle_current_setting(&mut self) {
        use super::settings::{windows, StartupLocation};

        match self.settings_selected_index {
            0 => {
                // Toggle context menu
                if self.settings.context_menu_enabled {
                    match windows::unregister_context_menu() {
                        Ok(()) => {
                            self.settings.context_menu_enabled = false;
                            self.message = Some("Context menu removed".to_string());
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                } else {
                    match windows::register_context_menu() {
                        Ok(()) => {
                            self.settings.context_menu_enabled = true;
                            self.message = Some("Context menu registered".to_string());
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
            1 => {
                // Cycle startup location
                self.settings.startup_location = match self.settings.startup_location {
                    StartupLocation::LastLocation => StartupLocation::CurrentFolder,
                    StartupLocation::CurrentFolder => StartupLocation::ComputerView,
                    StartupLocation::ComputerView => StartupLocation::LastLocation,
                };
                self.message = Some(format!("Startup: {:?}", self.settings.startup_location));
            }
            2 => {
                // Toggle PATH registration
                if self.settings.path_registered {
                    match windows::unregister_from_path() {
                        Ok(()) => {
                            self.settings.path_registered = false;
                            self.message = Some("Removed from PATH".to_string());
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                } else {
                    match windows::register_to_path() {
                        Ok(()) => {
                            self.settings.path_registered = true;
                            self.message = Some("Registered to PATH".to_string());
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
            3 => {
                // Toggle Menu shortcut (Start Menu on Windows, Applications on Linux/macOS)
                let labels = get_platform_labels();
                if windows::is_start_menu_shortcut_exists() {
                    match windows::remove_start_menu_shortcut() {
                        Ok(()) => {
                            self.message = Some(format!("{} removed", labels.menu_shortcut));
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                } else {
                    match windows::create_start_menu_shortcut() {
                        Ok(()) => {
                            self.message = Some(format!("{} created", labels.menu_shortcut));
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
            4 => {
                // Toggle Desktop shortcut
                let labels = get_platform_labels();
                if windows::is_desktop_shortcut_exists() {
                    match windows::remove_desktop_shortcut() {
                        Ok(()) => {
                            self.message = Some(format!("{} removed", labels.desktop_shortcut));
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                } else {
                    match windows::create_desktop_shortcut() {
                        Ok(()) => {
                            self.message = Some(format!("{} created", labels.desktop_shortcut));
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
            5 => {
                // Toggle language
                self.settings.language = self.settings.language.next();
                self.message = Some(format!("Language: {}", self.settings.language.display_name()));
            }
            6 => {
                // Toggle color palette
                self.settings.color_palette = self.settings.color_palette.next();
                self.message = Some(format!("Theme: {}", self.settings.color_palette.name()));
            }
            7 => {
                // Toggle ASCII icons
                self.settings.use_ascii_icons = !self.settings.use_ascii_icons;
                let mode = if self.settings.use_ascii_icons { "ASCII" } else { "Unicode" };
                self.message = Some(format!("Icons: {}", mode));
            }
            8 => {
                // Toggle allow delete
                self.settings.allow_delete = !self.settings.allow_delete;
                // Also update config.read_only
                self.config.read_only = !self.settings.allow_delete;
                let mode = if self.settings.allow_delete { "Enabled" } else { "Disabled" };
                self.message = Some(format!("Delete: {}", mode));
            }
            9 => {
                // Toggle delete method (trash vs permanent)
                let labels = get_platform_labels();
                self.settings.delete_to_trash = !self.settings.delete_to_trash;
                let mode = if self.settings.delete_to_trash { labels.trash_name } else { "Permanent" };
                self.message = Some(format!("Delete method: {}", mode));
            }
            10 => {
                // Toggle delete confirmation
                self.settings.show_delete_confirmation = !self.settings.show_delete_confirmation;
                let mode = if self.settings.show_delete_confirmation { "Enabled" } else { "Disabled" };
                self.message = Some(format!("Delete confirmation: {}", mode));
            }
            11 => {
                // Toggle run as admin/root
                let labels = get_platform_labels();
                self.settings.run_as_admin = !self.settings.run_as_admin;
                let mode = if self.settings.run_as_admin { "Enabled" } else { "Disabled" };
                self.message = Some(format!("{}: {}", labels.admin_label, mode));

                // If enabled and not currently admin/root, we need to restart
                if self.settings.run_as_admin && !windows::is_running_as_admin() {
                    // Save first, then the main loop will handle the restart
                    let _ = self.settings.save();
                    self.pending_admin_restart = true;
                }
            }
            12 => {
                // Cycle font name
                let fonts = windows::get_available_fonts();
                if !fonts.is_empty() {
                    let current_idx = fonts.iter()
                        .position(|f| f == &self.settings.font_name)
                        .unwrap_or(0);
                    let next_idx = (current_idx + 1) % fonts.len();
                    self.settings.font_name = fonts[next_idx].clone();
                    match windows::set_console_font(&self.settings.font_name, self.settings.font_size) {
                        Ok(()) => self.message = Some(format!("Font: {}", self.settings.font_name)),
                        Err(e) => self.message = Some(format!("Error: {}", e)),
                    }
                }
            }
            13 => {
                // Cycle font size
                const SIZES: &[u16] = &[8, 10, 12, 14, 16, 18, 20, 24];
                let current_idx = SIZES.iter()
                    .position(|&s| s == self.settings.font_size)
                    .unwrap_or(4); // default to 16
                let next_idx = (current_idx + 1) % SIZES.len();
                self.settings.font_size = SIZES[next_idx];
                match windows::set_console_font(&self.settings.font_name, self.settings.font_size) {
                    Ok(()) => self.message = Some(format!("Font size: {}pt", self.settings.font_size)),
                    Err(e) => self.message = Some(format!("Error: {}", e)),
                }
            }
            _ => {}
        }
        // Save settings after change (except font — font saves on close_settings)
        if self.settings_selected_index != 12 && self.settings_selected_index != 13 {
            let _ = self.settings.save();
        }
        // Update only the changed cache fields instead of full refresh
        // (full refresh spawns cmd.exe for is_running_as_admin and freezes UI)
        match self.settings_selected_index {
            0 => self.settings_cache.context_menu_registered = windows::is_context_menu_registered(),
            2 => self.settings_cache.path_registered = windows::is_path_registered(),
            3 => {
                self.settings_cache.start_menu_shortcut_exists = windows::is_start_menu_shortcut_exists();
                self.settings_cache.start_menu_path = windows::get_start_menu_path().display().to_string();
            }
            4 => {
                self.settings_cache.desktop_shortcut_exists = windows::is_desktop_shortcut_exists();
                self.settings_cache.desktop_path = windows::get_desktop_path().display().to_string();
            }
            _ => {} // Font, language, palette etc. read directly from self.settings
        }
    }

    pub fn increase_font_size(&mut self) {
        use super::settings::windows;
        const SIZES: &[u16] = &[8, 10, 12, 14, 16, 18, 20, 24];
        let current_idx = SIZES.iter()
            .position(|&s| s == self.settings.font_size)
            .unwrap_or(4);
        if current_idx < SIZES.len() - 1 {
            self.settings.font_size = SIZES[current_idx + 1];
            match windows::set_console_font(&self.settings.font_name, self.settings.font_size) {
                Ok(()) => self.message = Some(format!("Font: {}pt", self.settings.font_size)),
                Err(e) => self.message = Some(format!("Error: {}", e)),
            }
            let _ = self.settings.save();
        }
    }

    pub fn decrease_font_size(&mut self) {
        use super::settings::windows;
        const SIZES: &[u16] = &[8, 10, 12, 14, 16, 18, 20, 24];
        let current_idx = SIZES.iter()
            .position(|&s| s == self.settings.font_size)
            .unwrap_or(4);
        if current_idx > 0 {
            self.settings.font_size = SIZES[current_idx - 1];
            match windows::set_console_font(&self.settings.font_name, self.settings.font_size) {
                Ok(()) => self.message = Some(format!("Font: {}pt", self.settings.font_size)),
                Err(e) => self.message = Some(format!("Error: {}", e)),
            }
            let _ = self.settings.save();
        }
    }

    fn get_base_mode(&self) -> AppMode {
        if self.in_computer_view {
            AppMode::ComputerView
        } else if self.is_scanning() {
            AppMode::Scanning
        } else {
            AppMode::Browsing
        }
    }
}
