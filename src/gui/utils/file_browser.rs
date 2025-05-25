use crate::gui::state::UiState;
use std::path::PathBuf;

/// Browse for DVD input source (drive or folder)
pub fn browse_for_dvd_input(ui_state: &mut UiState) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Select DVD Drive or Folder")
        .pick_folder()
    {
        ui_state.input_path = path.to_string_lossy().to_string();
    }
}

/// Browse for output directory
pub fn browse_for_output_directory(ui_state: &mut UiState) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Select Output Directory")
        .pick_folder()
    {
        ui_state.output_path = path.to_string_lossy().to_string();
    }
}

/// Browse for HandBrake executable
pub fn browse_for_handbrake_executable(ui_state: &mut UiState) {
    let mut dialog = rfd::FileDialog::new()
        .set_title("Select HandBrakeCLI Executable");
    
    // Add platform-specific filters
    #[cfg(windows)]
    {
        dialog = dialog.add_filter("Executable", &["exe"]);
    }
    
    #[cfg(unix)]
    {
        dialog = dialog.add_filter("Executable", &["*"]);
    }
    
    if let Some(path) = dialog.pick_file() {
        ui_state.config_temp.handbrake_path = path.to_string_lossy().to_string();
    }
}

/// Browse for configuration file to import
pub fn browse_for_config_import() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Import Configuration")
        .add_filter("JSON", &["json"])
        .pick_file()
}

/// Browse for configuration file export location
pub fn browse_for_config_export() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Export Configuration")
        .add_filter("JSON", &["json"])
        .set_file_name("dvd_ripper_config.json")
        .save_file()
}

/// Browse for server profile import
pub fn browse_for_server_profile_import() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Import Server Profile")
        .add_filter("JSON", &["json"])
        .pick_file()
}

/// Browse for server profile export location
pub fn browse_for_server_profile_export() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Export Server Profile")
        .add_filter("JSON", &["json"])
        .set_file_name("server_profile.json")
        .save_file()
}

/// Get available DVD drives on the system
pub fn get_available_dvd_drives() -> Vec<PathBuf> {
    let mut drives = Vec::new();
    
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        
        unsafe {
            let drive_mask = winapi::um::fileapi::GetLogicalDrives();
            for i in 0..26 {
                if drive_mask & (1 << i) != 0 {
                    let drive_letter = ('A' as u8 + i) as char;
                    let drive_path = format!("{}:\\", drive_letter);
                    
                    let drive_type = winapi::um::fileapi::GetDriveTypeA(
                        std::ffi::CString::new(drive_path.clone()).unwrap().as_ptr()
                    );
                    
                    if drive_type == winapi::um::winbase::DRIVE_CDROM {
                        drives.push(PathBuf::from(drive_path));
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Check common mount points for optical drives
        let common_paths = [
            "/dev/sr0",
            "/dev/sr1",
            "/dev/cdrom",
            "/dev/dvd",
            "/media/cdrom",
            "/media/dvd",
            "/mnt/cdrom",
            "/mnt/dvd",
        ];
        
        for path in &common_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                drives.push(path_buf);
            }
        }
        
        // Also check /media and /mnt for mounted optical discs
        if let Ok(entries) = std::fs::read_dir("/media") {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        drives.push(entry.path());
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // Check /Volumes for mounted discs
        if let Ok(entries) = std::fs::read_dir("/Volumes") {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        let path = entry.path();
                        // Basic heuristic to identify optical media
                        if let Some(name) = path.file_name() {
                            let name_str = name.to_string_lossy().to_lowercase();
                            if name_str.contains("dvd") || 
                               name_str.contains("cd") || 
                               name_str.len() > 3 && !name_str.starts_with('.') {
                                drives.push(path);
                            }
                        }
                    }
                }
            }
        }
    }
    
    drives
}

/// Check if a path appears to be a DVD
pub fn is_dvd_path(path: &PathBuf) -> bool {
    // Check for VIDEO_TS folder (standard DVD structure)
    let video_ts = path.join("VIDEO_TS");
    if video_ts.exists() && video_ts.is_dir() {
        return true;
    }
    
    // Check for common DVD file patterns
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                let name_lower = name.to_lowercase();
                if name_lower.ends_with(".vob") || 
                   name_lower.ends_with(".ifo") || 
                   name_lower.ends_with(".bup") {
                    return true;
                }
            }
        }
    }
    
    false
}

/// Get a user-friendly name for a DVD path
pub fn get_dvd_display_name(path: &PathBuf) -> String {
    if let Some(file_name) = path.file_name() {
        file_name.to_string_lossy().to_string()
    } else {
        path.to_string_lossy().to_string()
    }
}

/// Validate that a path exists and is accessible
pub fn validate_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path cannot be empty".to_string());
    }
    
    let path_buf = PathBuf::from(path);
    
    if !path_buf.exists() {
        return Err("Path does not exist".to_string());
    }
    
    if !path_buf.is_dir() {
        return Err("Path must be a directory".to_string());
    }
    
    // Try to read the directory to check permissions
    match std::fs::read_dir(&path_buf) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Cannot access directory: {}", e)),
    }
}

/// Validate output directory and create if necessary
pub fn validate_output_directory(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Output path cannot be empty".to_string());
    }
    
    let path_buf = PathBuf::from(path);
    
    // If parent directory doesn't exist, try to create it
    if let Some(parent) = path_buf.parent() {
        if !parent.exists() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(format!("Cannot create output directory: {}", e));
            }
        }
    }
    
    // Create the directory if it doesn't exist
    if !path_buf.exists() {
        if let Err(e) = std::fs::create_dir_all(&path_buf) {
            return Err(format!("Cannot create output directory: {}", e));
        }
    }
    
    // Check if we can write to the directory
    let test_file = path_buf.join(".dvd_ripper_write_test");
    match std::fs::write(&test_file, "test") {
        Ok(_) => {
            let _ = std::fs::remove_file(&test_file);
            Ok(())
        }
        Err(e) => Err(format!("Cannot write to output directory: {}", e)),
    }
}