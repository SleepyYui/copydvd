use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

const GITHUB_API_RELEASES: &str = "https://api.github.com/repos/sleepyyui/copydvd/releases";
const UPDATE_CHECK_TIMEOUT: Duration = Duration::from_secs(10);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub prerelease: bool,
    pub draft: bool,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub download_url: String,
    pub size: u64,
    pub content_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub published_at: String,
    pub prerelease: bool,
    pub draft: bool,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub content_type: String,
}

impl From<GitHubRelease> for Release {
    fn from(gh: GitHubRelease) -> Self {
        Self {
            tag_name: gh.tag_name,
            name: gh.name,
            body: gh.body,
            published_at: gh.published_at,
            prerelease: gh.prerelease,
            draft: gh.draft,
            assets: gh.assets.into_iter().map(Asset::from).collect(),
        }
    }
}

impl From<GitHubAsset> for Asset {
    fn from(gh: GitHubAsset) -> Self {
        Self {
            name: gh.name,
            download_url: gh.browser_download_url,
            size: gh.size,
            content_type: gh.content_type,
        }
    }
}

#[derive(Debug, Clone)]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable {
        current: String,
        latest: Release,
        changelog: String,
    },
    Error(String),
}

#[derive(Debug, Clone)]
pub enum UpdateProgress {
    Checking,
    Downloading {
        progress: f32,
    },
    Installing,
    Complete,
    #[allow(dead_code)]
    Failed(String),
}

#[derive(Debug)]
pub struct Updater {
    current_version: String,
    #[allow(dead_code)]
    repo_owner: String,
    #[allow(dead_code)]
    repo_name: String,
    platform_filter: String,
    client: reqwest::Client,
}

