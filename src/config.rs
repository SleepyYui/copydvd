use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use directories::ProjectDirs;
use num_cpus;

use crate::error::{AppError, Result};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Path to HandBrakeCLI executable
    pub handbrake_path: Option<PathBuf>,
    
    /// Default output directory
    pub output_dir: PathBuf,
    
    /// Default encoding algorithm (x264 or x265)
    pub encode_algo: String,
    
    /// Whether to split chapters into separate files
    pub chapter_split: bool,
    
    /// Server settings for uploads
    pub server: Option<ServerConfig>,
    
    /// Whether to eject disc after ripping
    pub eject_after_rip: bool,
    
    /// Number of simultaneous ripping threads
    pub thread_count: usize,
    
    /// HandBrake management settings
    pub handbrake_management: HandBrakeManagementConfig,
}

/// Server configuration for uploads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub username: String,
    pub password: Option<String>,
    pub path: String,
}

/// HandBrake management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandBrakeManagementConfig {
    /// Whether to automatically download HandBrakeCLI if not found
    pub auto_download: bool,
    
    /// Whether to prefer system HandBrakeCLI over managed version
    pub prefer_system: bool,
    
    /// Maximum cache size in MB (0 = unlimited)
    pub max_cache_size_mb: u64,
    
    /// Whether to verify HandBrakeCLI on startup
    pub verify_on_startup: bool,
}

impl Default for Config {
    fn default() -> Self {
        let output_dir = dirs_base_path()
            .map(|p| p.join("outs"))
            .unwrap_or_else(|| PathBuf::from("./outs"));
        
        Self {
            handbrake_path: None,
            output_dir,
            encode_algo: "x264".to_string(),
            chapter_split: false,
            server: None,
            eject_after_rip: true,
            thread_count: num_cpus::get().max(1),
            handbrake_management: HandBrakeManagementConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from the default config file
    pub fn load() -> Result<Self> {
        let config_path = config_file_path()?;
        
        if !config_path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }
        
        let config_str = fs::read_to_string(&config_path)
            .map_err(|e| AppError::ConfigError(format!("Failed to read config file: {}", e)))?;
            
        serde_json::from_str(&config_str)
            .map_err(|e| AppError::ConfigError(format!("Failed to parse config file: {}", e)))
    }
    
    /// Save configuration to the default config file
    pub fn save(&self) -> Result<()> {
        let config_path = config_file_path()?;
        
        // Ensure directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::ConfigError(format!("Failed to create config directory: {}", e)))?;
        }
        
        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::ConfigError(format!("Failed to serialize config: {}", e)))?;
            
        fs::write(&config_path, config_str)
            .map_err(|e| AppError::ConfigError(format!("Failed to write config file: {}", e)))
    }
}

/// Get the base path for application directories
fn dirs_base_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "dvdripper", "DvdRipper")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

/// Get the path to the config file
fn config_file_path() -> Result<PathBuf> {
    dirs_base_path()
        .map(|p| p.join("config.json"))
        .ok_or_else(|| AppError::ConfigError("Failed to determine config directory".to_string()))
}

impl Default for HandBrakeManagementConfig {
    fn default() -> Self {
        Self {
            auto_download: true,
            prefer_system: true,
            max_cache_size_mb: 100,
            verify_on_startup: true,
        }
    }
}