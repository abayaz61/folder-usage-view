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

/// Get the console output handle via CONOUT$ (works in alternate screen mode).
/// Opened once and cached to avoid handle leaks.
fn get_console_handle() -> Result<windows::Win32::Foundation::HANDLE, String> {
    use std::os::windows::io::AsRawHandle;
    use std::sync::OnceLock;

    static HANDLE: OnceLock<isize> = OnceLock::new();

    let raw = HANDLE.get_or_init(|| {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("CONOUT$")
            .expect("Failed to open CONOUT$");
        let raw = file.as_raw_handle() as isize;
        std::mem::forget(file);
        raw
    });

    Ok(windows::Win32::Foundation::HANDLE(*raw as _))
}

/// Check if we're running inside Windows Terminal (vs legacy conhost.exe)
fn is_windows_terminal() -> bool {
    std::env::var("WT_SESSION").is_ok()
}

/// Get the path to Windows Terminal's settings.json
fn get_wt_settings_path() -> Option<PathBuf> {
    let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
    let base = PathBuf::from(local_app_data).join("Packages");

    // Check stable WT first, then preview
    let candidates = [
        "Microsoft.WindowsTerminal_8wekyb3d8bbwe",
        "Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe",
    ];

    for package in &candidates {
        let path = base
            .join(package)
            .join("LocalState")
            .join("settings.json");
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Strip single-line comments (//) and trailing commas from JSONC content.
/// Windows Terminal settings.json uses JSONC format.
fn strip_jsonc(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_string = false;
    let mut chars = input.chars().peekable();

    // Pass 1: strip // comments
    while let Some(ch) = chars.next() {
        if in_string {
            out.push(ch);
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
            out.push(ch);
        } else if ch == '/' && chars.peek() == Some(&'/') {
            for c in chars.by_ref() {
                if c == '\n' {
                    out.push('\n');
                    break;
                }
            }
        } else {
            out.push(ch);
        }
    }

    // Pass 2: remove trailing commas before } or ]
    // Use regex-like approach: find ",\s*[}\]]" and remove the comma
    let bytes = out.as_bytes();
    let mut result = String::with_capacity(out.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b',' {
            // Check if only whitespace follows before } or ]
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t' || bytes[j] == b'\n' || bytes[j] == b'\r') {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                // Skip the comma
                i += 1;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

/// Get the active WT profile GUID from environment.
fn get_wt_profile_id() -> Option<String> {
    std::env::var("WT_PROFILE_ID").ok()
}

/// Find the active profile object in WT settings and return a mutable reference to its font object.
/// Creates the font object if it doesn't exist.
fn get_wt_active_profile_font_mut(json: &mut serde_json::Value) -> Result<&mut serde_json::Map<String, serde_json::Value>, String> {
    let profile_id = get_wt_profile_id()
        .ok_or("WT_PROFILE_ID not set")?;

    let list = json
        .pointer_mut("/profiles/list")
        .and_then(|v| v.as_array_mut())
        .ok_or("profiles.list not found")?;

    let profile = list.iter_mut()
        .find(|p| p.get("guid").and_then(|g| g.as_str()) == Some(&profile_id))
        .ok_or(format!("Profile {} not found in WT settings", profile_id))?;

    let profile_obj = profile
        .as_object_mut()
        .ok_or("profile is not an object")?;

    let font = profile_obj
        .entry("font")
        .or_insert_with(|| serde_json::json!({}));

    font.as_object_mut()
        .ok_or_else(|| "font is not an object".to_string())
}

/// Read WT settings.json, parse it, return (json, path).
fn read_wt_settings() -> Result<(serde_json::Value, PathBuf), String> {
    let path = get_wt_settings_path()
        .ok_or("Could not find Windows Terminal settings.json")?;

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read WT settings: {}", e))?;

    let clean = strip_jsonc(&content);
    let json: serde_json::Value = serde_json::from_str(&clean)
        .map_err(|e| format!("Failed to parse WT settings: {}", e))?;

    Ok((json, path))
}

/// Write WT settings.json back.
fn write_wt_settings(json: &serde_json::Value, path: &PathBuf) -> Result<(), String> {
    let output = serde_json::to_string_pretty(json)
        .map_err(|e| format!("Failed to serialize WT settings: {}", e))?;

    fs::write(path, output)
        .map_err(|e| format!("Failed to write WT settings: {}", e))
}

/// Read font info from the active profile in WT settings.json.
fn get_wt_font() -> (String, u16) {
    let (json, _path) = match read_wt_settings() {
        Ok(v) => v,
        Err(_) => return ("Cascadia Mono".to_string(), 12),
    };

    let profile_id = match get_wt_profile_id() {
        Some(id) => id,
        None => return ("Cascadia Mono".to_string(), 12),
    };

    // Find active profile's font
    let font = json
        .pointer("/profiles/list")
        .and_then(|list| list.as_array())
        .and_then(|list| list.iter().find(|p| p.get("guid").and_then(|g| g.as_str()) == Some(&profile_id)))
        .and_then(|p| p.get("font"));

    if let Some(font) = font {
        let face = font["face"].as_str().unwrap_or("Cascadia Mono").to_string();
        let size = font["size"].as_u64().unwrap_or(12) as u16;
        return (face, size);
    }

    // Fallback: check profiles.defaults.font
    let defaults_font = &json["profiles"]["defaults"]["font"];
    let face = defaults_font["face"].as_str().unwrap_or("Cascadia Mono").to_string();
    let size = defaults_font["size"].as_u64().unwrap_or(12) as u16;
    (face, size)
}

/// Set font on the active profile in WT settings.json (both face and size in one write).
fn set_wt_font(name: &str, size: u16) -> Result<(), String> {
    let (mut json, path) = read_wt_settings()?;

    let font_obj = get_wt_active_profile_font_mut(&mut json)?;
    font_obj.insert("face".to_string(), serde_json::json!(name));
    font_obj.insert("size".to_string(), serde_json::json!(size));

    write_wt_settings(&json, &path)
}

/// Get the current console font name and size.
/// Routes to Windows Terminal settings or legacy console API depending on the host.
pub fn get_console_font() -> (String, u16) {
    if is_windows_terminal() {
        return get_wt_font();
    }

    use windows::Win32::System::Console::*;

    let handle = match get_console_handle() {
        Ok(h) => h,
        Err(_) => return ("Consolas".to_string(), 16),
    };

    unsafe {
        let mut info = CONSOLE_FONT_INFOEX {
            cbSize: std::mem::size_of::<CONSOLE_FONT_INFOEX>() as u32,
            ..Default::default()
        };
        if GetCurrentConsoleFontEx(handle, false, &mut info).is_ok() {
            let name = String::from_utf16_lossy(
                &info.FaceName[..info.FaceName.iter().position(|&c| c == 0).unwrap_or(info.FaceName.len())]
            );
            let size = info.dwFontSize.Y as u16;
            (name, size)
        } else {
            ("Consolas".to_string(), 16)
        }
    }
}

/// Write a debug line to a log file next to the executable.
fn debug_log(msg: &str) {
    use std::io::Write;
    let path = std::env::temp_dir().join("dua_font_debug.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "[{}] {}", chrono::Local::now().format("%H:%M:%S%.3f"), msg);
    }
}

/// Set the console font name and size.
/// Routes to Windows Terminal settings or legacy console API depending on the host.
pub fn set_console_font(name: &str, size: u16) -> Result<(), String> {
    debug_log(&format!("set_console_font called: name={}, size={}", name, size));
    debug_log(&format!("WT_SESSION={:?}", std::env::var("WT_SESSION")));
    debug_log(&format!("is_windows_terminal={}", is_windows_terminal()));

    if is_windows_terminal() {
        debug_log("Taking WT path (active profile)");
        let r = set_wt_font(name, size);
        debug_log(&format!("set_wt_font result: {:?}", r));
        return r;
    }

    debug_log("Taking conhost path");

    use windows::Win32::System::Console::*;

    let handle = get_console_handle()?;

    unsafe {
        let mut info = CONSOLE_FONT_INFOEX {
            cbSize: std::mem::size_of::<CONSOLE_FONT_INFOEX>() as u32,
            ..Default::default()
        };

        // Get current font info as base
        let _ = GetCurrentConsoleFontEx(handle, false, &mut info);

        // Set font size
        info.dwFontSize.X = 0;
        info.dwFontSize.Y = size as i16;
        info.FontWeight = 400; // FW_NORMAL

        // Set font name
        let name_wide: Vec<u16> = name.encode_utf16().collect();
        info.FaceName = [0u16; 32];
        let copy_len = name_wide.len().min(31);
        info.FaceName[..copy_len].copy_from_slice(&name_wide[..copy_len]);

        SetCurrentConsoleFontEx(handle, false, &info)
            .map_err(|e| format!("Failed to set console font: {}", e))
    }
}

/// Get a list of available monospace console fonts
pub fn get_available_fonts() -> Vec<String> {
    vec![
        "Consolas".to_string(),
        "Lucida Console".to_string(),
        "Courier New".to_string(),
        "Cascadia Mono".to_string(),
        "Cascadia Code".to_string(),
        "Terminal".to_string(),
        "Fira Code".to_string(),
        "JetBrains Mono".to_string(),
        "Source Code Pro".to_string(),
    ]
}