impl Updater {
    pub fn new() -> Self {
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        let platform_filter = Self::get_platform_filter();

        let client = reqwest::Client::builder()
            .user_agent(format!("CopyDVD/{}", current_version))
            .timeout(UPDATE_CHECK_TIMEOUT)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            current_version,
            repo_owner: "dvd-ripper".to_string(),
            repo_name: "copydvd".to_string(),
            platform_filter,
            client,
        }
    }

    fn get_platform_filter() -> String {
        #[cfg(target_os = "windows")]
        return "windows-msvc".to_string();

        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "x86_64")]
            return "x86_64-apple-darwin".to_string();
            #[cfg(target_arch = "aarch64")]
            return "aarch64-apple-darwin".to_string();
        }

        #[cfg(target_os = "linux")]
        return "x86_64-unknown-linux-gnu".to_string();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return "unknown".to_string();
    }

    pub async fn check_for_updates(&self) -> UpdateStatus {
        info!("Checking for updates...");

        match timeout(UPDATE_CHECK_TIMEOUT, self.fetch_releases()).await {
            Ok(Ok(releases)) => self.compare_versions(releases),
            Ok(Err(e)) => {
                error!("Failed to fetch releases: {}", e);
                UpdateStatus::Error(format!("Failed to check for updates: {}", e))
            }
            Err(_) => {
                warn!("Update check timed out");
                UpdateStatus::Error("Update check timed out".to_string())
            }
        }
    }

    async fn fetch_releases(
        &self,
    ) -> Result<Vec<Release>, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Fetching releases from GitHub API");

        let response = self
            .client
            .get(GITHUB_API_RELEASES)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("GitHub API returned status: {}", response.status()).into());
        }

        let github_releases: Vec<GitHubRelease> = response.json().await?;
        let releases: Vec<Release> = github_releases
            .into_iter()
            .filter(|r| !r.draft && !r.prerelease)
            .map(Release::from)
            .collect();

        debug!("Fetched {} releases", releases.len());
        Ok(releases)
    }

    fn compare_versions(&self, releases: Vec<Release>) -> UpdateStatus {
        let current = semver::Version::parse(self.current_version.trim_start_matches('v'))
            .unwrap_or_else(|_| semver::Version::new(0, 0, 0));

        // Clone releases for changelog generation
        let releases_for_changelog = releases.clone();

        let mut newer_releases = releases
            .into_iter()
            .filter_map(|release| {
                let version_str = release.tag_name.trim_start_matches('v');
                match semver::Version::parse(version_str) {
                    Ok(version) if version > current => Some((version, release)),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        if newer_releases.is_empty() {
            info!("No updates available");
            return UpdateStatus::UpToDate;
        }

        // Sort by version (newest first)
        newer_releases.sort_by(|(a, _), (b, _)| b.cmp(a));

        let (_latest_version, latest_release) = newer_releases.into_iter().next().unwrap();

        info!(
            "Update available: {} -> {}",
            self.current_version, latest_release.tag_name
        );

        UpdateStatus::UpdateAvailable {
            current: self.current_version.clone(),
            latest: latest_release,
            changelog: self
                .generate_changelog(&releases_for_changelog)
                .unwrap_or_default(),
        }
    }

    fn generate_changelog(&self, releases: &[Release]) -> Option<String> {
        let current = semver::Version::parse(self.current_version.trim_start_matches('v')).ok()?;

        let mut changelog = String::new();
        let mut relevant_releases: Vec<_> = releases
            .iter()
            .filter_map(|release| {
                let version_str = release.tag_name.trim_start_matches('v');
                match semver::Version::parse(version_str) {
                    Ok(version) if version > current => Some((version, release)),
                    _ => None,
                }
            })
            .collect();

        relevant_releases.sort_by(|(a, _), (b, _)| b.cmp(a));

        for (_, release) in relevant_releases {
            changelog.push_str(&format!("## {}\n", release.name));
            changelog.push_str(&format!("Released: {}\n\n", release.published_at));
            changelog.push_str(&release.body);
            changelog.push_str("\n\n---\n\n");
        }

        Some(changelog)
    }

    pub async fn download_and_install_update(
        &self,
        release: &Release,
        progress_callback: impl Fn(UpdateProgress) + Send + Sync + 'static,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        progress_callback(UpdateProgress::Checking);

        let asset = self.find_compatible_asset(release)?;
        info!("Found compatible asset: {}", asset.name);

        let download_path = self.get_download_path(&asset.name)?;

        progress_callback(UpdateProgress::Downloading { progress: 0.0 });
        self.download_asset(asset, &download_path, &progress_callback)
            .await?;

        progress_callback(UpdateProgress::Installing);
        self.install_update(&download_path, &asset.name).await?;

        progress_callback(UpdateProgress::Complete);
        info!("Update installation completed successfully");

        Ok(())
    }

    fn find_compatible_asset<'a>(
        &self,
        release: &'a Release,
    ) -> Result<&'a Asset, Box<dyn std::error::Error + Send + Sync>> {
        // Look for installer first, then fallback to binary
        let installer_patterns = self.get_installer_patterns();
        let binary_patterns = self.get_binary_patterns();

        // Try installer first
        for pattern in &installer_patterns {
            if let Some(asset) = release.assets.iter().find(|a| a.name.contains(pattern)) {
                return Ok(asset);
            }
        }

        // Fallback to binary
        for pattern in &binary_patterns {
            if let Some(asset) = release.assets.iter().find(|a| a.name.contains(pattern)) {
                return Ok(asset);
            }
        }

        Err(format!(
            "No compatible asset found for platform: {}",
            self.platform_filter
        )
        .into())
    }

    fn get_installer_patterns(&self) -> Vec<&str> {
        #[cfg(target_os = "windows")]
        return vec![".msi"];

        #[cfg(target_os = "macos")]
        return vec![".app.zip"];

        #[cfg(target_os = "linux")]
        return vec![".deb", ".rpm", ".appimage"];

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return vec![];
    }

    fn get_binary_patterns(&self) -> Vec<&str> {
        vec![&self.platform_filter]
    }

    fn get_download_path(
        &self,
        filename: &str,
    ) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let download_dir = dirs::download_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join("Downloads")))
            .ok_or("Could not determine download directory")?;

        Ok(download_dir.join(format!("copydvd_update_{}", filename)))
    }

    async fn download_asset(
        &self,
        asset: &Asset,
        download_path: &Path,
        progress_callback: &(impl Fn(UpdateProgress) + Send + Sync),
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Downloading {} to {:?}", asset.name, download_path);

        let client = reqwest::Client::builder()
            .timeout(DOWNLOAD_TIMEOUT)
            .build()?;

        let response = client.get(&asset.download_url).send().await?;

        if !response.status().is_success() {
            return Err(format!("Download failed with status: {}", response.status()).into());
        }

        let total_size = asset.size;
        let mut downloaded = 0u64;
        let mut file = fs::File::create(download_path).await?;

        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            let progress = if total_size > 0 {
                downloaded as f32 / total_size as f32
            } else {
                0.0
            };

            progress_callback(UpdateProgress::Downloading { progress });
        }

        file.flush().await?;
        info!("Download completed: {} bytes", downloaded);

        Ok(())
    }

    async fn install_update(
        &self,
        download_path: &Path,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_os = "windows")]
        {
            if filename.ends_with(".msi") {
                return self.install_windows_msi(download_path).await;
            } else if filename.ends_with(".exe") {
                return self.install_windows_exe(download_path).await;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if filename.ends_with(".app.zip") {
                return self.install_macos_app(download_path).await;
            }
        }

        #[cfg(target_os = "linux")]
        {
            if filename.ends_with(".deb") {
                return self.install_linux_deb(download_path).await;
            } else if filename.ends_with(".rpm") {
                return self.install_linux_rpm(download_path).await;
            } else if filename.ends_with(".appimage") {
                return self.install_linux_appimage(download_path).await;
            }
        }

        // Fallback: replace current binary
        self.replace_binary(download_path).await
    }

    #[cfg(target_os = "windows")]
    async fn install_windows_msi(
        &self,
        msi_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::process::Command;

        info!("Installing Windows MSI: {:?}", msi_path);

        // For MSI installations, we can install directly since MSI handles the process
        let output = Command::new("msiexec")
            .args(&["/i", &msi_path.to_string_lossy(), "/quiet", "/norestart"])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("MSI installation failed: {}", error).into());
        }

        // Clean up downloaded file
        let _ = fs::remove_file(msi_path).await;

        // Schedule restart notification
        self.schedule_restart_notification().await?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn install_windows_exe(
        &self,
        exe_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Installing Windows EXE: {:?}", exe_path);

        // For EXE replacements on Windows, we need to use a different strategy
        // since we can't replace a running binary
        let current_exe = std::env::current_exe()?;
        let backup_path = current_exe.with_extension("backup");
        let update_script_path = current_exe.with_file_name("update_copydvd.bat");

        // Create a batch script that will:
        // 1. Wait for the current process to exit
        // 2. Move the current exe to backup
        // 3. Move the new exe to replace the current one
        // 4. Start the new executable
        // 5. Delete itself
        let update_script = format!(
            r#"@echo off
echo Updating Copy DVD...
timeout /t 2 /nobreak >nul
move "{}" "{}"
move "{}" "{}"
start "" "{}"
del "%~f0"
"#,
            current_exe.to_string_lossy(),
            backup_path.to_string_lossy(),
            exe_path.to_string_lossy(),
            current_exe.to_string_lossy(),
            current_exe.to_string_lossy()
        );

        fs::write(&update_script_path, update_script).await?;

        // Make the script executable and hidden
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            use tokio::process::Command;

            // Hide the script file
            let _ = Command::new("attrib")
                .args(&["+H", &update_script_path.to_string_lossy()])
                .output()
                .await;
        }

        info!("Update script created at: {:?}", update_script_path);

        // Return success - the actual update will happen when the app exits and the script runs
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn schedule_restart_notification(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // This would integrate with the UI to show a restart notification
        info!("Update installed successfully. Application restart required.");
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub async fn execute_pending_update_and_restart(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::process::Command;

        let current_exe = std::env::current_exe()?;
        let update_script_path = current_exe.with_file_name("update_copydvd.bat");

        if update_script_path.exists() {
            info!("Executing pending update and restarting...");

            // Start the update script in the background
            Command::new("cmd")
                .args(&["/C", "start", "", &update_script_path.to_string_lossy()])
                .spawn()?;

            // Exit the current process to allow the update to proceed
            std::process::exit(0);
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn install_macos_app(
        &self,
        app_zip_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::process::Command;

        info!("Installing macOS app: {:?}", app_zip_path);

        // Extract to temporary directory
        let temp_dir = std::env::temp_dir().join("copydvd_update");
        fs::create_dir_all(&temp_dir).await?;

        let output = Command::new("unzip")
            .args([
                "-o",
                &app_zip_path.to_string_lossy(),
                "-d",
                &temp_dir.to_string_lossy(),
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Err("Failed to extract app bundle".into());
        }

        // Move to Applications
        let app_path = temp_dir.join("Copy DVD.app");
        let applications_path = Path::new("/Applications/Copy DVD.app");

        // Remove old version if it exists
        let _ = fs::remove_dir_all(applications_path).await;

        let output = Command::new("mv")
            .arg(app_path.as_os_str())
            .arg(applications_path.as_os_str())
            .output()
            .await?;

        if !output.status.success() {
            return Err("Failed to install app to Applications folder".into());
        }

        // Clean up
        let _ = fs::remove_file(app_zip_path).await;
        let _ = fs::remove_dir_all(&temp_dir).await;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn install_linux_deb(
        &self,
        deb_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::process::Command;

        info!("Installing Linux DEB: {:?}", deb_path);

        let output = Command::new("pkexec")
            .args(["dpkg", "-i", &deb_path.to_string_lossy()])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("DEB installation failed: {}", error).into());
        }

        let _ = fs::remove_file(deb_path).await;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn install_linux_rpm(
        &self,
        rpm_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::process::Command;

        info!("Installing Linux RPM: {:?}", rpm_path);

        let output = Command::new("pkexec")
            .args(["rpm", "-U", &rpm_path.to_string_lossy()])
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("RPM installation failed: {}", error).into());
        }

        let _ = fs::remove_file(rpm_path).await;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn install_linux_appimage(
        &self,
        appimage_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Installing Linux AppImage: {:?}", appimage_path);

        let install_dir = dirs::home_dir()
            .ok_or("Could not determine home directory")?
            .join(".local/bin");

        fs::create_dir_all(&install_dir).await?;

        let target_path = install_dir.join("copydvd");
        fs::copy(appimage_path, &target_path).await?;

        // Make executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_path).await?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms).await?;

        let _ = fs::remove_file(appimage_path).await;
        Ok(())
    }

    async fn replace_binary(
        &self,
        new_binary_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Replacing current binary with: {:?}", new_binary_path);

        let current_exe = std::env::current_exe()?;
        let backup_path = current_exe.with_extension("backup");

        // Create backup
        fs::copy(&current_exe, &backup_path).await?;

        // Replace binary
        fs::copy(new_binary_path, &current_exe).await?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&current_exe).await?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&current_exe, perms).await?;
        }

        // Clean up
        let _ = fs::remove_file(new_binary_path).await;

        info!("Binary replacement completed");
        Ok(())
    }

    pub async fn get_release_history(
        &self,
    ) -> Result<Vec<Release>, Box<dyn std::error::Error + Send + Sync>> {
        self.fetch_releases().await
    }

    #[allow(dead_code)]
    pub fn get_current_version(&self) -> &str {
        &self.current_version
    }
}

#[cfg(test)]
mod tests;
