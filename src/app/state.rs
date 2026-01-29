use crossbeam_channel::{Receiver, Sender};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::model::{DriveInfo, FileTree, NodeId, get_all_drives};
use crate::scanner::{ScanMessage, ScanProgress, ScanResult};

use super::config::Config;
use super::settings::Settings;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Treemap,
    List,
    Split,
}

pub struct App {
    pub config: Config,
    pub mode: AppMode,
    pub previous_mode: AppMode,
    pub view_mode: ViewMode,
    pub tree: FileTree,
    pub current_node: Option<NodeId>,
    pub selected_index: usize,
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
    // Error handling
    pub error_message: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let settings = Settings::load();
        Self {
            config,
            mode: AppMode::Scanning,
            previous_mode: AppMode::Scanning,
            view_mode: ViewMode::Split,
            tree: FileTree::new(),
            current_node: None,
            selected_index: 0,
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
            error_message: None,
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

    pub fn start_scan(&mut self) -> Sender<ScanMessage> {
        let (tx, rx) = crossbeam_channel::unbounded();
        self.scan_rx = Some(rx);
        self.mode = AppMode::Scanning;
        self.in_computer_view = false;

        // Initialize root
        let root_id = self.tree.set_root(&self.config.target_path);
        self.current_node = Some(root_id);

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
                .get_children_sorted_by_size(current)
                .into_iter()
                .map(|(id, node)| (id, node.name.clone(), node.size, node.is_dir()))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn navigate_into(&mut self) {
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

        let children = self.get_current_children();
        if self.selected_index < children.len() {
            let (child_id, _, _, is_dir) = &children[self.selected_index];
            if *is_dir {
                if let Some(current) = self.current_node {
                    self.navigation_stack.push(current);
                }
                self.current_node = Some(*child_id);
                self.selected_index = 0;
            }
        }
    }

    pub fn navigate_back(&mut self) {
        if self.in_computer_view {
            // Already at computer view, do nothing
            self.message = Some("Already at Computer view".to_string());
            return;
        }

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
        if children.is_empty() {
            return;
        }

        let len = children.len() as i32;
        let new_index = (self.selected_index as i32 + delta).rem_euclid(len) as usize;
        self.selected_index = new_index;
    }

    pub fn toggle_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Treemap => ViewMode::List,
            ViewMode::List => ViewMode::Split,
            ViewMode::Split => ViewMode::Treemap,
        };
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

    pub fn execute_delete(&mut self) -> Vec<(PathBuf, Result<(), std::io::Error>)> {
        let selected = self.get_selected_for_deletion();
        let mut results = Vec::new();

        for (id, path, _) in selected {
            let result = if path.is_dir() {
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
        self.mode = AppMode::Settings;
    }

    pub fn close_settings(&mut self) {
        self.mode = self.previous_mode;
    }

    pub fn move_settings_selection(&mut self, delta: i32) {
        const SETTINGS_COUNT: usize = 5; // Number of settings options
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
                // Toggle Start Menu shortcut
                if windows::is_start_menu_shortcut_exists() {
                    match windows::remove_start_menu_shortcut() {
                        Ok(()) => {
                            self.message = Some("Start Menu shortcut removed".to_string());
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                } else {
                    match windows::create_start_menu_shortcut() {
                        Ok(()) => {
                            self.message = Some("Start Menu shortcut created".to_string());
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
            4 => {
                // Toggle Desktop shortcut
                if windows::is_desktop_shortcut_exists() {
                    match windows::remove_desktop_shortcut() {
                        Ok(()) => {
                            self.message = Some("Desktop shortcut removed".to_string());
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                } else {
                    match windows::create_desktop_shortcut() {
                        Ok(()) => {
                            self.message = Some("Desktop shortcut created".to_string());
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                }
            }
            _ => {}
        }
        // Save settings after change
        let _ = self.settings.save();
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
