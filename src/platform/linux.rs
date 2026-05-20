//! Linux-specific platform implementations

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::{PlatformLabels, PlatformOps};

const APP_NAME: &str = "dua";
const DESKTOP_FILE_NAME: &str = "disk-usage-analyzer.desktop";

pub struct LinuxPlatform;

impl PlatformOps for LinuxPlatform {
    fn is_context_menu_registered() -> bool {
        is_context_menu_registered()
    }

    fn register_context_menu() -> Result<(), String> {
        register_context_menu()
    }

    fn unregister_context_menu() -> Result<(), String> {
        unregister_context_menu()
    }

    fn is_path_registered() -> bool {
        is_path_registered()
    }

    fn register_to_path() -> Result<(), String> {
        register_to_path()
    }

    fn unregister_from_path() -> Result<(), String> {
        unregister_from_path()
    }

    fn is_menu_shortcut_exists() -> bool {
        is_menu_entry_exists()
    }

    fn create_menu_shortcut() -> Result<(), String> {
        create_menu_entry()
    }

    fn remove_menu_shortcut() -> Result<(), String> {
        remove_menu_entry()
    }

    fn is_desktop_shortcut_exists() -> bool {
        is_desktop_shortcut_exists()
    }

    fn create_desktop_shortcut() -> Result<(), String> {
        create_desktop_shortcut()
    }

    fn remove_desktop_shortcut() -> Result<(), String> {
        remove_desktop_shortcut()
    }

    fn is_running_elevated() -> bool {
        is_running_as_root()
    }

    fn relaunch_elevated() -> Result<(), String> {
        relaunch_as_root()
    }

    fn get_labels() -> PlatformLabels {
        PlatformLabels::default()
    }
}

/// Check if running as root
pub fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Relaunch with root privileges using pkexec or sudo
pub fn relaunch_as_root() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let args_str = args.join(" ");

    // Try pkexec first (graphical), then fall back to terminal-based sudo
    let result = if Command::new("which")
        .arg("pkexec")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        Command::new("pkexec")
            .arg(&exe_path)
            .args(&args)
            .spawn()
    } else {
        // Fall back to x-terminal-emulator with sudo
        Command::new("x-terminal-emulator")
            .args(["-e", &format!("sudo {} {}", exe_path.display(), args_str)])
            .spawn()
    };

    result.map_err(|e| format!("Failed to elevate privileges: {}", e))?;
    Ok(())
}

/// Get the local bin directory (~/.local/bin)
pub fn get_local_bin_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".local")
        .join("bin")
}

/// Get the applications directory (~/.local/share/applications)
pub fn get_applications_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".local")
                .join("share")
        })
        .join("applications")
}

/// Get the desktop path
pub fn get_desktop_path() -> PathBuf {
    dirs::desktop_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("Desktop")
    })
}

/// Get Nautilus scripts directory
pub fn get_nautilus_scripts_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".local")
                .join("share")
        })
        .join("nautilus")
        .join("scripts")
}

/// Get KDE service menu directory
pub fn get_kde_services_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".local")
                .join("share")
        })
        .join("kservices5")
        .join("ServiceMenus")
}

/// Check if context menu is registered (Nautilus script or KDE service)
pub fn is_context_menu_registered() -> bool {
    let nautilus_script = get_nautilus_scripts_path().join("Disk Usage Analyzer");
    let kde_service = get_kde_services_path().join("dua.desktop");
    nautilus_script.exists() || kde_service.exists()
}

/// Register context menu for file managers
pub fn register_context_menu() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;

    // Create Nautilus script
    let nautilus_scripts = get_nautilus_scripts_path();
    if let Err(e) = fs::create_dir_all(&nautilus_scripts) {
        eprintln!("Warning: Could not create Nautilus scripts dir: {}", e);
    } else {
        let script_path = nautilus_scripts.join("Disk Usage Analyzer");
        let script_content = format!(
            r#"#!/bin/bash
# Disk Usage Analyzer - Nautilus context menu script
"{}" --path "$NAUTILUS_SCRIPT_CURRENT_URI"
"#,
            exe_path.display()
        );

        fs::write(&script_path, script_content)
            .map_err(|e| format!("Failed to create Nautilus script: {}", e))?;

        // Make executable
        let _ = Command::new("chmod").args(["+x", &script_path.to_string_lossy()]).output();
    }

    // Create KDE service menu
    let kde_services = get_kde_services_path();
    if let Err(e) = fs::create_dir_all(&kde_services) {
        eprintln!("Warning: Could not create KDE services dir: {}", e);
    } else {
        let service_path = kde_services.join("dua.desktop");
        let service_content = format!(
            r#"[Desktop Entry]
Type=Service
ServiceTypes=KonqPopupMenu/Plugin
MimeType=inode/directory;
Actions=analyzeDisk

[Desktop Action analyzeDisk]
Name=Disk Usage Analyzer
Icon=drive-harddisk
Exec="{}" --path %f
"#,
            exe_path.display()
        );

        fs::write(&service_path, service_content)
            .map_err(|e| format!("Failed to create KDE service menu: {}", e))?;
    }

    Ok(())
}

