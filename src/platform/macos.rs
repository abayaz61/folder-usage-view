//! macOS-specific platform implementations

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::{PlatformLabels, PlatformOps};

const APP_NAME: &str = "dua";

pub struct MacOSPlatform;

impl PlatformOps for MacOSPlatform {
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
        is_applications_entry_exists()
    }

    fn create_menu_shortcut() -> Result<(), String> {
        create_applications_entry()
    }

    fn remove_menu_shortcut() -> Result<(), String> {
        remove_applications_entry()
    }

    fn is_desktop_shortcut_exists() -> bool {
        is_desktop_alias_exists()
    }

    fn create_desktop_shortcut() -> Result<(), String> {
        create_desktop_alias()
    }

    fn remove_desktop_shortcut() -> Result<(), String> {
        remove_desktop_alias()
    }

    fn is_running_elevated() -> bool {
        is_running_as_root()
    }

    fn relaunch_elevated() -> Result<(), String> {
        relaunch_as_admin()
    }

    fn get_labels() -> PlatformLabels {
        PlatformLabels::default()
    }
}

/// Check if running as root
pub fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Relaunch with admin privileges using osascript
pub fn relaunch_as_admin() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let args_str = args.join(" ");

    // Use AppleScript to request admin privileges
    let script = format!(
        r#"do shell script "\"{}\" {}" with administrator privileges"#,
        exe_path.display(),
        args_str
    );

    Command::new("osascript")
        .args(["-e", &script])
        .spawn()
        .map_err(|e| format!("Failed to elevate privileges: {}", e))?;

    Ok(())
}

/// Get the local bin directory (/usr/local/bin)
pub fn get_local_bin_path() -> PathBuf {
    PathBuf::from("/usr/local/bin")
}

/// Get the user Applications directory (~/Applications)
pub fn get_user_applications_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Applications")
}

/// Get the desktop path
pub fn get_desktop_path() -> PathBuf {
    dirs::desktop_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("Desktop")
    })
}

/// Get the Services directory for Automator workflows
pub fn get_services_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Library")
        .join("Services")
}

/// Check if context menu is registered (Automator workflow in Services)
pub fn is_context_menu_registered() -> bool {
    let workflow_path = get_services_path().join("Disk Usage Analyzer.workflow");
    workflow_path.exists()
}

/// Register context menu as Automator Service/Quick Action
pub fn register_context_menu() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;
    let services_dir = get_services_path();

    fs::create_dir_all(&services_dir)
        .map_err(|e| format!("Failed to create Services directory: {}", e))?;

    let workflow_dir = services_dir.join("Disk Usage Analyzer.workflow");
    let contents_dir = workflow_dir.join("Contents");

    fs::create_dir_all(&contents_dir)
        .map_err(|e| format!("Failed to create workflow directory: {}", e))?;

    // Create Info.plist
    let info_plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSServices</key>
    <array>
        <dict>
            <key>NSMenuItem</key>
            <dict>
                <key>default</key>
                <string>Disk Usage Analyzer</string>
            </dict>
            <key>NSMessage</key>
            <string>runWorkflowAsService</string>
            <key>NSSendFileTypes</key>
            <array>
                <string>public.folder</string>
            </array>
        </dict>
    </array>
