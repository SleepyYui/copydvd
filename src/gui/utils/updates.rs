use crate::gui::state::{UiState, UpdateStatus};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// GitHub release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub html_url: String,
    pub assets: Vec<GitHubAsset>,
    pub published_at: String,
    pub prerelease: bool,
    pub draft: bool,
}

/// GitHub release asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub content_type: String,
}

/// Update check result
#[derive(Debug, Clone)]
pub enum UpdateCheckResult {
    UpToDate,
    UpdateAvailable {
        version: String,
        download_url: String,
        changelog: String,
    },
    Error(String),
}

/// Check for updates from GitHub releases
pub async fn check_for_updates() -> UpdateCheckResult {
    const GITHUB_API_URL: &str = "https://api.github.com/repos/dvdripper/dvdripper/releases/latest";
    const USER_AGENT: &str = concat!("dvd-ripper/", env!("CARGO_PKG_VERSION"));
    
    let client = match reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(client) => client,
        Err(e) => return UpdateCheckResult::Error(format!("Failed to create HTTP client: {}", e)),
    };
    
    let response = match client.get(GITHUB_API_URL).send().await {
        Ok(response) => response,
        Err(e) => return UpdateCheckResult::Error(format!("Failed to fetch release info: {}", e)),
    };
    
    if !response.status().is_success() {
        return UpdateCheckResult::Error(format!("HTTP error: {}", response.status()));
    }
    
    let release: GitHubRelease = match response.json().await {
        Ok(release) => release,
        Err(e) => return UpdateCheckResult::Error(format!("Failed to parse release info: {}", e)),
    };
    
    // Skip draft and pre-releases
    if release.draft || release.prerelease {
        return UpdateCheckResult::UpToDate;
    }
    
    let current_version = env!("CARGO_PKG_VERSION");
    let latest_version = release.tag_name.trim_start_matches('v');
    
    if is_newer_version(current_version, latest_version) {
        // Find the appropriate download URL for current platform
        let download_url = find_download_url_for_platform(&release.assets)
            .unwrap_or_else(|| release.html_url.clone());
        
        UpdateCheckResult::UpdateAvailable {
            version: latest_version.to_string(),
            download_url,
            changelog: release.body,
        }
    } else {
        UpdateCheckResult::UpToDate
    }
}

/// Compare version strings to determine if one is newer
fn is_newer_version(current: &str, latest: &str) -> bool {
    let current_parts: Vec<u32> = current
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    
    let latest_parts: Vec<u32> = latest
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    
    let max_len = current_parts.len().max(latest_parts.len());
    
    for i in 0..max_len {
        let current_part = current_parts.get(i).copied().unwrap_or(0);
        let latest_part = latest_parts.get(i).copied().unwrap_or(0);
        
        if latest_part > current_part {
            return true;
        } else if latest_part < current_part {
            return false;
        }
    }
    
    false
}

/// Find the download URL for the current platform
fn find_download_url_for_platform(assets: &[GitHubAsset]) -> Option<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    
    // Look for platform-specific assets
    for asset in assets {
        let name = asset.name.to_lowercase();
        
        // Match platform patterns
        let matches_os = match os {
            "windows" => name.contains("windows") || name.contains("win") || name.ends_with(".exe"),
            "macos" => name.contains("macos") || name.contains("darwin") || name.contains("osx"),
            "linux" => name.contains("linux") || name.contains("gnu"),
            _ => false,
        };
        
        let matches_arch = match arch {
            "x86_64" => name.contains("x64") || name.contains("x86_64") || name.contains("amd64"),
            "aarch64" => name.contains("arm64") || name.contains("aarch64"),
            _ => true, // Default to true for other architectures
        };
        
        if matches_os && matches_arch {
            return Some(asset.browser_download_url.clone());
        }
    }
    
    None
}

/// Check if automatic update checking is enabled
pub fn should_check_for_updates() -> bool {
    // TODO: Add configuration option for automatic update checking
    true
}

/// Get the last update check time
pub fn get_last_update_check() -> Option<SystemTime> {
    // TODO: Implement persistent storage of last check time
    None
}

