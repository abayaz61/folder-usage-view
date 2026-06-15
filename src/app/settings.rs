use std::fs;
use std::path::PathBuf;

use crate::app::state::ViewMode;
use crate::ui::theme::ColorPalette;
use crate::util::i18n::Language;

const APP_NAME: &str = "folder-usage-view";
const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum StartupLocation {
    #[default]
    LastLocation,
    CurrentFolder,
    ComputerView,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub startup_location: StartupLocation,
    pub context_menu_enabled: bool,
    pub path_registered: bool,
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub color_palette: ColorPalette,
    #[serde(default)]
    pub use_ascii_icons: bool,
    #[serde(default)]
    pub view_mode: ViewMode,
    #[serde(default)]
    pub allow_delete: bool,
    #[serde(default)]
    pub delete_to_trash: bool,
    #[serde(default = "default_true")]
    pub show_delete_confirmation: bool,
    #[serde(default)]
    pub run_as_admin: bool,
    #[serde(default = "default_font_name")]
    pub font_name: String,
    #[serde(default = "default_font_size")]
    pub font_size: u16,
}

fn default_true() -> bool {
    true
}

fn default_font_name() -> String {
    "Consolas".to_string()
}

fn default_font_size() -> u16 {
    16
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            startup_location: StartupLocation::LastLocation,
            context_menu_enabled: false,
            path_registered: false,
            language: Language::default(),
            color_palette: ColorPalette::default(),
            use_ascii_icons: false,
            view_mode: ViewMode::default(),
            allow_delete: false,
            delete_to_trash: false,
            show_delete_confirmation: true,
            run_as_admin: false,
            font_name: default_font_name(),
            font_size: default_font_size(),
        }
    }
}

impl Settings {
    /// Platform-specific config directory for the app (where settings.json and reports live).
    /// `%APPDATA%\folder-usage-view` on Windows, `~/.config/folder-usage-view` on Linux/macOS.
    pub fn get_config_dir() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var("APPDATA")
                .ok()
                .map(|p| PathBuf::from(p).join(APP_NAME))
        }
        #[cfg(not(windows))]
        {
            std::env::var("HOME")
                .ok()
                .map(|p| PathBuf::from(p).join(".config").join(APP_NAME))
        }
    }

    pub fn load() -> Self {
        if let Some(config_dir) = Self::get_config_dir() {
            let settings_path = config_dir.join(SETTINGS_FILE);
            if settings_path.exists() {
                if let Ok(content) = fs::read_to_string(&settings_path) {
                    if let Ok(settings) = serde_json::from_str(&content) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(config_dir) = Self::get_config_dir() {
            fs::create_dir_all(&config_dir)?;
            let settings_path = config_dir.join(SETTINGS_FILE);
            let content = serde_json::to_string_pretty(self)
                .map_err(std::io::Error::other)?;
            fs::write(settings_path, content)?;
        }
        Ok(())
    }
}

// Platform-specific module re-exports for backwards compatibility
// The actual implementations are now in src/platform/
pub mod windows {
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    use std::path::PathBuf;

    #[cfg(target_os = "windows")]
    pub use crate::platform::{
        create_desktop_shortcut, create_start_menu_shortcut, get_desktop_path, get_install_path,
        get_start_menu_path, is_context_menu_registered, is_desktop_shortcut_exists,
        is_path_registered, is_running_as_admin, is_start_menu_shortcut_exists, register_context_menu,
        register_to_path, relaunch_as_admin, relaunch_as_admin_with_flag, remove_desktop_shortcut,
        remove_start_menu_shortcut, unregister_context_menu, unregister_from_path,
        get_console_font, set_console_font, get_available_fonts,
    };

    #[cfg(target_os = "linux")]
    pub use crate::platform::{
        create_desktop_shortcut, create_menu_entry as create_start_menu_shortcut,
        get_desktop_path, get_local_bin_path as get_install_path,
        get_applications_path as get_start_menu_path, is_context_menu_registered,
        is_desktop_shortcut_exists, is_menu_entry_exists as is_start_menu_shortcut_exists,
        is_path_registered, is_running_as_root as is_running_as_admin, register_context_menu,
        register_to_path, relaunch_as_root as relaunch_as_admin,
        remove_desktop_shortcut, remove_menu_entry as remove_start_menu_shortcut,
        unregister_context_menu, unregister_from_path,
        get_console_font, set_console_font, get_available_fonts,
    };

    #[cfg(target_os = "linux")]
    pub fn relaunch_as_admin_with_flag() -> Result<(), String> {
        relaunch_as_admin()
    }

    #[cfg(target_os = "macos")]
    pub use crate::platform::{
        create_desktop_alias as create_desktop_shortcut,
        create_applications_entry as create_start_menu_shortcut, get_desktop_path,
        get_local_bin_path as get_install_path,
        get_user_applications_path as get_start_menu_path, is_context_menu_registered,
        is_desktop_alias_exists as is_desktop_shortcut_exists,
        is_applications_entry_exists as is_start_menu_shortcut_exists, is_path_registered,
        is_running_as_root as is_running_as_admin, register_context_menu, register_to_path,
        relaunch_as_admin, remove_desktop_alias as remove_desktop_shortcut,
        remove_applications_entry as remove_start_menu_shortcut, unregister_context_menu,
        unregister_from_path,
        get_console_font, set_console_font, get_available_fonts,
    };

    #[cfg(target_os = "macos")]
    pub fn relaunch_as_admin_with_flag() -> Result<(), String> {
        relaunch_as_admin()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn is_running_as_admin() -> bool {
        false
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn relaunch_as_admin() -> Result<(), String> {
        Err("Admin elevation is not supported on this platform".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn relaunch_as_admin_with_flag() -> Result<(), String> {
        Err("Admin elevation is not supported on this platform".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn is_context_menu_registered() -> bool {
        false
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn register_context_menu() -> Result<(), String> {
        Err("Context menu is not supported on this platform".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn unregister_context_menu() -> Result<(), String> {
        Err("Context menu is not supported on this platform".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn is_path_registered() -> bool {
        false
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn register_to_path() -> Result<(), String> {
        Err("PATH registration is not supported on this platform".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn unregister_from_path() -> Result<(), String> {
        Err("PATH registration is not supported on this platform".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn is_start_menu_shortcut_exists() -> bool {
        false
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn is_desktop_shortcut_exists() -> bool {
        false
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn create_start_menu_shortcut() -> Result<(), String> {
        Err("Menu shortcuts are not supported on this platform".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn remove_start_menu_shortcut() -> Result<(), String> {
        Err("Menu shortcuts are not supported on this platform".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn create_desktop_shortcut() -> Result<(), String> {
        Err("Desktop shortcuts are not supported on this platform".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn remove_desktop_shortcut() -> Result<(), String> {
        Err("Desktop shortcuts are not supported on this platform".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn get_install_path() -> PathBuf {
        PathBuf::from("/usr/local/bin")
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn get_start_menu_path() -> PathBuf {
        PathBuf::from("/usr/share/applications")
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn get_console_font() -> (String, u16) {
        ("Default".to_string(), 16)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn set_console_font(_name: &str, _size: u16) -> Result<(), String> {
        Err("Font settings are not supported on this platform".to_string())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn get_available_fonts() -> Vec<String> {
        Vec::new()
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    pub fn get_desktop_path() -> PathBuf {
        dirs::desktop_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
    }
}
