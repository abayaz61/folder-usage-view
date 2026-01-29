use std::path::PathBuf;
use sysinfo::Disks;

#[derive(Debug, Clone)]
pub struct DriveInfo {
    pub name: String,
    pub mount_point: PathBuf,
    pub total_space: u64,
    pub available_space: u64,
    pub used_space: u64,
    pub file_system: String,
    pub is_removable: bool,
}

impl DriveInfo {
    pub fn usage_percentage(&self) -> f64 {
        if self.total_space == 0 {
            return 0.0;
        }
        (self.used_space as f64 / self.total_space as f64) * 100.0
    }

    pub fn display_name(&self) -> String {
        if self.name.is_empty() {
            self.mount_point.display().to_string()
        } else {
            format!("{} ({})", self.mount_point.display(), self.name)
        }
    }
}

pub fn get_all_drives() -> Vec<DriveInfo> {
    let disks = Disks::new_with_refreshed_list();

    disks
        .iter()
        .map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);

            DriveInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_path_buf(),
                total_space: total,
                available_space: available,
                used_space: used,
                file_system: disk.file_system().to_string_lossy().to_string(),
                is_removable: disk.is_removable(),
            }
        })
        .collect()
}
