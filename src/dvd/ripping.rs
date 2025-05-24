use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn};

use crate::config::Config;
use crate::dvd::types::{Dvd, RipTask, Title}; // Added Title import
use crate::error::{Result, AppError};

impl Dvd {
    pub async fn new(path: PathBuf, config: Config) -> Result<Self> {
        // Optionally, verify path here using detection::verify_dvd_path(&path)?
        Ok(Dvd {
            path,
            titles: Vec::new(),
            config,
        })
    }

    pub fn create_rip_tasks(
        &self,
        output_dir: PathBuf,
        selected_titles_numbers: Option<Vec<usize>>, // Numbers of titles to rip
        chapter_split: bool,
    ) -> Vec<RipTask> {
        let mut tasks = Vec::new();
        let titles_to_process: Vec<&Title> = match selected_titles_numbers {
            Some(numbers) => self
                .titles
                .iter()
                .filter(|t| numbers.contains(&t.number))
                .collect(),
            None => self.titles.iter().collect(), // Rip all titles if None
        };

        for title in titles_to_process {
            let output_filename = format!("title_{}.mkv", title.number);
            let output_path = output_dir.join(output_filename);
            tasks.push(RipTask {
                title: title.clone(),
                output_path,
                chapter_split,
            });
        }
        tasks
    }

    pub async fn rip_title(&self, task: &RipTask) -> Result<()> {
        info!(
            "Ripping title {} to {}",
            task.title.number,
            task.output_path.display()
        );

        let handbrake_path = self.config.handbrake_path.as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "HandBrakeCLI".to_string());

        let mut cmd = Command::new(handbrake_path);
        cmd.arg("-i")
            .arg(self.path.as_os_str())
            .arg("-t")
            .arg(task.title.number.to_string())
            .arg("-o")
            .arg(task.output_path.as_os_str())
            .arg("--preset") // Example: use a default preset
            .arg("Fast 1080p30") 
            .arg("-e")
            .arg(&self.config.encode_algo) // x264, x265
            .arg("-q") // Quality setting
            .arg("20"); // Example quality

        if task.chapter_split {
            // HandBrakeCLI might not directly split into files per chapter easily.
            // This usually means ripping chapters individually.
            // For simplicity, we'll ignore this for now or assume it means something else.
            // A more complex implementation would iterate task.title.chapters and rip ranges.
            warn!("Chapter splitting requested but not fully implemented in this basic rip_title function.");
        }
        
        // Add more HandBrakeCLI options based on self.config as needed
        // e.g., audio tracks, subtitles, quality, encoder options

        cmd.stdout(Stdio::piped()); // Capture stdout
        cmd.stderr(Stdio::piped()); // Capture stderr for progress or errors

        let process = cmd.spawn()
            .map_err(|e| AppError::HandbrakeError(format!("Failed to start HandBrakeCLI: {}", e)))?;
        
        // You could use process.stdout and process.stderr to parse progress here
        // For now, just wait for completion.

        let output = process.wait_with_output().await
            .map_err(|e| AppError::HandbrakeError(format!("HandBrakeCLI execution failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("HandBrakeCLI error for title {}: {}", task.title.number, stderr);
            return Err(AppError::HandbrakeError(format!(
                "HandBrakeCLI failed for title {} with status {}: {}",
                task.title.number, output.status, stderr
            )));
        }

        info!("Finished ripping title {}", task.title.number);
        Ok(())
    }

    pub async fn eject(&self) -> Result<()> {
        info!("Ejecting DVD: {}", self.path.display());
        #[cfg(target_os = "macos")]
        {
            let status = Command::new("diskutil")
                .arg("eject")
                .arg(self.path.as_os_str())
                .status()
                .await
                .map_err(|e| AppError::DvdError(format!("Failed to execute diskutil: {}", e)))?;
            if !status.success() {
                return Err(AppError::DvdError("diskutil eject failed".to_string()));
            }
        }
        #[cfg(target_os = "linux")]
        {
            let status = Command::new("eject")
                .arg(self.path.as_os_str())
                .status()
                .await
                .map_err(|e| AppError::DvdError(format!("Failed to execute eject: {}", e)))?;
            if !status.success() {
                return Err(AppError::DvdError("eject command failed".to_string()));
            }
        }
        #[cfg(target_os = "windows")]
        {
            // Ejecting on Windows is more complex, often requires PowerShell or direct API calls.
            // This is a placeholder.
            warn!("Eject not implemented for Windows in this example.");
        }
        Ok(())
    }

    pub fn find_main_feature(&self) -> Option<&Title> {
        // Find the title with the longest duration (main feature)
        self.titles
            .iter()
            .max_by_key(|title| title.duration)
    }
}
