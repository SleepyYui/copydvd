use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use crate::config::Config;
use crate::handbrake_manager::HandBrakeManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Title {
    pub number: usize,
    pub duration: Duration,
    pub size: DvdSize,
    pub chapters: Vec<Chapter>,
    pub description: Option<String>, // e.g., "Main Movie", "Bonus Feature"
    // Add other relevant fields like audio tracks, subtitles if needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub number: usize,
    pub duration: Duration,
    // Add other relevant fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DvdSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct RipTask {
    pub title: Title,
    pub output_path: PathBuf,
    pub chapter_split: bool,
    // Add other relevant fields like specific chapters to rip, audio/subtitle selection
}

#[derive(Debug)]
pub struct Dvd {
    pub path: PathBuf,
    pub titles: Vec<Title>,
    pub config: Config, // DVD-specific operations might need config
    pub handbrake_manager: HandBrakeManager,
    // Add other fields like volume ID, disk label, etc.
}

// Implementation for Dvd will be in other dvd submodule files (scanning.rs, ripping.rs)
// Or potentially some methods directly here if they are simple type-related helpers.
impl Dvd {
    // Constructor and basic methods might live in a different submodule or here
    // For now, assume they are in other files like ripping.rs or scanning.rs
    // Example:
    // pub async fn new(path: PathBuf, config: Config) -> crate::error::Result<Self> { /* ... */ }
    // pub async fn scan_titles(&mut self) -> crate::error::Result<()> { /* ... */ }
    // pub fn find_main_feature(&self) -> Option<&Title> { /* ... */ }
    // pub fn create_rip_tasks(...) -> Vec<RipTask> { /* ... */ }
    // pub async fn rip_title(...) -> crate::error::Result<()> { /* ... */ }
    // pub async fn eject(&self) -> crate::error::Result<()> { /* ... */ }
}

