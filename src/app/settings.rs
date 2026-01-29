use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "folder-usage-view";
const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StartupLocation {
    LastLocation,
    CurrentFolder,
    ComputerView,
}

impl Default for StartupLocation {
    fn default() -> Self {
        StartupLocation::LastLocation
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub startup_location: StartupLocation,
    pub context_menu_enabled: bool,
    pub path_registered: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            startup_location: StartupLocation::LastLocation,
            context_menu_enabled: false,
            path_registered: false,
        }
    }
}

impl Settings {
    fn get_config_dir() -> Option<PathBuf> {
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
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            fs::write(settings_path, content)?;
        }
        Ok(())
    }
}

// Windows-specific functions for registry and PATH
#[cfg(windows)]
pub mod windows {
    use std::process::Command;
    use std::path::PathBuf;
    use std::fs;

    const INSTALL_DIR: &str = "FolderUsageView";

    pub fn get_install_path() -> PathBuf {
        let program_files = std::env::var("ProgramFiles")
            .unwrap_or_else(|_| "C:\\Program Files".to_string());
        PathBuf::from(program_files).join(INSTALL_DIR)
    }

    pub fn get_exe_path() -> Option<PathBuf> {
        std::env::current_exe().ok()
    }

    pub fn is_context_menu_registered() -> bool {
        let output = Command::new("reg")
            .args(["query", r"HKEY_CLASSES_ROOT\Directory\shell\FolderUsageView"])
            .output();

        matches!(output, Ok(o) if o.status.success())
    }

    pub fn register_context_menu() -> Result<(), String> {
        let exe_path = get_exe_path()
            .ok_or("Could not get executable path")?;

        let exe_str = exe_path.to_string_lossy();

        // Create the shell key
        let result = Command::new("reg")
            .args([
                "add",
                r"HKEY_CLASSES_ROOT\Directory\shell\FolderUsageView",
                "/ve",
                "/d",
                "Usage Analytics",
                "/f"
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !result.status.success() {
            return Err("Failed to create registry key. Run as Administrator.".to_string());
        }

        // Set icon
        let _ = Command::new("reg")
            .args([
                "add",
                r"HKEY_CLASSES_ROOT\Directory\shell\FolderUsageView",
                "/v",
                "Icon",
                "/d",
                &format!("{},0", exe_str),
                "/f"
            ])
            .output();

        // Create command key
        let command = format!("\"{}\" --path \"%V\"", exe_str);
        let result = Command::new("reg")
            .args([
                "add",
                r"HKEY_CLASSES_ROOT\Directory\shell\FolderUsageView\command",
                "/ve",
                "/d",
                &command,
                "/f"
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !result.status.success() {
            return Err("Failed to create command key. Run as Administrator.".to_string());
        }

        // Also add for Directory Background (right-click in folder background)
        let _ = Command::new("reg")
            .args([
                "add",
                r"HKEY_CLASSES_ROOT\Directory\Background\shell\FolderUsageView",
                "/ve",
                "/d",
                "Usage Analytics",
                "/f"
            ])
            .output();

        let _ = Command::new("reg")
            .args([
                "add",
                r"HKEY_CLASSES_ROOT\Directory\Background\shell\FolderUsageView",
                "/v",
                "Icon",
                "/d",
                &format!("{},0", exe_str),
                "/f"
            ])
            .output();

        let command_bg = format!("\"{}\" --path \"%V\"", exe_str);
        let _ = Command::new("reg")
            .args([
                "add",
                r"HKEY_CLASSES_ROOT\Directory\Background\shell\FolderUsageView\command",
                "/ve",
                "/d",
                &command_bg,
                "/f"
            ])
            .output();

        Ok(())
    }

    pub fn unregister_context_menu() -> Result<(), String> {
        let result = Command::new("reg")
            .args([
                "delete",
                r"HKEY_CLASSES_ROOT\Directory\shell\FolderUsageView",
                "/f"
            ])
            .output()
            .map_err(|e| e.to_string())?;

        let _ = Command::new("reg")
            .args([
                "delete",
                r"HKEY_CLASSES_ROOT\Directory\Background\shell\FolderUsageView",
                "/f"
            ])
            .output();

        if !result.status.success() {
            return Err("Failed to remove registry key. Run as Administrator.".to_string());
        }

        Ok(())
    }

    pub fn is_path_registered() -> bool {
        let install_path = get_install_path();
        let exe_path = install_path.join("folder-usage-view.exe");
        exe_path.exists()
    }

    pub fn register_to_path() -> Result<(), String> {
        let install_path = get_install_path();
        let source_exe = get_exe_path()
            .ok_or("Could not get executable path")?;

        // Create install directory
        fs::create_dir_all(&install_path)
            .map_err(|e| format!("Failed to create directory: {}. Run as Administrator.", e))?;

        // Copy executable
        let dest_exe = install_path.join("folder-usage-view.exe");
        fs::copy(&source_exe, &dest_exe)
            .map_err(|e| format!("Failed to copy executable: {}. Run as Administrator.", e))?;

        // Add to system PATH
        let install_path_str = install_path.to_string_lossy();
        let result = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    r#"$path = [Environment]::GetEnvironmentVariable('Path', 'Machine'); if ($path -notlike '*{}*') {{ [Environment]::SetEnvironmentVariable('Path', $path + ';{}', 'Machine') }}"#,
                    install_path_str, install_path_str
                )
            ])
            .output()
            .map_err(|e| e.to_string())?;

        if !result.status.success() {
            return Err("Failed to update PATH. Run as Administrator.".to_string());
        }

        Ok(())
    }

    pub fn unregister_from_path() -> Result<(), String> {
        let install_path = get_install_path();

        // Remove from PATH
        let install_path_str = install_path.to_string_lossy();
        let _ = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    r#"$path = [Environment]::GetEnvironmentVariable('Path', 'Machine'); $newPath = ($path -split ';' | Where-Object {{ $_ -ne '{}' }}) -join ';'; [Environment]::SetEnvironmentVariable('Path', $newPath, 'Machine')"#,
                    install_path_str
                )
            ])
            .output();

        // Remove installed files
        if install_path.exists() {
            fs::remove_dir_all(&install_path)
                .map_err(|e| format!("Failed to remove directory: {}. Run as Administrator.", e))?;
        }

        Ok(())
    }
}

#[cfg(not(windows))]
pub mod windows {
    pub fn is_context_menu_registered() -> bool { false }
    pub fn register_context_menu() -> Result<(), String> {
        Err("Context menu is only supported on Windows".to_string())
    }
    pub fn unregister_context_menu() -> Result<(), String> {
        Err("Context menu is only supported on Windows".to_string())
    }
    pub fn is_path_registered() -> bool { false }
    pub fn register_to_path() -> Result<(), String> {
        Err("PATH registration is only supported on Windows".to_string())
    }
    pub fn unregister_from_path() -> Result<(), String> {
        Err("PATH registration is only supported on Windows".to_string())
    }
}
