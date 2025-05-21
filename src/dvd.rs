use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::process::Command;
use std::collections::HashMap;
use std::fs;
use regex::Regex;
use tracing::{info, warn, debug};
use tokio::process::Command as TokioCommand;
use tokio::time::sleep;

use crate::error::{AppError, Result};
use crate::config::Config;

/// Represents DVD title information
#[derive(Debug, Clone)]
pub struct Title {
    pub number: usize,
    pub duration: Duration,
    pub chapters: Vec<Chapter>,
    pub audio_tracks: HashMap<usize, AudioTrack>,
    pub subtitle_tracks: HashMap<usize, SubtitleTrack>,
    pub size: VideoSize,
}

/// Represents a chapter in a DVD title
#[derive(Debug, Clone)]
pub struct Chapter {
    pub number: usize,
    pub duration: Duration,
}

/// Represents an audio track in a DVD title
#[derive(Debug, Clone)]
pub struct AudioTrack {
    pub number: usize,
    pub language: String,
    pub codec: String,
    pub channels: usize,
    pub iso639_2: String,
}

/// Represents a subtitle track in a DVD title
#[derive(Debug, Clone)]
pub struct SubtitleTrack {
    pub number: usize,
    pub language: String,
}

/// Represents video dimensions and framerate
#[derive(Debug, Clone)]
pub struct VideoSize {
    pub width: usize,
    pub height: usize,
    pub pixel_aspect_width: usize,
    pub pixel_aspect_height: usize,
    pub fps: f64,
}

/// Represents a ripping task
#[derive(Debug, Clone)]
pub struct RipTask {
    pub title: Title,
    pub chapter: Option<usize>,
    pub output_path: PathBuf,
}

/// DVD handling and ripping
#[derive(Debug)]
pub struct Dvd {
    pub path: PathBuf,
    pub mount_point: Option<PathBuf>,
    pub titles: Vec<Title>,
    pub config: Config,
}

impl Dvd {
    /// Create a new DVD instance
    pub async fn new(path: impl AsRef<Path>, config: Config) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut dvd = Self {
            path,
            mount_point: None,
            titles: Vec::new(),
            config,
        };
        
        // Detect mount point or mount the DVD
        dvd.detect_mount_point().await?;
        