</dict>
</plist>"#
    );

    fs::write(contents_dir.join("Info.plist"), info_plist)
        .map_err(|e| format!("Failed to create Info.plist: {}", e))?;

    // Create document.wflow (Automator workflow)
    let workflow_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>AMApplicationBuild</key>
    <string>523</string>
    <key>AMApplicationVersion</key>
    <string>2.10</string>
    <key>AMDocumentVersion</key>
    <string>2</string>
    <key>actions</key>
    <array>
        <dict>
            <key>action</key>
            <dict>
                <key>AMAccepts</key>
                <dict>
                    <key>Container</key>
                    <string>List</string>
                    <key>Optional</key>
                    <true/>
                    <key>Types</key>
                    <array>
                        <string>com.apple.cocoa.path</string>
                    </array>
                </dict>
                <key>AMActionVersion</key>
                <string>2.0.3</string>
                <key>AMApplication</key>
                <array>
                    <string>Automator</string>
                </array>
                <key>AMCategory</key>
                <string>AMCategoryUtilities</string>
                <key>AMIconName</key>
                <string>Automator</string>
                <key>AMName</key>
                <string>Run Shell Script</string>
                <key>AMProvides</key>
                <dict>
                    <key>Container</key>
                    <string>List</string>
                    <key>Types</key>
                    <array>
                        <string>com.apple.cocoa.string</string>
                    </array>
                </dict>
                <key>ActionBundlePath</key>
                <string>/System/Library/Automator/Run Shell Script.action</string>
                <key>ActionName</key>
                <string>Run Shell Script</string>
                <key>ActionParameters</key>
                <dict>
                    <key>COMMAND_STRING</key>
                    <string>for f in "$@"; do
    open -a Terminal.app "{}" --args --path "$f"
done</string>
                    <key>CheckedForUserDefaultShell</key>
                    <true/>
                    <key>inputMethod</key>
                    <integer>1</integer>
                    <key>shell</key>
                    <string>/bin/bash</string>
                    <key>source</key>
                    <string></string>
                </dict>
                <key>BundleIdentifier</key>
                <string>com.apple.RunShellScript</string>
                <key>CFBundleVersion</key>
                <string>2.0.3</string>
                <key>CanShowSelectedItemsWhenRun</key>
                <false/>
                <key>CanShowWhenRun</key>
                <true/>
                <key>Category</key>
                <array>
                    <string>AMCategoryUtilities</string>
                </array>
                <key>Class Name</key>
                <string>RunShellScriptAction</string>
                <key>IgnoresInput</key>
                <false/>
                <key>InputUUID</key>
                <string>E7E5E15B-8E48-47B0-8727-A0E4E4A8C9D6</string>
                <key>Keywords</key>
                <array>
                    <string>Shell</string>
                    <string>Script</string>
                    <string>Command</string>
                    <string>Run</string>
                    <string>Unix</string>
                </array>
                <key>OutputUUID</key>
                <string>D8D99C86-6A19-4CDB-B7DB-E1E0E5F66C6F</string>
            </dict>
        </dict>
    </array>
    <key>connectors</key>
    <dict/>
    <key>workflowMetaData</key>
    <dict>
        <key>workflowTypeIdentifier</key>
        <string>com.apple.Automator.servicesMenu</string>
    </dict>
</dict>
</plist>"#,
        exe_path.display()
    );

    fs::write(contents_dir.join("document.wflow"), workflow_content)
        .map_err(|e| format!("Failed to create workflow: {}", e))?;

    Ok(())
}

/// Unregister context menu
pub fn unregister_context_menu() -> Result<(), String> {
    let workflow_path = get_services_path().join("Disk Usage Analyzer.workflow");

    if workflow_path.exists() {
        fs::remove_dir_all(&workflow_path)
            .map_err(|e| format!("Failed to remove workflow: {}", e))?;
    }

    Ok(())
}

/// Check if PATH is registered (symlink in /usr/local/bin)
pub fn is_path_registered() -> bool {
    let symlink_path = get_local_bin_path().join(APP_NAME);
    symlink_path.exists()
}

/// Register to PATH by creating symlink in /usr/local/bin
pub fn register_to_path() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;
    let local_bin = get_local_bin_path();

    // Create /usr/local/bin if it doesn't exist (requires admin)
    if !local_bin.exists() {
        fs::create_dir_all(&local_bin)
            .map_err(|e| format!("Failed to create /usr/local/bin: {}. Try running with sudo.", e))?;
    }

    let symlink_path = local_bin.join(APP_NAME);

    // Remove existing symlink if present
    if symlink_path.exists() || symlink_path.is_symlink() {
        let _ = fs::remove_file(&symlink_path);
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&exe_path, &symlink_path)
            .map_err(|e| format!("Failed to create symlink: {}. Try running with sudo.", e))?;
    }

    Ok(())
}

