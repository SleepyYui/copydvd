use crate::error::{AppError, Result};
use std::path::PathBuf;

/// Auto-detect DVD drive path
pub fn auto_detect_dvd() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        for path in &["/dev/sr0", "/dev/cdrom"] {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(entries) = std::fs::read_dir("/Volumes") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("VIDEO_TS").exists() {
                    return Ok(path);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        for drive in b'D'..=b'Z' {
            let drive_path_str = format!("{}:\\", drive as char);
            let drive_path = PathBuf::from(&drive_path_str);
            if drive_path.exists() && drive_path.join("VIDEO_TS").exists() {
                return Ok(drive_path);
            }
        }
    }
    Err(AppError::DvdNotFound(PathBuf::from("auto-detected DVD")))
}