        Ok(dvd)
    }
    
    /// Detect or create mount point for the DVD
    async fn detect_mount_point(&mut self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            if self.path.metadata()?.file_type().is_block_device() {
                self.mount_point = Some(find_mount_point(&self.path).await?);
            } else if self.path.is_dir() {
                self.mount_point = Some(self.path.clone());
            } else {
                return Err(AppError::DvdNotFound(self.path.clone()));
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if self.path.is_dir() {
                self.mount_point = Some(self.path.clone());
            } else {
                return Err(AppError::DvdNotFound(self.path.clone()));
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            if self.path.is_dir() {
                self.mount_point = Some(self.path.clone());
            } else {
                return Err(AppError::DvdNotFound(self.path.clone()));
            }
        }
        
        debug!("DVD mount point detected at: {:?}", self.mount_point);
        Ok(())
    }
    
    /// Scan the DVD for titles
    pub async fn scan_titles(&mut self) -> Result<()> {
        info!("Scanning DVD titles...");
        
        let mount_point = self.mount_point.as_ref()
            .ok_or_else(|| AppError::MountFailed("DVD not mounted".to_string()))?;
            
        // Find HandBrakeCLI path
        let handbrake = self.config.handbrake_path.clone()
            .unwrap_or_else(|| PathBuf::from("HandBrakeCLI"));
            
        // Verify HandBrakeCLI is available
        let handbrake_path = which::which(&handbrake)
            .map_err(|_| AppError::HandBrakeNotFound)?;
        
        info!("Using HandBrakeCLI at: {}", handbrake_path.display());
        
        // Execute HandBrakeCLI scan command
        let output = TokioCommand::new(&handbrake_path)
            .arg("--scan")
            .arg("--title").arg("1")
            .arg("-i").arg(mount_point)
            .output()
            .await
            .map_err(|e| AppError::CommandFailed(format!("Failed to execute HandBrakeCLI: {}", e)))?;
            
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Parse title count
        let title_count = parse_title_count(&stderr)
            .ok_or_else(|| AppError::DvdParseFailed("Failed to parse title count".to_string()))?;
            
        info!("Found {} titles on DVD", title_count);
        
        // Parse first title details
        let mut titles = Vec::with_capacity(title_count);
        let first_title = parse_title_details(1, &stderr)?;
        titles.push(first_title);
        
        // Scan remaining titles
        for title_num in 2..=title_count {
            match self.scan_title(title_num).await {
                Ok(title) => titles.push(title),
                Err(e) => warn!("Failed to scan title {}: {}", title_num, e),
            }
        }
        
        if titles.is_empty() {
            return Err(AppError::NoTitlesFound);
        }
        
        self.titles = titles;
        Ok(())
    }
    
    /// Scan a specific title
    async fn scan_title(&self, title_number: usize) -> Result<Title> {
        let mount_point = self.mount_point.as_ref()
            .ok_or_else(|| AppError::MountFailed("DVD not mounted".to_string()))?;
            
        // Find HandBrakeCLI path
        let handbrake = self.config.handbrake_path.clone()
            .unwrap_or_else(|| PathBuf::from("HandBrakeCLI"));
            
        // Verify HandBrakeCLI is available
        let handbrake_path = which::which(&handbrake)
            .map_err(|_| AppError::HandBrakeNotFound)?;
        
        // Execute HandBrakeCLI scan command for specific title
        let output = TokioCommand::new(&handbrake_path)
            .arg("--scan")
            .arg("--title").arg(title_number.to_string())
            .arg("-i").arg(mount_point)
            .output()
            .await
            .map_err(|e| AppError::CommandFailed(format!("Failed to execute HandBrakeCLI: {}", e)))?;
            
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Parse title details
        parse_title_details(title_number, &stderr)
    }
    
    /// Rip a DVD title
    pub async fn rip_title(&self, task: &RipTask) -> Result<()> {
        let mount_point = self.mount_point.as_ref()
            .ok_or_else(|| AppError::MountFailed("DVD not mounted".to_string()))?;
            
        info!("Ripping title {} to {}", task.title.number, task.output_path.display());
        
        // Find HandBrakeCLI path
        let handbrake = self.config.handbrake_path.clone()
            .unwrap_or_else(|| PathBuf::from("HandBrakeCLI"));
            
        // Verify HandBrakeCLI is available
        let handbrake_path = which::which(&handbrake)
            .map_err(|_| AppError::HandBrakeNotFound)?;
            
        // Create output directory if it doesn't exist
        if let Some(parent) = task.output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Check if output file already exists
        if task.output_path.exists() {
            return Err(AppError::OutputExists(task.output_path.clone()));
        }
        
        // Build the HandBrakeCLI command
        let mut command = TokioCommand::new(&handbrake_path);
        command
            .arg("--title").arg(task.title.number.to_string())
            .arg("--preset").arg("Production Standard")
            .arg("--encoder").arg(&self.config.encode_algo);
            
        // Add audio tracks
        let audio_tracks: Vec<String> = task.title.audio_tracks.keys()
            .map(|k| k.to_string())
            .collect();
            
        if !audio_tracks.is_empty() {
            command
                .arg("--audio").arg(audio_tracks.join(","))
                .arg("--aencoder").arg(vec!["faac"; audio_tracks.len()].join(","));
        }
        
        // Add chapter selection if specified
        if let Some(chapter) = task.chapter {
            command.arg("--chapters").arg(chapter.to_string());
        }
        
        // Add subtitle tracks
        let subtitle_tracks: Vec<String> = task.title.subtitle_tracks.keys()
            .map(|k| k.to_string())
            .collect();
            
        if !subtitle_tracks.is_empty() {
            command.arg("--subtitle").arg(subtitle_tracks.join(","));
        }
        
        // Add remaining arguments
        command
            .arg("--markers")
            .arg("--optimize")
            .arg("--input").arg(mount_point)
            .arg("--output").arg(&task.output_path);
            
        // Execute the command
        info!("Starting HandBrakeCLI with command: {:?}", command);
        
        let status = command
            .status()
            .await
            .map_err(|e| AppError::CommandFailed(format!("Failed to execute HandBrakeCLI: {}", e)))?;
            
        if !status.success() {
            return Err(AppError::CommandFailed(format!(
                "HandBrakeCLI exited with status: {}", 
                status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())
            )));
        }
        
        info!("Successfully ripped title {} to {}", task.title.number, task.output_path.display());
        Ok(())
    }
    
    /// Eject the DVD
    pub async fn eject(&self) -> Result<()> {
        info!("Ejecting DVD...");
        
        #[cfg(target_os = "linux")]
        {
            let status = Command::new("eject")
                .arg(&self.path)
                .status()
                .map_err(|e| AppError::CommandFailed(format!("Failed to eject DVD: {}", e)))?;
                
            if !status.success() {
                warn!("Failed to eject DVD, status code: {:?}", status.code());
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            let status = Command::new("drutil")
                .arg("eject")
                .status()
                .map_err(|e| AppError::CommandFailed(format!("Failed to eject DVD: {}", e)))?;
                
            if !status.success() {
                warn!("Failed to eject DVD, status code: {:?}", status.code());
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // On Windows, we can use PowerShell to eject
            let drive_letter = self.path.to_string_lossy().chars().next()
                .ok_or_else(|| AppError::CommandFailed("Invalid drive path".to_string()))?;
                
            let script = format!(
                "$driveEject = New-Object -comObject Shell.Application; $driveEject.Namespace(17).ParseName('{}:').InvokeVerb('Eject');",
                drive_letter
            );
            
            let status = Command::new("powershell")
                .arg("-Command")
                .arg(&script)
                .status()
                .map_err(|e| AppError::CommandFailed(format!("Failed to eject DVD: {}", e)))?;
                
            if !status.success() {
                warn!("Failed to eject DVD, status code: {:?}", status.code());
            }
        }
        
        Ok(())
    }
    
    /// Find the main feature title (usually the longest)
    pub fn find_main_feature(&self) -> Option<&Title> {
        self.titles.iter().max_by_key(|t| t.duration)
    }
    
    /// Create ripping tasks for selected titles
    pub fn create_rip_tasks(&self, output_dir: impl AsRef<Path>, selected_titles: Option<Vec<usize>>, 
                           chapter_split: bool) -> Vec<RipTask> {
        let output_dir = output_dir.as_ref();
        let mut tasks = Vec::new();
        
        // Filter titles if selection provided
        let titles = match selected_titles {
            Some(selected) => self.titles.iter()
                .filter(|t| selected.contains(&t.number))
                .collect::<Vec<_>>(),
            None => self.titles.iter().collect::<Vec<_>>(),
        };
        
        let multi_title = titles.len() > 1;
        
        for title in titles {
            if chapter_split && !title.chapters.is_empty() {
                // Create a task for each chapter
                for chapter in &title.chapters {
                    let output_path = if multi_title {
                        output_dir.join(format!("Title{:02}_Ch{:02}.mp4", title.number, chapter.number))
                    } else {
                        output_dir.join(format!("Chapter{:02}.mp4", chapter.number))
                    };
                    
                    tasks.push(RipTask {
                        title: title.clone(),
                        chapter: Some(chapter.number),
                        output_path,
                    });
                }
            } else {
                // Create a task for the entire title
                let output_path = if multi_title {
                    output_dir.join(format!("Title{:02}.mp4", title.number))
                } else {
                    output_dir.join("Movie.mp4")
                };
                
                tasks.push(RipTask {
                    title: title.clone(),
                    chapter: None,
                    output_path,
                });
            }
        }
        
        tasks
    }
}

/// Parse the title count from HandBrakeCLI output
fn parse_title_count(output: &str) -> Option<usize> {
    lazy_static::lazy_static! {
        static ref RE1: Regex = Regex::new(r"scan: DVD has (\d+) title\(s\)").unwrap();
        static ref RE2: Regex = Regex::new(r"Scanning title \d+ of (\d+)").unwrap();
    }
    
    if let Some(caps) = RE1.captures(output) {
        return caps.get(1).and_then(|m| m.as_str().parse::<usize>().ok());
    }
    
    if let Some(caps) = RE2.captures(output) {
        return caps.get(1).and_then(|m| m.as_str().parse::<usize>().ok());
    }
    
    None
}

/// Parse title details from HandBrakeCLI output
fn parse_title_details(title_number: usize, output: &str) -> Result<Title> {
    // This function is complex as it parses the HandBrakeCLI output format
    // We'll implement a simplified version for now
    
    // Duration parsing
    lazy_static::lazy_static! {
        static ref DURATION_RE: Regex = Regex::new(r"duration: (\d{2}):(\d{2}):(\d{2})").unwrap();
        static ref SIZE_RE: Regex = Regex::new(r"size: (\d+)x(\d+), pixel aspect: (\d+)/(\d+).*?(\d+\.\d+) fps").unwrap();
    }
    
    // Extract duration
    let duration = DURATION_RE.captures(output)
        .and_then(|caps| {
            let hours = caps.get(1)?.as_str().parse::<u64>().ok()?;
            let minutes = caps.get(2)?.as_str().parse::<u64>().ok()?;
            let seconds = caps.get(3)?.as_str().parse::<u64>().ok()?;
            
            Some(Duration::from_secs(hours * 3600 + minutes * 60 + seconds))
        })
        .ok_or_else(|| AppError::DvdParseFailed("Failed to parse title duration".to_string()))?;
    
    // Extract video size
    let size = SIZE_RE.captures(output)
        .and_then(|caps| {
            let width = caps.get(1)?.as_str().parse::<usize>().ok()?;
            let height = caps.get(2)?.as_str().parse::<usize>().ok()?;
            let pixel_aspect_width = caps.get(3)?.as_str().parse::<usize>().ok()?;
            let pixel_aspect_height = caps.get(4)?.as_str().parse::<usize>().ok()?;
            let fps = caps.get(5)?.as_str().parse::<f64>().ok()?;
            
            Some(VideoSize {
                width,
                height,
                pixel_aspect_width,
                pixel_aspect_height,
                fps,
            })
        })
        .ok_or_else(|| AppError::DvdParseFailed("Failed to parse video size".to_string()))?;
    
    // For now, we'll create a simple title structure
    // In a full implementation, we would parse chapters, audio tracks, and subtitles as well
    let title = Title {
        number: title_number,
        duration,
        chapters: Vec::new(), // Would be populated in full implementation
        audio_tracks: HashMap::new(), // Would be populated in full implementation
        subtitle_tracks: HashMap::new(), // Would be populated in full implementation
        size,
    };
    
    Ok(title)
}

/// Find the mount point for a device on Linux
#[cfg(target_os = "linux")]
async fn find_mount_point(device: &Path) -> Result<PathBuf> {
    let device_path = fs::canonicalize(device)
        .map_err(|_| AppError::DvdNotFound(device.to_path_buf()))?;
        
    let output = TokioCommand::new("df")
        .arg("-P")
        .output()
        .await
        .map_err(|e| AppError::CommandFailed(format!("Failed to run df command: {}", e)))?;
        
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Find the line that starts with our device path
    for line in output_str.lines().skip(1) { // Skip header
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() >= 6 && fields[0] == device_path.to_string_lossy() {
            return Ok(PathBuf::from(fields[5]));
        }
    }
    
    // If not mounted, try to mount
    let mount_point = PathBuf::from("/media/dvd");
    
    // Ensure mount point exists
    if !mount_point.exists() {
        fs::create_dir_all(&mount_point)
            .map_err(|e| AppError::MountFailed(format!("Failed to create mount point: {}", e)))?;
    }
    
    // Mount the DVD
    let status = TokioCommand::new("mount")
        .arg(device)
        .arg(&mount_point)
        .status()
        .await
        .map_err(|e| AppError::MountFailed(format!("Failed to mount DVD: {}", e)))?;
        
    if !status.success() {
        return Err(AppError::MountFailed(format!(
            "Mount command failed with status: {}", 
            status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())
        )));
    }
    
    Ok(mount_point)
}