/// Unregister from PATH
pub fn unregister_from_path() -> Result<(), String> {
    let symlink_path = get_local_bin_path().join(APP_NAME);

    if symlink_path.exists() || symlink_path.is_symlink() {
        fs::remove_file(&symlink_path)
            .map_err(|e| format!("Failed to remove symlink: {}. Try running with sudo.", e))?;
    }

    Ok(())
}

/// Check if Applications entry exists
pub fn is_applications_entry_exists() -> bool {
    let app_path = get_user_applications_path().join("Disk Usage Analyzer.app");
    app_path.exists()
}

/// Create Applications entry (simple .app bundle wrapper)
pub fn create_applications_entry() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;
    let apps_dir = get_user_applications_path();

    fs::create_dir_all(&apps_dir)
        .map_err(|e| format!("Failed to create Applications directory: {}", e))?;

    let app_bundle = apps_dir.join("Disk Usage Analyzer.app");
    let contents_dir = app_bundle.join("Contents");
    let macos_dir = contents_dir.join("MacOS");

    fs::create_dir_all(&macos_dir)
        .map_err(|e| format!("Failed to create app bundle: {}", e))?;

    // Create Info.plist
    let info_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>launcher</string>
    <key>CFBundleIdentifier</key>
    <string>com.codegen.disk-usage-analyzer</string>
    <key>CFBundleName</key>
    <string>Disk Usage Analyzer</string>
    <key>CFBundleDisplayName</key>
    <string>Disk Usage Analyzer</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.13</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.folder</string>
            </array>
        </dict>
    </array>
</dict>
</plist>"#;

    fs::write(contents_dir.join("Info.plist"), info_plist)
        .map_err(|e| format!("Failed to create Info.plist: {}", e))?;

    // Create launcher script
    let launcher_script = format!(
        r#"#!/bin/bash
open -a Terminal.app "{}"
"#,
        exe_path.display()
    );

    let launcher_path = macos_dir.join("launcher");
    fs::write(&launcher_path, launcher_script)
        .map_err(|e| format!("Failed to create launcher: {}", e))?;

    // Make executable
    Command::new("chmod")
        .args(["+x", &launcher_path.to_string_lossy()])
        .output()
        .map_err(|e| format!("Failed to make launcher executable: {}", e))?;

    Ok(())
}

/// Remove Applications entry
pub fn remove_applications_entry() -> Result<(), String> {
    let app_bundle = get_user_applications_path().join("Disk Usage Analyzer.app");

    if app_bundle.exists() {
        fs::remove_dir_all(&app_bundle)
            .map_err(|e| format!("Failed to remove app bundle: {}", e))?;
    }

    Ok(())
}

/// Check if desktop alias exists
pub fn is_desktop_alias_exists() -> bool {
    let alias_path = get_desktop_path().join("Disk Usage Analyzer");
    alias_path.exists()
}

/// Create desktop alias
pub fn create_desktop_alias() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;
    let desktop = get_desktop_path();
    let alias_path = desktop.join("Disk Usage Analyzer");

    // Use AppleScript to create a proper Finder alias
    let script = format!(
        r#"tell application "Finder"
    make alias file to POSIX file "{}" at POSIX file "{}"
    set name of result to "Disk Usage Analyzer"
end tell"#,
        exe_path.display(),
        desktop.display()
    );

    let result = Command::new("osascript")
        .args(["-e", &script])
        .output()
        .map_err(|e| format!("Failed to create alias: {}", e))?;

    if !result.status.success() {
        // Fallback: create symlink
        #[cfg(unix)]
        {
            if alias_path.exists() || alias_path.is_symlink() {
                let _ = fs::remove_file(&alias_path);
            }
            std::os::unix::fs::symlink(&exe_path, &alias_path)
                .map_err(|e| format!("Failed to create symlink: {}", e))?;
        }
    }

    Ok(())
}

/// Remove desktop alias
pub fn remove_desktop_alias() -> Result<(), String> {
    let alias_path = get_desktop_path().join("Disk Usage Analyzer");

    if alias_path.exists() || alias_path.is_symlink() {
        fs::remove_file(&alias_path)
            .map_err(|e| format!("Failed to remove alias: {}", e))?;
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
