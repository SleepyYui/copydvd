use crate::config::Config;
use crate::dvd::types::{Dvd, RipTask};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub dvd: Option<Arc<Mutex<Dvd>>>,
    pub rip_tasks: Vec<RipTask>,
    pub status: AppStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppStatus {
    Idle,
    Ready, // Added Ready
    Scanning,
    ScanComplete(usize), // Added ScanComplete with title count
    Ripping { completed: usize, total: usize },
    RipComplete,                 // Added RipComplete
    Uploading { progress: f32 }, // Added Uploading with progress
    UploadComplete,              // Added UploadComplete
    Completed,
    Error(String),
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            dvd: None,
            rip_tasks: Vec::new(),
            status: AppStatus::Idle,
        }
    }
}
