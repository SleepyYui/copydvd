use crate::error::{Result, AppError};
use anyhow::Context;
use directories::ProjectDirs;
use reqwest;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn, debug};
use zip::ZipArchive;

const HANDBRAKE_VERSION: &str = "1.9.2";
const HANDBRAKE_BASE_URL: &str = "https://github.com/HandBrake/HandBrake/releases/download";

#[derive(Debug, Clone)]
pub struct HandBrakeManager {
    cache_dir: PathBuf,
    binary_path: Option<PathBuf>,
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
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))
            .map_err(|e| AppError::HandbrakeError(e.to_string()))?;

        Ok(Self {
            cache_dir,
            binary_path: None,
        })
    }

    fn get_cache_dir() -> Result<PathBuf> {
        ProjectDirs::from("com", "dvd-ripper", "dvd-ripper")
            .map(|proj_dirs| proj_dirs.cache_dir().join("handbrake"))
            .ok_or_else(|| AppError::HandbrakeError("Failed to determine cache directory".to_string()))
    }

    pub async fn get_handbrake_path(&mut self) -> Result<PathBuf> {
        // If we already have a cached path, return it
        if let Some(path) = &self.binary_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // Check if HandBrakeCLI is available in system PATH
        if let Ok(system_path) = which::which("HandBrakeCLI") {
            info!("Found HandBrakeCLI in system PATH: {}", system_path.display());
            self.binary_path = Some(system_path.clone());
            return Ok(system_path);
        }

        // Check if we have a cached binary
        let cached_binary = self.get_cached_binary_path()?;
        if cached_binary.exists() {
            info!("Found cached HandBrakeCLI: {}", cached_binary.display());
            self.binary_path = Some(cached_binary.clone());
            return Ok(cached_binary);
        }

        // Download and cache HandBrakeCLI
        info!("HandBrakeCLI not found. Downloading and caching...");
        self.download_handbrake().await?;

        let binary_path = self.get_cached_binary_path()?;
        if !binary_path.exists() {
            return Err(AppError::HandbrakeError(
                "Failed to download HandBrakeCLI".to_string()
            ));
        }

        self.binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }

    fn get_cached_binary_path(&self) -> Result<PathBuf> {
        let platform_info = Self::get_platform_info()?;
        Ok(self.cache_dir.join(&platform_info.binary_name))
    }

    async fn download_handbrake(&self) -> Result<()> {
        let platform_info = Self::get_platform_info()?;
        
        info!("Downloading HandBrakeCLI from: {}", platform_info.download_url);

        let client = reqwest::Client::new();
        let response = client
            .get(&platform_info.download_url)
            .send()
            .await
            .with_context(|| format!("Failed to download from {}", platform_info.download_url))
            .map_err(|e| AppError::HandbrakeError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AppError::HandbrakeError(format!(
                "Failed to download HandBrakeCLI: HTTP {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .with_context(|| "Failed to read download response")
            .map_err(|e| AppError::HandbrakeError(e.to_string()))?;

        // Verify checksum if available
        if let Some(expected_hash) = &platform_info.expected_sha256 {
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual_hash = format!("{:x}", hasher.finalize());
            
            if &actual_hash != expected_hash {
                return Err(AppError::HandbrakeError(format!(
                    "Checksum verification failed. Expected: {}, Got: {}",
                    expected_hash, actual_hash
                )));
            }
            debug!("Checksum verification passed");
        }

        self.extract_binary(&bytes, &platform_info).await?;
        
        info!("HandBrakeCLI downloaded and cached successfully");
        Ok(())
    }

    async fn extract_binary(&self, archive_bytes: &[u8], platform_info: &PlatformInfo) -> Result<()> {
        let cursor = std::io::Cursor::new(archive_bytes);
        
        if cfg!(target_os = "macos") {
            self.extract_from_dmg(archive_bytes, platform_info).await
        } else if cfg!(target_os = "windows") {
            self.extract_from_zip(cursor, platform_info).await
        } else {
            Err(AppError::HandbrakeError(
                "Automatic HandBrakeCLI download not supported on this platform. Please install HandBrakeCLI manually.".to_string()
            ))
        }
    }

    async fn extract_from_zip(&self, cursor: std::io::Cursor<&[u8]>, platform_info: &PlatformInfo) -> Result<()> {
        let mut archive = ZipArchive::new(cursor)
            .with_context(|| "Failed to open ZIP archive")
            .map_err(|e| AppError::HandbrakeError(e.to_string()))?;

        // Look for HandBrakeCLI.exe in the archive
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .with_context(|| format!("Failed to access file {} in archive", i))
                .map_err(|e| AppError::HandbrakeError(e.to_string()))?;

            if file.name().ends_with("HandBrakeCLI.exe") {
                let target_path = self.cache_dir.join(&platform_info.binary_name);
                let mut output = fs::File::create(&target_path)
                    .with_context(|| format!("Failed to create file: {}", target_path.display()))
                    .map_err(|e| AppError::HandbrakeError(e.to_string()))?;

                std::io::copy(&mut file, &mut output)
                    .with_context(|| "Failed to extract HandBrakeCLI.exe")
                    .map_err(|e| AppError::HandbrakeError(e.to_string()))?;

                info!("Extracted HandBrakeCLI.exe to: {}", target_path.display());
                return Ok(());
            }
        }

        Err(AppError::HandbrakeError(
            "HandBrakeCLI.exe not found in downloaded archive".to_string()
        ))
    }

    async fn extract_from_dmg(&self, _dmg_bytes: &[u8], _platform_info: &PlatformInfo) -> Result<()> {
        // For macOS, we'll use a simpler approach: instruct users to install via Homebrew
        // or provide a more complex DMG extraction (which requires additional dependencies)
        warn!("DMG extraction not implemented. Falling back to system installation check.");
        
        // Check if user has Homebrew and can install HandBrakeCLI
        if Command::new("brew").arg("--version").output().is_ok() {
            return Err(AppError::HandbrakeError(
                "HandBrakeCLI not found. Please install it using: brew install handbrake".to_string()
            ));
        }

        Err(AppError::HandbrakeError(
            "HandBrakeCLI not found. Please download and install HandBrake from https://handbrake.fr/".to_string()
        ))
    }

    fn get_platform_info() -> Result<PlatformInfo> {
        let version = HANDBRAKE_VERSION;
        
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            Ok(PlatformInfo {
                download_url: format!("{}/{}/HandBrakeCLI-{}-win-x86_64.zip", HANDBRAKE_BASE_URL, version, version),
                binary_name: "HandBrakeCLI.exe".to_string(),
                expected_sha256: None, // Add actual checksums from HandBrake releases if needed
            })
        }

        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        {
            Ok(PlatformInfo {
                download_url: format!("{}/{}/HandBrakeCLI-{}-win-aarch64.zip", HANDBRAKE_BASE_URL, version, version),
                binary_name: "HandBrakeCLI.exe".to_string(),
                expected_sha256: None,
            })
        }

        #[cfg(target_os = "macos")]
        {
            Ok(PlatformInfo {
                download_url: format!("{}/{}/HandBrakeCLI-{}.dmg", HANDBRAKE_BASE_URL, version, version),
                binary_name: "HandBrakeCLI".to_string(),
                expected_sha256: None,
            })
        }

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            // For Linux, we'll recommend package manager installation
            Err(AppError::HandbrakeError(
                "Automatic download not available for Linux. Please install HandBrakeCLI using your package manager:\n\
                 Ubuntu/Debian: sudo apt install handbrake-cli\n\
                 Fedora: sudo dnf install handbrake-cli\n\
                 Arch: sudo pacman -S handbrake-cli".to_string()
            ))
        }

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        {
            Err(AppError::HandbrakeError(
                "Automatic download not available for Linux ARM64. Please install HandBrakeCLI using your package manager:\n\
                 Ubuntu/Debian: sudo apt install handbrake-cli\n\
                 Fedora: sudo dnf install handbrake-cli\n\
                 Arch: sudo pacman -S handbrake-cli".to_string()
            ))
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(AppError::HandbrakeError(
                "Unsupported platform for automatic HandBrakeCLI download".to_string()
            ))
        }
    }

    pub async fn verify_handbrake(&mut self) -> Result<String> {
        let binary_path = self.get_handbrake_path().await?;
        
        let output = Command::new(&binary_path)
            .arg("--version")
            .output()
            .with_context(|| format!("Failed to execute HandBrakeCLI: {}", binary_path.display()))
            .map_err(|e| AppError::HandbrakeError(e.to_string()))?;

        if !output.status.success() {
            return Err(AppError::HandbrakeError(
                "HandBrakeCLI failed to execute properly".to_string()
            ));
        }

        let version_output = String::from_utf8_lossy(&output.stdout);
        info!("HandBrakeCLI version: {}", version_output.trim());
        
        Ok(binary_path.to_string_lossy().to_string())
    }

    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)
                .with_context(|| format!("Failed to clear cache directory: {}", self.cache_dir.display()))
                .map_err(|e| AppError::HandbrakeError(e.to_string()))?;
            
            fs::create_dir_all(&self.cache_dir)
                .with_context(|| format!("Failed to recreate cache directory: {}", self.cache_dir.display()))
                .map_err(|e| AppError::HandbrakeError(e.to_string()))?;
        }
        
        info!("HandBrake cache cleared");
        Ok(())
    }

    pub fn get_cache_info(&self) -> Result<(PathBuf, u64)> {
        let mut total_size = 0u64;
        
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)
                .with_context(|| format!("Failed to read cache directory: {}", self.cache_dir.display()))
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
}

impl Default for HandBrakeManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            cache_dir: PathBuf::from(".handbrake_cache"),
            binary_path: None,
        })
    }
}