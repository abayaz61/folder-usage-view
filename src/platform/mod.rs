//! Platform-specific implementations for Windows, Linux, and macOS

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
pub use windows::*;
#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "macos")]
pub use macos::*;

/// Platform-agnostic trait for platform operations
pub trait PlatformOps {
    /// Check if context menu is registered
    fn is_context_menu_registered() -> bool;

    /// Register context menu integration
    fn register_context_menu() -> Result<(), String>;

    /// Unregister context menu integration
    fn unregister_context_menu() -> Result<(), String>;

    /// Check if PATH is registered
    fn is_path_registered() -> bool;

    /// Register to system PATH
    fn register_to_path() -> Result<(), String>;

    /// Unregister from system PATH
    fn unregister_from_path() -> Result<(), String>;

    /// Check if menu/application shortcut exists
    fn is_menu_shortcut_exists() -> bool;

    /// Create menu/application shortcut
    fn create_menu_shortcut() -> Result<(), String>;

    /// Remove menu/application shortcut
    fn remove_menu_shortcut() -> Result<(), String>;

    /// Check if desktop shortcut exists
    fn is_desktop_shortcut_exists() -> bool;

    /// Create desktop shortcut
    fn create_desktop_shortcut() -> Result<(), String>;

    /// Remove desktop shortcut
    fn remove_desktop_shortcut() -> Result<(), String>;

    /// Check if running with elevated privileges
    fn is_running_elevated() -> bool;

    /// Relaunch with elevated privileges
    fn relaunch_elevated() -> Result<(), String>;

    /// Get platform-specific labels for UI
    fn get_labels() -> PlatformLabels;
}

/// Platform-specific UI labels
#[derive(Clone)]
pub struct PlatformLabels {
    pub context_menu: &'static str,
    pub context_menu_desc: &'static str,
    pub menu_shortcut: &'static str,
    pub menu_shortcut_desc: &'static str,
    pub desktop_shortcut: &'static str,
    pub desktop_shortcut_desc: &'static str,
    pub admin_label: &'static str,
    pub admin_desc: &'static str,
    pub trash_name: &'static str,
}

impl Default for PlatformLabels {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self {
                context_menu: "Explorer Context Menu",
                context_menu_desc: "Add 'Usage Analytics' to folder right-click menu",
                menu_shortcut: "Start Menu Shortcut",
                menu_shortcut_desc: "Add to Windows Start Menu",
                desktop_shortcut: "Desktop Shortcut",
                desktop_shortcut_desc: "Add shortcut to Desktop",
                admin_label: "Run as Administrator",
                admin_desc: "Elevate privileges for full access",
                trash_name: "Recycle Bin",
            }
        }
        #[cfg(target_os = "linux")]
        {
            Self {
                context_menu: "File Manager Integration",
                context_menu_desc: "Add to Nautilus/Dolphin context menu",
                menu_shortcut: "Application Menu Entry",
                menu_shortcut_desc: "Add to system applications menu",
                desktop_shortcut: "Desktop Shortcut",
                desktop_shortcut_desc: "Add .desktop file to Desktop",
                admin_label: "Run as Root",
                admin_desc: "Run with superuser privileges",
                trash_name: "Trash",
            }
        }
        #[cfg(target_os = "macos")]
        {
            Self {
                context_menu: "Finder Services",
                context_menu_desc: "Add to Finder Services menu",
                menu_shortcut: "Applications Folder",
                menu_shortcut_desc: "Add to ~/Applications",
                desktop_shortcut: "Desktop Alias",
                desktop_shortcut_desc: "Add alias to Desktop",
                admin_label: "Run as Admin",
                admin_desc: "Run with administrator privileges",
                trash_name: "Trash",
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Self {
                context_menu: "Context Menu",
                context_menu_desc: "Not supported on this platform",
                menu_shortcut: "Menu Shortcut",
                menu_shortcut_desc: "Not supported on this platform",
                desktop_shortcut: "Desktop Shortcut",
                desktop_shortcut_desc: "Not supported on this platform",
                admin_label: "Elevated Privileges",
                admin_desc: "Not supported on this platform",
                trash_name: "Trash",
            }
        }
    }
}

/// Get current platform labels
pub fn get_platform_labels() -> PlatformLabels {
    PlatformLabels::default()
}

/// Get the executable path
pub fn get_exe_path() -> Option<std::path::PathBuf> {
    std::env::current_exe().ok()
}
