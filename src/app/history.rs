use std::fs;
use std::path::{Path, PathBuf};

const APP_NAME: &str = "folder-usage-view";
const HISTORY_FILE: &str = "last_location.txt";

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

pub fn save_last_location(path: &Path) -> std::io::Result<()> {
    if let Some(config_dir) = get_config_dir() {
        fs::create_dir_all(&config_dir)?;
        let history_path = config_dir.join(HISTORY_FILE);
        fs::write(history_path, path.display().to_string())?;
    }
    Ok(())
}

pub fn load_last_location() -> Option<PathBuf> {
    let config_dir = get_config_dir()?;
    let history_path = config_dir.join(HISTORY_FILE);

    if history_path.exists() {
        let content = fs::read_to_string(history_path).ok()?;
        let path = PathBuf::from(content.trim());

        // Verify path still exists
        if path.exists() && path.is_dir() {
            return Some(path);
        }
    }

    None
}

pub fn clear_last_location() -> std::io::Result<()> {
    if let Some(config_dir) = get_config_dir() {
        let history_path = config_dir.join(HISTORY_FILE);
        if history_path.exists() {
            fs::remove_file(history_path)?;
        }
    }
    Ok(())
}