/// Unregister context menu
pub fn unregister_context_menu() -> Result<(), String> {
    let nautilus_script = get_nautilus_scripts_path().join("Disk Usage Analyzer");
    let kde_service = get_kde_services_path().join("dua.desktop");

    if nautilus_script.exists() {
        fs::remove_file(&nautilus_script)
            .map_err(|e| format!("Failed to remove Nautilus script: {}", e))?;
    }

    if kde_service.exists() {
        fs::remove_file(&kde_service)
            .map_err(|e| format!("Failed to remove KDE service: {}", e))?;
    }

    Ok(())
}

/// Check if PATH is registered (symlink in ~/.local/bin)
pub fn is_path_registered() -> bool {
    let symlink_path = get_local_bin_path().join(APP_NAME);
    symlink_path.exists()
}

/// Register to PATH by creating symlink in ~/.local/bin
pub fn register_to_path() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;
    let local_bin = get_local_bin_path();

    fs::create_dir_all(&local_bin)
        .map_err(|e| format!("Failed to create ~/.local/bin: {}", e))?;

    let symlink_path = local_bin.join(APP_NAME);

    // Remove existing symlink if present
    if symlink_path.exists() || symlink_path.is_symlink() {
        let _ = fs::remove_file(&symlink_path);
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&exe_path, &symlink_path)
            .map_err(|e| format!("Failed to create symlink: {}", e))?;
    }

    // Remind user to add ~/.local/bin to PATH if needed
    if let Ok(path) = std::env::var("PATH") {
        if !path.contains(".local/bin") {
            eprintln!(
                "Note: Add ~/.local/bin to your PATH. Add this to ~/.bashrc or ~/.zshrc:\n\
                 export PATH=\"$HOME/.local/bin:$PATH\""
            );
        }
    }

    Ok(())
}

/// Unregister from PATH
pub fn unregister_from_path() -> Result<(), String> {
    let symlink_path = get_local_bin_path().join(APP_NAME);

    if symlink_path.exists() || symlink_path.is_symlink() {
        fs::remove_file(&symlink_path)
            .map_err(|e| format!("Failed to remove symlink: {}", e))?;
    }

    Ok(())
}

/// Get the .desktop file content
fn get_desktop_file_content() -> Result<String, String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;

    Ok(format!(
        r#"[Desktop Entry]
Version=1.0
Type=Application
Name=Disk Usage Analyzer
Comment=Ultra high-performance disk usage analyzer with TUI
Exec="{}" %f
Icon=drive-harddisk
Terminal=true
Categories=System;Utility;FileManager;
Keywords=disk;usage;space;analyzer;folder;size;
MimeType=inode/directory;
"#,
        exe_path.display()
    ))
}

/// Check if menu entry exists
pub fn is_menu_entry_exists() -> bool {
    let desktop_file = get_applications_path().join(DESKTOP_FILE_NAME);
    desktop_file.exists()
}

/// Create application menu entry
pub fn create_menu_entry() -> Result<(), String> {
    let applications_dir = get_applications_path();

    fs::create_dir_all(&applications_dir)
        .map_err(|e| format!("Failed to create applications directory: {}", e))?;

    let desktop_file = applications_dir.join(DESKTOP_FILE_NAME);
    let content = get_desktop_file_content()?;

    fs::write(&desktop_file, content)
        .map_err(|e| format!("Failed to create .desktop file: {}", e))?;

    // Update desktop database
    let _ = Command::new("update-desktop-database")
        .arg(&applications_dir)
        .output();

    Ok(())
}

/// Remove application menu entry
pub fn remove_menu_entry() -> Result<(), String> {
    let desktop_file = get_applications_path().join(DESKTOP_FILE_NAME);

    if desktop_file.exists() {
        fs::remove_file(&desktop_file)
            .map_err(|e| format!("Failed to remove .desktop file: {}", e))?;
    }

    Ok(())
}

/// Check if desktop shortcut exists
pub fn is_desktop_shortcut_exists() -> bool {
    let desktop_file = get_desktop_path().join(DESKTOP_FILE_NAME);
    desktop_file.exists()
}

/// Create desktop shortcut
pub fn create_desktop_shortcut() -> Result<(), String> {
    let desktop_dir = get_desktop_path();
    let desktop_file = desktop_dir.join(DESKTOP_FILE_NAME);
    let content = get_desktop_file_content()?;

    fs::write(&desktop_file, content)
        .map_err(|e| format!("Failed to create desktop shortcut: {}", e))?;

    // Make executable (required by some desktop environments)
    let _ = Command::new("chmod")
        .args(["+x", &desktop_file.to_string_lossy()])
        .output();

    // Trust the desktop file (GNOME)
    let _ = Command::new("gio")
        .args(["set", &desktop_file.to_string_lossy(), "metadata::trusted", "true"])
        .output();

    Ok(())
}

/// Remove desktop shortcut
pub fn remove_desktop_shortcut() -> Result<(), String> {
    let desktop_file = get_desktop_path().join(DESKTOP_FILE_NAME);

    if desktop_file.exists() {
        fs::remove_file(&desktop_file)
            .map_err(|e| format!("Failed to remove desktop shortcut: {}", e))?;
    }

    Ok(())
}

pub fn get_console_font() -> (String, u16) {
    ("Default".to_string(), 16)
}

pub fn set_console_font(_name: &str, _size: u16) -> Result<(), String> {
    Err("Font settings are controlled by the terminal emulator".to_string())
}

pub fn get_available_fonts() -> Vec<String> {
    Vec::new()
}
