use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing::{info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::dvd::{Dvd, RipTask};
use crate::error::{AppError, Result};
use crate::cli;
use crate::gui;
use crate::upload;

/// Application state
#[derive(Debug)]
pub struct AppState {
    pub config: Config,
    pub dvd: Option<Arc<Mutex<Dvd>>>,
    pub rip_tasks: Vec<RipTask>,
    pub status: AppStatus,
}

/// Application status
#[derive(Debug, Clone, PartialEq)]
pub enum AppStatus {
    Idle,
    Scanning,
    Ripping { completed: usize, total: usize },
    Completed,
    Error(String),
}

impl AppState {
    /// Create a new application state
    pub fn new(config: Config) -> Self {
        Self {
            config,
            dvd: None,
            rip_tasks: Vec::new(),
            status: AppStatus::Idle,
        }
    }
}

/// Run the application
pub async fn run() -> Result<()> {
    // Load configuration
    let config = Config::load()?;
    
    // Create application state
    let app_state = Arc::new(Mutex::new(AppState::new(config.clone())));
    
    // Check if GUI mode or CLI mode
    let args = cli::parse_args();
    
    if args.gui {
        // Run in GUI mode
        info!("Starting in GUI mode");
        gui::run(app_state).await?;
    } else {
        // Run in CLI mode
        info!("Starting in CLI mode");
        run_cli(app_state, args).await?;
    }
    
    Ok(())
}

/// Run the application in CLI mode
async fn run_cli(app_state: Arc<Mutex<AppState>>, args: cli::Args) -> Result<()> {
    // Initialize DVD
    let mut state = app_state.lock().await;
    state.status = AppStatus::Scanning;
    
    let dvd_path = args.input.unwrap_or_else(|| {
        info!("No DVD path specified, attempting to auto-detect");
        auto_detect_dvd()
            .expect("Failed to auto-detect DVD drive")
    });
    
    info!("Using DVD path: {}", dvd_path.display());
    
    // Create and scan the DVD
    let dvd = Dvd::new(dvd_path, state.config.clone()).await?;
    let dvd = Arc::new(Mutex::new(dvd));
    state.dvd = Some(Arc::clone(&dvd));
    drop(state);
    
    // Scan titles
    let mut dvd_lock = dvd.lock().await;
    dvd_lock.scan_titles().await?;
    
    // Get output directory
    let output_dir = args.output.unwrap_or_else(|| {
        let config = &dvd_lock.config;
        config.output_dir.clone()
    });
    
    // Prepare ripping tasks
    let selected_titles = if args.main_feature {
        // If main feature flag is set, find the longest title
        info!("Selecting main feature only");
        dvd_lock.find_main_feature()
            .map(|title| vec![title.number])
    } else if let Some(titles) = args.titles {
        // If specific titles are requested
        info!("Selecting titles: {:?}", titles);
        Some(titles)
    } else {
        // Otherwise, select all titles
        info!("Selecting all titles");
        None
    };
    
    let chapter_split = args.chapter_split.unwrap_or(dvd_lock.config.chapter_split);
    let tasks = dvd_lock.create_rip_tasks(output_dir, selected_titles, chapter_split);
    let task_count = tasks.len();
    
    info!("Created {} ripping tasks", task_count);
    
    // Update app state
    let mut state = app_state.lock().await;
    state.rip_tasks = tasks.clone();
    state.status = AppStatus::Ripping { completed: 0, total: task_count };
    drop(state);
    drop(dvd_lock);
    
    // Execute ripping tasks in parallel
    let (tx, mut rx) = mpsc::channel(task_count);
    let thread_count = args.threads.unwrap_or_else(|| {
        let config = &app_state.lock().await.config;
        config.thread_count
    });
    
    info!("Using {} threads for ripping", thread_count);
    
    // Create a task for each ripping job
    for (i, task) in tasks.iter().enumerate() {
        let dvd_clone = Arc::clone(&dvd);
        let task_clone = task.clone();
        let tx_clone = tx.clone();
        
        tokio::spawn(async move {
            let result = dvd_clone.lock().await.rip_title(&task_clone).await;
            let _ = tx_clone.send((i, result)).await;
        });
        
        // Limit concurrency to thread_count
        if (i + 1) % thread_count == 0 {
            if let Some((task_index, result)) = rx.recv().await {
                handle_task_result(Arc::clone(&app_state), task_index, result).await;
            }
        }
    }
    
    // Drop the sender to close the channel once all tasks are spawned
    drop(tx);
    
    // Wait for remaining tasks
    while let Some((task_index, result)) = rx.recv().await {
        handle_task_result(Arc::clone(&app_state), task_index, result).await;
    }
    
    // Eject disc if configured
    let dvd_lock = dvd.lock().await;
    if dvd_lock.config.eject_after_rip {
        info!("Ejecting DVD");
        if let Err(e) = dvd_lock.eject().await {
            warn!("Failed to eject DVD: {}", e);
        }
    }
    
    // Upload files if requested and server config exists
    if args.upload {
        if let Some(server_config) = &dvd_lock.config.server {
            info!("Uploading ripped files to server");
            
            // Get a descriptive name for the DVD if possible
            let movie_name = dvd_lock.find_main_feature()
                .map(|title| format!("Movie_Title{}", title.number))
                .unwrap_or_else(|| "DVD_Rip".to_string());
            
            // Create upload tasks for each ripped file
            for task in &state.rip_tasks {
                if task.output_path.exists() {
                    let upload_task = upload::create_upload_task(
                        &task.output_path,
                        server_config,
                        &movie_name
                    );
                    
                    info!("Uploading {} to {}", 
                          task.output_path.display(), 
                          upload_task.remote_path);
                    
                    match upload::upload_file(&upload_task).await {
                        Ok(_) => info!("Upload complete for {}", task.output_path.display()),
                        Err(e) => warn!("Upload failed for {}: {}", task.output_path.display(), e),
                    }
                }
            }
        } else {
            warn!("Upload requested but no server configuration found");
        }
    }
    
    // Update status to completed
    let mut state = app_state.lock().await;
    state.status = AppStatus::Completed;
    
    info!("DVD ripping completed successfully");
    Ok(())
}

/// Handle the result of a ripping task
async fn handle_task_result(app_state: Arc<Mutex<AppState>>, task_index: usize, result: Result<()>) {
    let mut state = app_state.lock().await;
    
    if let AppStatus::Ripping { completed, total } = state.status {
        let new_completed = completed + 1;
        
        if let Err(e) = result {
            warn!("Task {} failed: {}", task_index, e);
            // Continue with other tasks even if one fails
        }
        
        state.status = if new_completed >= total {
            AppStatus::Completed
        } else {
            AppStatus::Ripping { completed: new_completed, total }
        };
    }
}

/// Auto-detect DVD drive path
fn auto_detect_dvd() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        // On Linux, try /dev/sr0 and /dev/cdrom
        for path in &["/dev/sr0", "/dev/cdrom"] {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // On macOS, use /dev/disk*
        for entry in std::fs::read_dir("/Volumes")? {
            let entry = entry?;
            let path = entry.path();
            
            // Check if it looks like a DVD
            if path.is_dir() && path.join("VIDEO_TS").exists() {
                return Ok(path);
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        // On Windows, check drive letters
        for drive in b'D'..=b'Z' {
            let drive_path = PathBuf::from(format!("{}:\\", drive as char));
            
            if drive_path.exists() && drive_path.join("VIDEO_TS").exists() {
                return Ok(drive_path);
            }
        }
    }
    
    Err(AppError::DvdNotFound(PathBuf::from("DVD")))
}