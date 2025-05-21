use std::path::Path;
use std::time::Duration;
use tracing::info;

/// Format duration as hours:minutes:seconds
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Check if a path exists and is a DVD
pub fn is_dvd_path(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    
    // Check for VIDEO_TS directory (standard DVD structure)
    if path.is_dir() && path.join("VIDEO_TS").exists() {
        return true;
    }
    
    // Check if it's a device file (Linux/macOS)
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileTypeExt;
        if let Ok(metadata) = path.metadata() {
            let file_type = metadata.file_type();
            if file_type.is_block_device() || file_type.is_char_device() {
                return true;
            }
        }
    }
    
    false
}

/// Compute aspect ratio from dimensions
pub fn compute_aspect_ratio(width: usize, height: usize) -> (usize, usize) {
    let gcd = gcd(width, height);
    (width / gcd, height / gcd)
}

/// Calculate greatest common divisor
fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Log system information for debugging
pub fn log_system_info() {
    info!("System information:");
    info!("  OS: {}", std::env::consts::OS);
    info!("  Architecture: {}", std::env::consts::ARCH);
    info!("  CPU cores: {}", num_cpus::get());
    
    #[cfg(target_os = "linux")]
    {
        if let Ok(release) = std::fs::read_to_string("/etc/os-release") {
            if let Some(line) = release.lines().find(|l| l.starts_with("PRETTY_NAME=")) {
                if let Some(distro) = line.split('=').nth(1) {
                    info!("  Distribution: {}", distro.trim_matches('"'));
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            info!("  macOS version: {}", version);
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("wmic").args(&["os", "get", "Caption"]).output() {
            let output = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = output.lines().nth(1) {
                info!("  Windows version: {}", line.trim());
            }
        }
    }
}