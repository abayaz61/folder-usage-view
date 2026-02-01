//! Windows-specific platform implementations

use std::fs;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use super::{PlatformLabels, PlatformOps};

const INSTALL_DIR: &str = "FolderUsageView";
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct WindowsPlatform;

impl PlatformOps for WindowsPlatform {
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
        is_start_menu_shortcut_exists()
    }

    fn create_menu_shortcut() -> Result<(), String> {
        create_start_menu_shortcut()
    }

    fn remove_menu_shortcut() -> Result<(), String> {
        remove_start_menu_shortcut()
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
        is_running_as_admin()
    }

    fn relaunch_elevated() -> Result<(), String> {
        relaunch_as_admin()
    }

    fn get_labels() -> PlatformLabels {
        PlatformLabels::default()
    }
}

/// Check if the current process is running with admin privileges
pub fn is_running_as_admin() -> bool {
    let output = Command::new("cmd")
        .args(["/C", "net session >nul 2>&1 && echo true || echo false"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(o) => {
            let result = String::from_utf8_lossy(&o.stdout).trim().to_lowercase();
            result.contains("true")
        }
        Err(_) => false,
    }
}

/// Relaunch the current application with admin privileges
pub fn relaunch_as_admin() -> Result<(), String> {
    relaunch_as_admin_with_flag()
}

/// Relaunch the current application with admin privileges, adding --elevated flag
pub fn relaunch_as_admin_with_flag() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;

    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|arg| arg != "--elevated")
        .collect();

    let args_str = if args.is_empty() {
        "--elevated".to_string()
    } else {
        format!("{} --elevated", args.join(" "))
    };

    let ps_script = format!(
        r#"Start-Process -FilePath '{}' -ArgumentList '{}' -Verb RunAs"#,
        exe_path.to_string_lossy().replace("'", "''"),
        args_str.replace("'", "''")
    );

    let result = Command::new("powershell")
        .args(["-Command", &ps_script])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| e.to_string())?;

    drop(result);
    Ok(())
}

pub fn get_install_path() -> PathBuf {
    let program_files =
        std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
    PathBuf::from(program_files).join(INSTALL_DIR)
}

