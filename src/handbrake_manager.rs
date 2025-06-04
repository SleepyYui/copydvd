use crate::error::{AppError, Result};
use crate::handbrake_auto_fix::MacOSAutoFix;
use anyhow::Context;
use directories::ProjectDirs;

use sha2::{Digest, Sha256};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{info, warn};
use zip::ZipArchive;

// Progress callback type for UI updates
#[derive(Debug, Clone)]
pub enum HandBrakePhase {
    Downloading,
    Extracting,
    Installing,
    Verifying,
}

pub type ProgressCallback = Box<dyn Fn(HandBrakePhase, f32) + Send + Sync>;

const HANDBRAKE_VERSION: &str = "1.9.2";
const HANDBRAKE_BASE_URL: &str = "https://github.com/HandBrake/HandBrake/releases/download";

pub struct HandBrakeManager {
    cache_dir: PathBuf,
    binary_path: Option<PathBuf>,
    progress_callback: Option<ProgressCallback>,
}

#[derive(Debug)]
struct PlatformInfo {
    download_url: String,
    binary_name: String,
    expected_sha256: Option<String>,
}

impl HandBrakeManager {
    pub fn new() -> Result<Self> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            binary_path: None,
            progress_callback: None,
        })
    }

    #[allow(dead_code)]
    pub fn set_progress_callback(&mut self, callback: ProgressCallback) {
        self.progress_callback = Some(callback);
    }

    fn update_progress(&self, phase: HandBrakePhase, progress: f32) {
        if let Some(callback) = &self.progress_callback {
            callback(phase, progress);
        }
    }

    fn get_cache_dir() -> Result<PathBuf> {
        ProjectDirs::from("com", "sleepyyui", "copydvd")
            .map(|proj_dirs| proj_dirs.cache_dir().join("handbrake"))
            .ok_or_else(|| {
                AppError::HandbrakeError("Failed to determine cache directory".to_string())
            })
    }

    pub async fn get_handbrake_path(&mut self) -> Result<PathBuf> {
        info!("Starting HandBrake path resolution...");

        // If we already have a cached path, return it
        if let Some(path) = &self.binary_path {
            if path.exists() {
                info!("Using already cached HandBrake path: {}", path.display());
                return Ok(path.clone());
            } else {
                warn!("Cached HandBrake path no longer exists: {}", path.display());
                self.binary_path = None;
            }
        }

        // Check if HandBrake is available in system PATH
        info!("Checking for HandBrake in system PATH...");

        // On macOS, try both "HandBrake" and "HandBrakeCLI"
        let possible_names = if cfg!(target_os = "macos") {
            vec!["HandBrakeCLI", "HandBrake", "handbrake"]
        } else {
            vec!["HandBrake", "handbrake"]
        };

        for name in possible_names {
            if let Ok(system_path) = which::which(name) {
                info!(
                    "Found HandBrake in system PATH: {} -> {}",
                    name,
                    system_path.display()
                );
                self.binary_path = Some(system_path.clone());
                return Ok(system_path);
            }
        }
        info!("HandBrake not found in system PATH");

        // Check if we have a cached binary
        let cached_binary = self.get_cached_binary_path()?;
        info!(
            "Checking for cached HandBrake at: {}",
            cached_binary.display()
        );
        if cached_binary.exists() {
            info!("Found cached HandBrake: {}", cached_binary.display());
            self.binary_path = Some(cached_binary.clone());
            return Ok(cached_binary);
        }
        info!("No cached HandBrake found");

        // Download and cache HandBrake
        info!("HandBrake not found anywhere. Starting download process...");
        self.download_handbrake().await?;

        let binary_path = self.get_cached_binary_path()?;
        if !binary_path.exists() {
            let error_msg = format!(
                "Failed to download HandBrake - binary not found at expected path: {}",
                binary_path.display()
            );
            warn!("{}", error_msg);
            return Err(AppError::HandbrakeError(error_msg));
        }

        info!(
            "HandBrake successfully downloaded to: {}",
            binary_path.display()
        );
        self.binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }

    fn get_cached_binary_path(&self) -> Result<PathBuf> {
        let platform_info = Self::get_platform_info()?;
        Ok(self.cache_dir.join(&platform_info.binary_name))
    }

    async fn download_handbrake(&self) -> Result<()> {
        let platform_info = Self::get_platform_info()?;

        info!("=== Starting HandBrake Download ===");
        info!(
            "Platform: {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
        info!("Download URL: {}", platform_info.download_url);
        info!("Target binary: {}", platform_info.binary_name);
        info!("Cache directory: {}", self.cache_dir.display());

        // Create HTTP client with timeout and user agent
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout
            .user_agent("CopyDVD/1.0")
            .build()
            .map_err(|e| {
                AppError::HandbrakeError(format!("Failed to create HTTP client: {}", e))
            })?;

        info!("Sending HTTP request to download HandBrake...");
        let response = client
            .get(&platform_info.download_url)
            .send()
            .await
            .with_context(|| format!("Failed to download from {}", platform_info.download_url))
            .map_err(|e| {
                warn!("HTTP request failed: {}", e);
                AppError::HandbrakeError(e.to_string())
            })?;

        info!("HTTP Response Status: {}", response.status());
        info!("Response Headers: {:#?}", response.headers());

        if !response.status().is_success() {
            let error_msg = format!("Failed to download HandBrake: HTTP {}", response.status());
            warn!("{}", error_msg);
            return Err(AppError::HandbrakeError(error_msg));
        }

        // Get content length for progress tracking
        let content_length = response.content_length();
        if let Some(length) = content_length {
            info!(
                "Download size: {} bytes ({:.2} MB)",
                length,
                length as f64 / 1024.0 / 1024.0
            );
        } else {
            info!("Download size: unknown (no Content-Length header)");
        }

        info!("Starting streaming download...");
        let mut stream = response.bytes_stream();
        let mut downloaded_bytes = Vec::new();
        let mut total_downloaded = 0u64;

        // Stream the download with progress tracking
        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .with_context(|| "Failed to read chunk from download stream")
                .map_err(|e| {
                    warn!("Failed to read chunk: {}", e);
                    AppError::HandbrakeError(e.to_string())
                })?;

            downloaded_bytes.extend_from_slice(&chunk);
            total_downloaded += chunk.len() as u64;

            // Update progress and log
            if let Some(total_size) = content_length {
                let download_progress = total_downloaded as f64 / total_size as f64;
                self.update_progress(HandBrakePhase::Downloading, download_progress as f32);
                if total_downloaded % (1024 * 1024) == 0 || download_progress >= 0.99 {
                    // Log every MB or at completion
                    info!(
                        "Download progress: {:.1}% ({}/{} bytes)",
                        download_progress * 100.0,
                        total_downloaded,
                        total_size
                    );
                }
            } else {
                // Without content length, estimate progress based on downloaded size
                let download_progress =
                    (total_downloaded as f64 / (20.0 * 1024.0 * 1024.0)).min(1.0); // Assume ~20MB max
                self.update_progress(HandBrakePhase::Downloading, download_progress as f32);
                if total_downloaded % (1024 * 1024) == 0 {
                    // Log every MB
                    info!("Downloaded: {} bytes", total_downloaded);
                }
            }
        }

        let bytes = bytes::Bytes::from(downloaded_bytes);
        info!("Download complete. Total downloaded: {} bytes", bytes.len());

        // Verify checksum if available
        if let Some(expected_hash) = &platform_info.expected_sha256 {
            info!("Verifying checksum...");
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual_hash = format!("{:x}", hasher.finalize());

            if &actual_hash != expected_hash {
                let error_msg = format!(
                    "Checksum verification failed. Expected: {}, Got: {}",
                    expected_hash, actual_hash
                );
                warn!("{}", error_msg);
                return Err(AppError::HandbrakeError(error_msg));
            }
            info!("Checksum verification passed");
        } else {
            info!("No checksum provided, skipping verification");
        }

        info!("Extracting HandBrake binary...");
        self.update_progress(HandBrakePhase::Extracting, 0.0);
        self.extract_binary(&bytes, &platform_info).await?;
        self.update_progress(HandBrakePhase::Verifying, 0.0);

        info!("=== HandBrake Download Complete ===");
        Ok(())
    }

    async fn extract_binary(
        &self,
        archive_bytes: &[u8],
        platform_info: &PlatformInfo,
    ) -> Result<()> {
        let cursor = std::io::Cursor::new(archive_bytes);

        if cfg!(target_os = "macos") {
            self.extract_from_dmg(archive_bytes, platform_info).await
        } else if cfg!(target_os = "windows") {
            self.extract_from_zip(cursor, platform_info).await
        } else {
            Err(AppError::HandbrakeError(
                "Automatic HandBrake download not supported on this platform. Please install HandBrake manually.".to_string()
            ))
        }
    }

    async fn extract_from_zip(
        &self,
        cursor: std::io::Cursor<&[u8]>,
        platform_info: &PlatformInfo,
    ) -> Result<()> {
        info!("Opening ZIP archive for extraction...");
        let mut archive = ZipArchive::new(cursor)
            .with_context(|| "Failed to open ZIP archive")
            .map_err(|e| {
                warn!("Failed to open ZIP archive: {}", e);
                AppError::HandbrakeError(e.to_string())
            })?;

        info!("ZIP archive contains {} files", archive.len());

        // List all files in the archive for debugging
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                info!("Archive file {}: {}", i, file.name());
            }
        }

        // Look for HandBrake.exe in the archive
        info!("Looking for HandBrake executable in archive...");
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .with_context(|| format!("Failed to access file {} in archive", i))
                .map_err(|e| {
                    warn!("Failed to access archive file {}: {}", i, e);
                    AppError::HandbrakeError(e.to_string())
                })?;

            let file_name = file.name();
            info!("Checking file: {}", file_name);

            if file_name.ends_with("HandBrake.exe")
                || file_name.ends_with(&platform_info.binary_name)
            {
                info!("Found HandBrake executable: {}", file_name);
                let target_path = self.cache_dir.join(&platform_info.binary_name);
                info!("Extracting to: {}", target_path.display());

                let mut output = fs::File::create(&target_path)
                    .with_context(|| format!("Failed to create file: {}", target_path.display()))
                    .map_err(|e| {
                        warn!("Failed to create output file: {}", e);
                        AppError::HandbrakeError(e.to_string())
                    })?;

                let bytes_copied = std::io::copy(&mut file, &mut output)
                    .with_context(|| "Failed to extract HandBrake executable")
                    .map_err(|e| {
                        warn!("Failed to extract executable: {}", e);
                        AppError::HandbrakeError(e.to_string())
                    })?;

                info!(
                    "Successfully extracted {} bytes to: {}",
                    bytes_copied,
                    target_path.display()
                );

                // Make executable on Unix systems
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&target_path)
                        .map_err(|e| {
                            AppError::HandbrakeError(format!("Failed to get file metadata: {}", e))
                        })?
                        .permissions();
                    perms.set_mode(0o755); // rwxr-xr-x
                    fs::set_permissions(&target_path, perms).map_err(|e| {
                        AppError::HandbrakeError(format!(
                            "Failed to set executable permissions: {}",
                            e
                        ))
                    })?;
                    info!("Set executable permissions on {}", target_path.display());
                }

                return Ok(());
            }
        }

        let error_msg = format!(
            "HandBrake executable not found in downloaded archive. Looking for: {}",
            platform_info.binary_name
        );
        warn!("{}", error_msg);
        Err(AppError::HandbrakeError(error_msg))
    }

    async fn extract_from_dmg(&self, dmg_bytes: &[u8], platform_info: &PlatformInfo) -> Result<()> {
        info!("=== macOS DMG Extraction ===");
        info!("Received DMG file: {} bytes", dmg_bytes.len());

        // Write DMG to temporary file
        self.update_progress(HandBrakePhase::Extracting, 0.0);
        let temp_dmg = self.cache_dir.join("handbrake_temp.dmg");
        std::fs::write(&temp_dmg, dmg_bytes).map_err(|e| {
            AppError::HandbrakeError(format!("Failed to write DMG to temporary file: {}", e))
        })?;
        info!("Wrote DMG to: {}", temp_dmg.display());

        // Mount the DMG
        info!("Mounting DMG...");
        let mount_output = Command::new("hdiutil")
            .args(["attach", "-nobrowse", "-readonly"])
            .arg(&temp_dmg)
            .output()
            .map_err(|e| AppError::HandbrakeError(format!("Failed to mount DMG: {}", e)))?;

        if !mount_output.status.success() {
            let error = String::from_utf8_lossy(&mount_output.stderr);
            warn!("Failed to mount DMG: {}", error);
            return Err(AppError::HandbrakeError(format!(
                "Failed to mount DMG: {}",
                error
            )));
        }

        let mount_info = String::from_utf8_lossy(&mount_output.stdout);
        info!("Mount output: {}", mount_info);

        // Parse mount point from output
        let mount_point = mount_info
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 && parts[0].starts_with("/dev/disk") {
                    Some(parts[2])
                } else {
                    None
                }
            })
            .next()
            .ok_or_else(|| {
                AppError::HandbrakeError("Could not determine mount point".to_string())
            })?;

        info!("DMG mounted at: {}", mount_point);

        // Find HandBrake.app in the mounted volume
        let app_path = std::path::Path::new(mount_point).join("HandBrake.app");
        if !app_path.exists() {
            // Try to find it with different case or location
            let mount_dir = std::fs::read_dir(mount_point).map_err(|e| {
                AppError::HandbrakeError(format!("Failed to read mount directory: {}", e))
            })?;

            let mut found_app = None;
            for entry in mount_dir {
                let entry = entry?;
                let name = entry.file_name();
                if name.to_string_lossy().to_lowercase().contains("handbrake")
                    && name.to_string_lossy().ends_with(".app")
                {
                    found_app = Some(entry.path());
                    break;
                }
            }

            let app_path = found_app.ok_or_else(|| {
                AppError::HandbrakeError("HandBrake.app not found in DMG".to_string())
            })?;

            info!("Found HandBrake app at: {}", app_path.display());
        }

        // Extract HandBrakeCLI from the app bundle - try multiple possible locations
        let possible_cli_paths = vec![
            app_path.join("Contents/MacOS/HandBrakeCLI"),
            app_path.join("Contents/Resources/HandBrakeCLI"),
            app_path.join("HandBrakeCLI"),
            app_path.join("Contents/MacOS/HandBrake"),
        ];

        let mut cli_source = None;
        for path in &possible_cli_paths {
            info!("Checking for HandBrakeCLI at: {}", path.display());
            if path.exists() {
                cli_source = Some(path.clone());
                break;
            }
        }

        let cli_source = match cli_source {
            Some(path) => path,
            None => {
                // List contents of app bundle for debugging
                if let Ok(contents_dir) = std::fs::read_dir(app_path.join("Contents")) {
                    info!("Contents of {}/Contents:", app_path.display());
                    for entry in contents_dir.flatten() {
                        info!("  {}", entry.file_name().to_string_lossy());
                    }
                }

                if let Ok(macos_dir) = std::fs::read_dir(app_path.join("Contents/MacOS")) {
                    info!("Contents of {}/Contents/MacOS:", app_path.display());
                    for entry in macos_dir.flatten() {
                        info!("  {}", entry.file_name().to_string_lossy());
                    }
                }

                let error_msg = format!(
                    "HandBrakeCLI not found in any expected locations within {}. Tried: {}",
                    app_path.display(),
                    possible_cli_paths
                        .iter()
                        .map(|p| p.to_string_lossy())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                warn!("{}", error_msg);

                // Unmount before returning error
                let _ = Command::new("hdiutil")
                    .args(["detach"])
                    .arg(mount_point)
                    .output();
                std::fs::remove_file(&temp_dmg).ok();

                return Err(AppError::HandbrakeError(error_msg));
            }
        };

        info!("Found HandBrakeCLI at: {}", cli_source.display());
        self.update_progress(HandBrakePhase::Installing, 0.0);

        // Copy HandBrakeCLI to our cache directory
        let cli_dest = self.cache_dir.join(&platform_info.binary_name);
        std::fs::copy(&cli_source, &cli_dest)
            .map_err(|e| AppError::HandbrakeError(format!("Failed to copy HandBrakeCLI: {}", e)))?;

        // Make it executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&cli_dest)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&cli_dest, perms)?;
        }

        info!(
            "Successfully copied HandBrakeCLI to: {}",
            cli_dest.display()
        );

        // Unmount the DMG
        info!("Unmounting DMG...");
        let unmount_output = Command::new("hdiutil")
            .args(["detach"])
            .arg(mount_point)
            .output();

        if let Ok(output) = unmount_output {
            if output.status.success() {
                info!("DMG unmounted successfully");
            } else {
                warn!(
                    "Failed to unmount DMG: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }

        // Clean up temporary DMG file
        if let Err(e) = std::fs::remove_file(&temp_dmg) {
            warn!("Failed to remove temporary DMG file: {}", e);
        }

        info!("macOS DMG extraction completed successfully");
        Ok(())
    }

    fn get_platform_info() -> Result<PlatformInfo> {
        let version = HANDBRAKE_VERSION;

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            Ok(PlatformInfo {
                download_url: format!(
                    "{}/{}/HandBrake-{}-win-x86_64.zip",
                    HANDBRAKE_BASE_URL, version, version
                ),
                binary_name: "HandBrake.exe".to_string(),
                expected_sha256: None, // Add actual checksums from HandBrake releases if needed
            })
        }

        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        {
            Ok(PlatformInfo {
                download_url: format!(
                    "{}/{}/HandBrake-{}-win-aarch64.zip",
                    HANDBRAKE_BASE_URL, version, version
                ),
                binary_name: "HandBrake.exe".to_string(),
                expected_sha256: None,
            })
        }

        #[cfg(target_os = "macos")]
        {
            Ok(PlatformInfo {
                download_url: format!(
                    "{}/{}/HandBrake-{}.dmg",
                    HANDBRAKE_BASE_URL, version, version
                ),
                binary_name: "HandBrakeCLI".to_string(),
                expected_sha256: None,
            })
        }

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            // For Linux, we'll recommend package manager installation
            Err(AppError::HandbrakeError(
                "Automatic download not available for Linux. Please install HandBrake using your package manager:\n\
                 Ubuntu/Debian: sudo apt install handbrake-cli\n\
                 Fedora: sudo dnf install handbrake-cli\n\
                 Arch: sudo pacman -S handbrake-cli".to_string()
            ))
        }

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            Err(AppError::HandbrakeError(
                "Automatic download not available for Linux ARM64. Please install HandBrake using your package manager:\n\
                 Ubuntu/Debian: sudo apt install handbrake-cli\n\
                 Fedora: sudo dnf install handbrake-cli\n\
                 Arch: sudo pacman -S handbrake-cli".to_string()
            ))
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(AppError::HandbrakeError(
                "Unsupported platform for automatic HandBrake download".to_string(),
            ))
        }
    }

    /// Check if HandBrake exists without triggering download
    pub fn handbrake_exists(&self) -> bool {
        if let Ok(binary_path) = self.get_cached_binary_path() {
            binary_path.exists()
        } else {
            false
        }
    }

    pub async fn verify_handbrake(&mut self) -> Result<String> {
        info!("=== Verifying HandBrake Installation ===");
        let binary_path = self.get_handbrake_path().await?;
        info!("Verifying HandBrake at: {}", binary_path.display());

        // Check if file exists and is executable
        if !binary_path.exists() {
            let error_msg = format!("HandBrake binary not found at: {}", binary_path.display());
            warn!("{}", error_msg);
            return Err(AppError::HandbrakeError(error_msg));
        }

        let metadata = fs::metadata(&binary_path)
            .map_err(|e| AppError::HandbrakeError(format!("Failed to get file metadata: {}", e)))?;
        info!("HandBrake file size: {} bytes", metadata.len());

        // Prepare binary for execution (handle permissions and security attributes)
        self.prepare_binary_for_execution(&binary_path)?;

        // Try to execute HandBrake --version
        info!("Executing HandBrake --version...");
        let output = Command::new(&binary_path)
            .arg("--version")
            .output()
            .with_context(|| format!("Failed to execute HandBrake: {}", binary_path.display()))
            .map_err(|e| {
                warn!("Failed to execute HandBrake: {}", e);
                AppError::HandbrakeError(e.to_string())
            })?;

        info!("HandBrake exit status: {}", output.status);
        info!(
            "HandBrake stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        info!(
            "HandBrake stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        if !output.status.success() {
            let error_msg = if output.status.code().is_none() {
                // Process was terminated by signal (like SIGKILL)
                warn!("HandBrake was terminated by system signal (SIGKILL), attempting automatic quarantine removal...");

                #[cfg(target_os = "macos")]
                {
                    info!(
                        "HandBrake was terminated by macOS, attempting automatic security fixes..."
                    );

                    // Attempt automatic fixes using the dedicated module
                    match MacOSAutoFix::attempt_all_fixes(&binary_path) {
                        Ok(fixes_applied) if fixes_applied => {
                            info!("Automatic macOS security fixes applied successfully, retrying HandBrake...");
                            // Try running HandBrake again after automatic fixes
                            let retry_output = Command::new(&binary_path)
                                .arg("--version")
                                .output()
                                .with_context(|| {
                                    format!(
                                        "Failed to retry HandBrake execution: {}",
                                        binary_path.display()
                                    )
                                })
                                .map_err(|e| AppError::HandbrakeError(e.to_string()))?;

                            if retry_output.status.success() {
                                info!(
                                    "HandBrake execution successful after automatic security fixes"
                                );
                                let version_output = String::from_utf8_lossy(&retry_output.stdout);
                                let stderr_output = String::from_utf8_lossy(&retry_output.stderr);
                                let version_info = if !version_output.trim().is_empty() {
                                    version_output.trim()
                                } else {
                                    stderr_output.trim()
                                };
                                info!(
                                    "HandBrake verification successful. Version info: {}",
                                    version_info
                                );
                                return Ok(binary_path.to_string_lossy().to_string());
                            } else {
                                warn!("HandBrake still failing after automatic security fixes");
                            }
                        }
                        Ok(_) => {
                            info!("No automatic fixes were applied");
                        }
                        Err(e) => {
                            warn!("Failed to apply automatic fixes: {}", e);
                        }
                    }

                    format!(
                        "HandBrake was terminated by macOS Gatekeeper ({}). Automatic security fixes were attempted.\n\
                        \n\
                        ⚠️  ACTION REQUIRED: macOS is still blocking this unsigned binary.\n\
                        \n\
                        The application automatically tried to fix this, but manual intervention may be needed:\n\
                        \n\
                        1. Try running the application again (sometimes it takes a moment)\n\
                        2. If still blocked, manually run this command:\n\
                           {:?} --version\n\
                        3. Then go to System Settings > Privacy & Security\n\
                        4. Look for 'HandBrakeCLI was blocked' message\n\
                        5. Click 'Allow Anyway' next to the message\n\
                        6. Try running the application again\n\
                        \n\
                        The app tried these automatic fixes:\n\
                        - Removed quarantine attributes\n\
                        - Attempted to open Security preferences\n\
                        - Set appropriate file permissions\n\
                        \n\
                        stderr: {}",
                        output.status,
                        binary_path,
                        String::from_utf8_lossy(&output.stderr)
                    )
                }
                #[cfg(not(target_os = "macos"))]
                format!(
                    "HandBrake was terminated by the system ({}). stderr: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                )
            } else {
                format!(
                    "HandBrake failed to execute properly. Exit code: {}, stderr: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                )
            };
            warn!("{}", error_msg);
            return Err(AppError::HandbrakeError(error_msg));
        }

        let version_output = String::from_utf8_lossy(&output.stdout);
        let stderr_output = String::from_utf8_lossy(&output.stderr);

        // HandBrakeCLI often outputs version info to stderr
        let version_info = if !version_output.trim().is_empty() {
            version_output.trim()
        } else {
            stderr_output.trim()
        };

        info!(
            "HandBrake verification successful. Version info: {}",
            version_info
        );

        Ok(binary_path.to_string_lossy().to_string())
    }

    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)
                .with_context(|| {
                    format!(
                        "Failed to clear cache directory: {}",
                        self.cache_dir.display()
                    )
                })
                .map_err(|e| AppError::HandbrakeError(e.to_string()))?;

            fs::create_dir_all(&self.cache_dir)
                .with_context(|| {
                    format!(
                        "Failed to recreate cache directory: {}",
                        self.cache_dir.display()
                    )
                })
                .map_err(|e| AppError::HandbrakeError(e.to_string()))?;
        }

        info!("HandBrake cache cleared");
        Ok(())
    }

    pub fn get_cache_info(&self) -> Result<(PathBuf, u64)> {
        let mut total_size = 0u64;

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)
                .with_context(|| {
                    format!(
                        "Failed to read cache directory: {}",
                        self.cache_dir.display()
                    )
                })
                .map_err(|e| AppError::HandbrakeError(e.to_string()))?
            {
                let entry = entry
                    .with_context(|| "Failed to read directory entry")
                    .map_err(|e| AppError::HandbrakeError(e.to_string()))?;

                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }

        Ok((self.cache_dir.clone(), total_size))
    }

    /// Prepare a binary for execution by setting permissions and removing security attributes
    fn prepare_binary_for_execution(&self, binary_path: &Path) -> Result<()> {
        info!("Preparing binary for execution: {}", binary_path.display());

        // Set execute permissions
        let mut perms = fs::metadata(binary_path)
            .with_context(|| format!("Failed to get metadata for {}", binary_path.display()))
            .map_err(|e| AppError::HandbrakeError(e.to_string()))?
            .permissions();

        #[cfg(unix)]
        {
            // Ensure the owner has execute permission
            let mode = perms.mode();
            perms.set_mode(mode | 0o700); // rwx for owner
            fs::set_permissions(binary_path, perms)
                .with_context(|| {
                    format!(
                        "Failed to set execute permissions on {}",
                        binary_path.display()
                    )
                })
                .map_err(|e| AppError::HandbrakeError(e.to_string()))?;
            info!("Set execute permissions on binary");
        }

        // On macOS, remove quarantine attributes that might prevent execution
        #[cfg(target_os = "macos")]
        {
            let _ = self.remove_macos_quarantine(binary_path)?;
        }

        info!("Binary preparation completed");
        Ok(())
    }

    /// Remove macOS quarantine attributes with automatic sudo elevation if needed
    #[cfg(target_os = "macos")]
    fn remove_macos_quarantine(&self, binary_path: &Path) -> Result<bool> {
        info!("Removing macOS quarantine attributes...");

        // Check if quarantine attribute exists first
        let has_quarantine = self.has_quarantine_attribute(binary_path);
        info!("Binary has quarantine attribute: {}", has_quarantine);

        if has_quarantine {
            // For downloaded binaries that are being killed by SIGKILL,
            // go straight to elevated removal since normal xattr usually fails
            let removed = self.remove_quarantine_with_password_prompt(binary_path)?;

            // Also try to remove other common quarantine attributes
            let _ = Command::new("xattr")
                .args(["-d", "com.apple.metadata:kMDItemWhereFroms"])
                .arg(binary_path)
                .output();

            Ok(removed)
        } else {
            info!("No quarantine attribute found on binary");
            Ok(true) // Nothing to remove, consider it successful
        }
    }

    /// Remove quarantine with password prompt using AppleScript
    #[cfg(target_os = "macos")]
    fn remove_quarantine_with_password_prompt(&self, binary_path: &Path) -> Result<bool> {
        info!("Prompting user for password to remove quarantine attributes...");

        let script = format!(
            r#"do shell script "xattr -d com.apple.quarantine '{}'" with administrator privileges"#,
            binary_path.display()
        );

        let output = Command::new("osascript").args(["-e", &script]).output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    info!("Successfully removed quarantine attributes with user elevation");
                    Ok(true)
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    if stderr.contains("User canceled") {
                        info!("User canceled elevation request for quarantine removal");
                        Ok(false)
                    } else if stderr.contains("No such xattr: com.apple.quarantine") {
                        info!("Quarantine attribute is already missing from binary");
                        Ok(true) // Not an error - attribute is already gone
                    } else {
                        warn!(
                            "Failed to remove quarantine attributes with elevation: {}",
                            stderr
                        );
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                warn!("Failed to execute AppleScript elevation: {}", e);
                Ok(false)
            }
        }
    }

    /// Check if quarantine attribute exists on the binary
    #[cfg(target_os = "macos")]
    fn has_quarantine_attribute(&self, binary_path: &Path) -> bool {
        MacOSAutoFix::has_quarantine_attribute(binary_path)
    }
}

impl std::fmt::Debug for HandBrakeManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandBrakeManager")
            .field("cache_dir", &self.cache_dir)
            .field("binary_path", &self.binary_path)
            .field("progress_callback", &self.progress_callback.is_some())
            .finish()
    }
}

impl Default for HandBrakeManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            cache_dir: std::env::temp_dir(),
            binary_path: None,
            progress_callback: None,
        })
    }
}