/// Set the last update check time
pub fn set_last_update_check(time: SystemTime) {
    // TODO: Implement persistent storage of last check time
}

/// Check if enough time has passed since last update check
pub fn should_check_for_updates_now() -> bool {
    const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60); // 24 hours
    
    if !should_check_for_updates() {
        return false;
    }
    
    match get_last_update_check() {
        Some(last_check) => {
            match SystemTime::now().duration_since(last_check) {
                Ok(elapsed) => elapsed >= CHECK_INTERVAL,
                Err(_) => true, // If time calculation fails, check anyway
            }
        }
        None => true, // First time checking
    }
}

/// Perform automatic update check if needed
pub async fn auto_check_for_updates(ui_state: &mut UiState) {
    if !should_check_for_updates_now() {
        return;
    }
    
    ui_state.checking_updates = true;
    
    match check_for_updates().await {
        UpdateCheckResult::UpdateAvailable { version, download_url, changelog: _ } => {
            ui_state.update_status = UpdateStatus::UpdateAvailable {
                version,
                url: download_url,
            };
            ui_state.show_update_dialog = true;
        }
        UpdateCheckResult::UpToDate => {
            ui_state.update_status = UpdateStatus::UpToDate;
        }
        UpdateCheckResult::Error(error) => {
            ui_state.update_status = UpdateStatus::Error(error);
        }
    }
    
    ui_state.checking_updates = false;
    set_last_update_check(SystemTime::now());
}

/// Manual update check triggered by user
pub async fn manual_check_for_updates(ui_state: &mut UiState) {
    ui_state.checking_updates = true;
    ui_state.set_status("Checking for updates...".to_string());
    
    match check_for_updates().await {
        UpdateCheckResult::UpdateAvailable { version, download_url, changelog: _ } => {
            ui_state.update_status = UpdateStatus::UpdateAvailable {
                version: version.clone(),
                url: download_url,
            };
            ui_state.set_status(format!("Update available: version {}", version));
            ui_state.show_update_dialog = true;
        }
        UpdateCheckResult::UpToDate => {
            ui_state.update_status = UpdateStatus::UpToDate;
            ui_state.set_status("You have the latest version".to_string());
        }
        UpdateCheckResult::Error(error) => {
            ui_state.update_status = UpdateStatus::Error(error.clone());
            ui_state.set_error(format!("Update check failed: {}", error));
        }
    }
    
    ui_state.checking_updates = false;
    set_last_update_check(SystemTime::now());
}

/// Open the download URL in the default browser
pub fn open_download_url(url: &str) {
    if let Err(e) = open::that(url) {
        eprintln!("Failed to open download URL: {}", e);
    }
}

/// Get changelog for a specific version
pub async fn get_changelog(version: &str) -> Result<String, String> {
    const GITHUB_API_URL: &str = "https://api.github.com/repos/dvdripper/dvdripper/releases";
    const USER_AGENT: &str = concat!("dvd-ripper/", env!("CARGO_PKG_VERSION"));
    
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client
        .get(GITHUB_API_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch releases: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    let releases: Vec<GitHubRelease> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse releases: {}", e))?;
    
    for release in releases {
        let release_version = release.tag_name.trim_start_matches('v');
        if release_version == version {
            return Ok(release.body);
        }
    }
    
    Err(format!("Changelog not found for version {}", version))
}

/// Format changelog for display
pub fn format_changelog(changelog: &str) -> String {
    // Basic markdown to text conversion
    changelog
        .lines()
        .map(|line| {
            let line = line.trim();
            if line.starts_with("##") {
                format!("\n{}\n{}", line.trim_start_matches('#').trim(), "=".repeat(40))
            } else if line.starts_with('#') {
                format!("\n{}\n{}", line.trim_start_matches('#').trim(), "-".repeat(30))
            } else if line.starts_with("- ") || line.starts_with("* ") {
                format!("  • {}", line.trim_start_matches(&['-', '*', ' '][..]).trim())
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}