pub fn is_context_menu_registered() -> bool {
    let output = Command::new("reg")
        .args(["query", r"HKEY_CLASSES_ROOT\Directory\shell\FolderUsageView"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    matches!(output, Ok(o) if o.status.success())
}

pub fn register_context_menu() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;
    let exe_str = exe_path.to_string_lossy();

    let result = Command::new("reg")
        .args([
            "add",
            r"HKEY_CLASSES_ROOT\Directory\shell\FolderUsageView",
            "/ve",
            "/d",
            "Usage Analytics",
            "/f",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;

    if !result.status.success() {
        return Err("Failed to create registry key. Run as Administrator.".to_string());
    }

    let _ = Command::new("reg")
        .args([
            "add",
            r"HKEY_CLASSES_ROOT\Directory\shell\FolderUsageView",
            "/v",
            "Icon",
            "/d",
            &format!("{},0", exe_str),
            "/f",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let command = format!("\"{}\" --path \"%V\"", exe_str);
    let result = Command::new("reg")
        .args([
            "add",
            r"HKEY_CLASSES_ROOT\Directory\shell\FolderUsageView\command",
            "/ve",
            "/d",
            &command,
            "/f",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;

    if !result.status.success() {
        return Err("Failed to create command key. Run as Administrator.".to_string());
    }

    // Also add for Directory Background
    let _ = Command::new("reg")
        .args([
            "add",
            r"HKEY_CLASSES_ROOT\Directory\Background\shell\FolderUsageView",
            "/ve",
            "/d",
            "Usage Analytics",
            "/f",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let _ = Command::new("reg")
        .args([
            "add",
            r"HKEY_CLASSES_ROOT\Directory\Background\shell\FolderUsageView",
            "/v",
            "Icon",
            "/d",
            &format!("{},0", exe_str),
            "/f",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let command_bg = format!("\"{}\" --path \"%V\"", exe_str);
    let _ = Command::new("reg")
        .args([
            "add",
            r"HKEY_CLASSES_ROOT\Directory\Background\shell\FolderUsageView\command",
            "/ve",
            "/d",
            &command_bg,
            "/f",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    Ok(())
}

pub fn unregister_context_menu() -> Result<(), String> {
    let result = Command::new("reg")
        .args([
            "delete",
            r"HKEY_CLASSES_ROOT\Directory\shell\FolderUsageView",
            "/f",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;

    let _ = Command::new("reg")
        .args([
            "delete",
            r"HKEY_CLASSES_ROOT\Directory\Background\shell\FolderUsageView",
            "/f",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    if !result.status.success() {
        return Err("Failed to remove registry key. Run as Administrator.".to_string());
    }

    Ok(())
}

pub fn is_path_registered() -> bool {
    let install_path = get_install_path();
    let exe_path = install_path.join("dua.exe");
    exe_path.exists()
}

pub fn register_to_path() -> Result<(), String> {
    let install_path = get_install_path();
    let source_exe = super::get_exe_path().ok_or("Could not get executable path")?;

    fs::create_dir_all(&install_path)
        .map_err(|e| format!("Failed to create directory: {}. Run as Administrator.", e))?;

    let dest_exe = install_path.join("dua.exe");
    fs::copy(&source_exe, &dest_exe)
        .map_err(|e| format!("Failed to copy executable: {}. Run as Administrator.", e))?;

    let install_path_str = install_path.to_string_lossy();
    let result = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                r#"$path = [Environment]::GetEnvironmentVariable('Path', 'Machine'); if ($path -notlike '*{}*') {{ [Environment]::SetEnvironmentVariable('Path', $path + ';{}', 'Machine') }}"#,
                install_path_str, install_path_str
            ),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;

    if !result.status.success() {
        return Err("Failed to update PATH. Run as Administrator.".to_string());
    }

    Ok(())
}

pub fn unregister_from_path() -> Result<(), String> {
    let install_path = get_install_path();

    let install_path_str = install_path.to_string_lossy();
    let _ = Command::new("powershell")
        .args([
            "-Command",
            &format!(
                r#"$path = [Environment]::GetEnvironmentVariable('Path', 'Machine'); $newPath = ($path -split ';' | Where-Object {{ $_ -ne '{}' }}) -join ';'; [Environment]::SetEnvironmentVariable('Path', $newPath, 'Machine')"#,
                install_path_str
            ),
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    if install_path.exists() {
        fs::remove_dir_all(&install_path)
            .map_err(|e| format!("Failed to remove directory: {}. Run as Administrator.", e))?;
    }

    Ok(())
}

pub fn get_start_menu_path() -> PathBuf {
    let appdata = std::env::var("APPDATA")
        .unwrap_or_else(|_| "C:\\Users\\Default\\AppData\\Roaming".to_string());
    PathBuf::from(appdata)
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
}

pub fn get_desktop_path() -> PathBuf {
    dirs::desktop_dir().unwrap_or_else(|| {
        let userprofile =
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
        PathBuf::from(userprofile).join("Desktop")
    })
}

pub fn is_start_menu_shortcut_exists() -> bool {
    let shortcut_path = get_start_menu_path().join("Disk Usage Analyzer.lnk");
    shortcut_path.exists()
}

pub fn is_desktop_shortcut_exists() -> bool {
    let shortcut_path = get_desktop_path().join("Disk Usage Analyzer.lnk");
    shortcut_path.exists()
}

pub fn create_shortcut(
    target_path: &str,
    shortcut_path: &str,
    description: &str,
) -> Result<(), String> {
    let ps_script = format!(
        r#"$WshShell = New-Object -ComObject WScript.Shell; $Shortcut = $WshShell.CreateShortcut('{}'); $Shortcut.TargetPath = '{}'; $Shortcut.Description = '{}'; $Shortcut.WorkingDirectory = '%USERPROFILE%'; $Shortcut.Save()"#,
        shortcut_path.replace("'", "''"),
        target_path.replace("'", "''"),
        description.replace("'", "''")
    );

    let result = Command::new("powershell")
        .args(["-Command", &ps_script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!("Failed to create shortcut: {}", stderr));
    }

    Ok(())
}

pub fn create_start_menu_shortcut() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;

    let start_menu = get_start_menu_path();
    fs::create_dir_all(&start_menu)
        .map_err(|e| format!("Failed to create Start Menu directory: {}", e))?;

    let shortcut_path = start_menu.join("Disk Usage Analyzer.lnk");

    create_shortcut(
        &exe_path.to_string_lossy(),
        &shortcut_path.to_string_lossy(),
        "Disk Usage Analyzer - Ultra high-performance disk usage analyzer",
    )
}

pub fn remove_start_menu_shortcut() -> Result<(), String> {
    let shortcut_path = get_start_menu_path().join("Disk Usage Analyzer.lnk");
    if shortcut_path.exists() {
        fs::remove_file(&shortcut_path).map_err(|e| format!("Failed to remove shortcut: {}", e))?;
    }
    Ok(())
}

pub fn create_desktop_shortcut() -> Result<(), String> {
    let exe_path = super::get_exe_path().ok_or("Could not get executable path")?;

    let desktop = get_desktop_path();
    let shortcut_path = desktop.join("Disk Usage Analyzer.lnk");

    create_shortcut(
        &exe_path.to_string_lossy(),
        &shortcut_path.to_string_lossy(),
        "Disk Usage Analyzer - Ultra high-performance disk usage analyzer",
    )
}

pub fn remove_desktop_shortcut() -> Result<(), String> {
    let shortcut_path = get_desktop_path().join("Disk Usage Analyzer.lnk");
    if shortcut_path.exists() {
        fs::remove_file(&shortcut_path).map_err(|e| format!("Failed to remove shortcut: {}", e))?;
    }
    Ok(())
}
