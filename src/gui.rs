use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use iced::{
    Application, Command, Element, Settings, Theme,
    widget::{button, column, container, progress_bar, row, text, text_input, scrollable, Column, checkbox},
    executor, Length, Subscription, subscription, Color,
    window, alignment,
};
use iced::window::icon;
use tracing::{info, warn};

use crate::app::{AppState, AppStatus};
use crate::config::ServerConfig;
use crate::dvd::types::{Dvd, Title};
use crate::error::{AppError, Result};
use crate::utils;
use crate::upload;

/// Run the GUI application
pub async fn run(app_state: Arc<Mutex<AppState>>) -> Result<()> {
    // Load a system font as fallback to avoid glyph rasterizer issues
    let font_bytes = include_bytes!("../resources/NotoSans-Regular.ttf");
    
    let gui_settings = Settings {
        window: window::Settings {
            size: (800, 600),
            position: window::Position::Centered,
            min_size: Some((640, 480)),
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: None,
            visible: true,
            #[cfg(target_os = "macos")]
            platform_specific: window::PlatformSpecific {
                title_hidden: false,
                fullsize_content_view: false,
                titlebar_transparent: false,
            },
            #[cfg(not(target_os = "macos"))]
            platform_specific: window::PlatformSpecific::default(),
        },
        flags: app_state,
        id: None,
        default_font: Some(font_bytes),
        default_text_size: 16.0,
        text_multithreading: false,
        antialiasing: false,      // Disable antialiasing to simplify rendering
        exit_on_close_request: true,
        try_opengles_first: true, // Try OpenGL ES instead of Metal on macOS
    };

    DvdRipperGui::run(gui_settings)
        .map_err(|e| AppError::GuiError(format!("GUI error: {}", e)))
}

/// DVD Ripper GUI Application
struct DvdRipperGui {
    app_state: Arc<Mutex<AppState>>,
    input_path: String,
    output_path: String,
    selected_titles: Vec<usize>,
    chapter_split: bool,
    main_feature_only: bool,
    upload_to_server: bool,
    is_scanning: bool,
    is_ripping: bool,
    is_uploading: bool,
    scan_complete: bool,
    rip_progress: f32,
    upload_progress: f32,
    status_message: String,
    error_message: Option<String>,
    titles: Vec<Title>,
    
    // Configuration options
    handbrake_cli_path_input: String, // For text input
    encode_algo: String,
    thread_count_input: String, // For text input
    eject_after_rip: bool,
    
    // Server config
    server_host: String,
    server_username: String,
    server_password: String,
    server_path: String,
    
    // UI state
    show_config_panel: bool,
    show_server_config: bool,
}

#[derive(Debug, Clone)]
enum Message {
    InputPathChanged(String),
    OutputPathChanged(String),
    BrowseInput,
    BrowseOutput,
    ScanDvd,
    TitleSelected(usize, bool), // Title number, is_selected
    ToggleMainFeature(bool),
    ToggleChapterSplit(bool),
    ToggleUpload(bool),
    StartRipping,
    CancelRipping,
    StatusUpdate(String),      // Renamed from UpdateStatus for consistency
    ScanComplete(Vec<Title>),
    RipProgress(f32),          // Renamed from UpdateProgress
    UploadProgress(f32),
    RipComplete,
    UploadComplete,
    Error(String),             // Renamed from ShowError
    DismissError,
    Tick,
    None, // Added None variant

    // Configuration messages
    ToggleConfigPanel,
    ToggleServerConfig,
    HandbrakePathChanged(String), // New message
    EncodeAlgoChanged(String),
    ThreadCountChanged(String),
    ToggleEjectAfterRip(bool),
    
    // Server configuration
    ServerHostChanged(String),
    ServerUsernameChanged(String),
    ServerPasswordChanged(String),
    ServerPathChanged(String),
    SaveConfig,
}

impl Application for DvdRipperGui {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Arc<Mutex<AppState>>;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let app_state = flags;
        
        // Load default config values
        let config = {
            let state_guard = app_state.try_lock().expect("Failed to lock app state");
            state_guard.config.clone()
        };
        
        let gui = Self {
            app_state,
            input_path: String::new(),
            output_path: config.output_dir.to_string_lossy().to_string(),
            selected_titles: Vec::new(),
            chapter_split: config.chapter_split,
            main_feature_only: false,
            upload_to_server: config.server.is_some(),
            is_scanning: false,
            is_ripping: false,
            is_uploading: false,
            scan_complete: false,
            rip_progress: 0.0,
            upload_progress: 0.0,
            status_message: "Ready to scan DVD".to_string(),
            error_message: None,
            titles: Vec::new(),
            
            // Configuration options
            handbrake_cli_path_input: config.handbrake_path.as_ref().map_or(String::new(), |p| p.to_string_lossy().to_string()),
            encode_algo: config.encode_algo.clone(),
            thread_count_input: config.thread_count.to_string(),
            eject_after_rip: config.eject_after_rip,
            
            // Server config
            server_host: config.server.as_ref().map_or(String::new(), |s| s.host.clone()),
            server_username: config.server.as_ref().map_or(String::new(), |s| s.username.clone()),
            server_password: config.server.as_ref().map_or(String::new(), |s| s.password.clone().unwrap_or_default()),
            server_path: config.server.as_ref().map_or(String::new(), |s| s.path.clone()),
            
            // UI state
            show_config_panel: false,
            show_server_config: config.server.is_some(),
        };
        
        // Return the GUI with initial command
        (gui, Command::none())
    }

    fn title(&self) -> String {
        "DVD Ripper".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::InputPathChanged(path) => {
                self.input_path = path;
                Command::none()
            }
            
            Message::OutputPathChanged(path) => {
                self.output_path = path;
                Command::none()
            }
            
            Message::BrowseInput => {
                Command::perform(browse_for_folder("Select DVD Drive or Directory"), |result| {
                    match result {
                        Ok(path) => {
                            if let Some(path_str) = path.to_str() {
                                Message::InputPathChanged(path_str.to_string())
                            } else {
                                Message::Error("Invalid path selected".to_string())
                            }
                        }
                        Err(e) => Message::Error(format!("Failed to select folder: {}", e)),
                    }
                })
            }
            
            Message::BrowseOutput => {
                Command::perform(browse_for_folder("Select Output Directory"), |result| {
                    match result {
                        Ok(path) => {
                            if let Some(path_str) = path.to_str() {
                                Message::OutputPathChanged(path_str.to_string())
                            } else {
                                Message::Error("Invalid path selected".to_string())
                            }
                        }
                        Err(e) => Message::Error(format!("Failed to select folder: {}", e)),
                    }
                })
            }
            
            Message::ScanDvd => {
                if self.input_path.is_empty() {
                    return Command::perform(
                        async { Err("Please select a DVD path first".to_string()) },
                        |result: std::result::Result<(), String>| match result {
                            Ok(_) => unreachable!(),
                            Err(e) => Message::Error(e),
                        },
                    );
                }
                
                self.is_scanning = true;
                self.scan_complete = false;
                self.status_message = "Scanning DVD...".to_string();
                
                let input_path = PathBuf::from(&self.input_path);
                let app_state: Arc<Mutex<AppState>> = Arc::clone(&self.app_state);
                
                Command::perform(
                    async move {
                        // Update app state to scanning
                        let mut state = app_state.lock().await;
                        state.status = AppStatus::Scanning;
                        
                        // Create and configure DVD
                        let config = state.config.clone();
                        drop(state);
                        
                        let mut dvd = Dvd::new(input_path, config).await?;
                        
                        // Scan titles
                        dvd.scan_titles().await?;
                        
                        // Store the titles for returning before we wrap the DVD
                        let titles = dvd.titles.clone();
                        
                        // Update app state with DVD
                        let mut state = app_state.lock().await;
                        let dvd = Arc::new(Mutex::new(dvd));
                        state.dvd = Some(dvd);
                        
                        // Return titles for display
                        Ok(titles)
                    },
                    |result: std::result::Result<Vec<Title>, AppError>| match result {
                        Ok(titles) => Message::ScanComplete(titles),
                        Err(e) => Message::Error(e.to_string()),
                    },
                )
            }
            
            Message::TitleSelected(title_num, selected) => {
                if selected {
                    if !self.selected_titles.contains(&title_num) {
                        self.selected_titles.push(title_num);
                    }
                } else {
                    self.selected_titles.retain(|&t| t != title_num);
                }
                
                Command::none()
            }
            
            Message::ToggleMainFeature(enabled) => {
                self.main_feature_only = enabled;
                
                if enabled {
                    // Clear other selections when main feature is selected
                    self.selected_titles.clear();
                }
                
                Command::none()
            }
            
            Message::ToggleChapterSplit(enabled) => {
                self.chapter_split = enabled;
                Command::none()
            }

            Message::ToggleUpload(enabled) => {
                self.upload_to_server = enabled;
                Command::none()
            }
            
            Message::StartRipping => {
                if self.output_path.is_empty() {
                    return Command::perform(
                        async { Err("Please select an output directory first".to_string()) },
                        |result: std::result::Result<(), String>| match result {
                            Ok(_) => unreachable!(),
                            Err(e) => Message::Error(e),
                        },
                    );
                }
                
                if !self.main_feature_only && self.selected_titles.is_empty() {
                    return Command::perform(
                        async { Err("Please select at least one title to rip".to_string()) },
                        |result: std::result::Result<(), String>| match result {
                            Ok(_) => unreachable!(),
                            Err(e) => Message::Error(e),
                        },
                    );
                }
                
                self.is_ripping = true;
                self.status_message = "Starting ripping process...".to_string();
                self.rip_progress = 0.0;
                
                let app_state: Arc<Mutex<AppState>> = Arc::clone(&self.app_state);
                let output_path = PathBuf::from(&self.output_path);
                let selected_titles = if self.main_feature_only {
                    None // Will select main feature in the task
                } else {
                    Some(self.selected_titles.clone())
                };
                let chapter_split = self.chapter_split;
                let main_feature_only = self.main_feature_only;
                let upload_to_server = self.upload_to_server;
                
                Command::perform(
                    async move {
                        let mut state_guard = app_state.lock().await; // Lock app_state first
                        
                        let dvd_arc: Arc<Mutex<Dvd>> = state_guard.dvd.as_ref()
                            .ok_or_else(|| AppError::UserError("DVD not scanned".to_string()))?
                            .clone();
                        
                        let selected_titles_vec = if main_feature_only {
                            let dvd_lock_guard = dvd_arc.lock().await; // Lock dvd_arc for find_main_feature
                            dvd_lock_guard.find_main_feature()
                                .map(|title| vec![title.number])
                                .ok_or_else(|| AppError::UserError("No main feature found".to_string()))?
                        } else {
                            selected_titles.ok_or_else(|| AppError::UserError("No titles selected".to_string()))?
                        };
                        
                        let tasks = {
                            let dvd_lock_guard = dvd_arc.lock().await; // Lock dvd_arc for create_rip_tasks
                            dvd_lock_guard.create_rip_tasks(output_path, Some(selected_titles_vec), chapter_split)
                        };
                        let task_count = tasks.len();
                        
                        state_guard.rip_tasks = tasks.clone();
                        state_guard.status = AppStatus::Ripping { completed: 0, total: task_count };
                        
                        // Drop the guard before spawning the tokio task to release the lock
                        drop(state_guard);

                        // Clone Arcs for the spawned task
                        let app_state_clone_for_spawn: Arc<Mutex<AppState>> = Arc::clone(&app_state);
                        let dvd_arc_clone_for_spawn: Arc<Mutex<Dvd>> = Arc::clone(&dvd_arc);
                        let tasks_clone_for_spawn = tasks.clone();

                        tokio::spawn(async move {
                            let mut completed = 0;
                            
                            for task in tasks_clone_for_spawn {
                                let result = dvd_arc_clone_for_spawn.lock().await.rip_title(&task).await;
                                
                                if let Err(e) = result {
                                    warn!("Failed to rip title {}: {}", task.title.number, e);
                                }
                                
                                completed += 1;
                                
                                let mut state_guard_spawn = app_state_clone_for_spawn.lock().await;
                                state_guard_spawn.status = AppStatus::Ripping {
                                    completed,
                                    total: task_count,
                                };
                                // Drop guard inside loop iteration if possible, or ensure it's dropped before next .await
                                drop(state_guard_spawn);
                            }
                            
                            let dvd_lock_guard_eject = dvd_arc_clone_for_spawn.lock().await;
                            if dvd_lock_guard_eject.config.eject_after_rip {
                                if let Err(e) = dvd_lock_guard_eject.eject().await {
                                    warn!("Failed to eject DVD: {}", e);
                                }
                            }
                            drop(dvd_lock_guard_eject); // Explicitly drop before potential upload
                            
                            if upload_to_server {
                                let dvd_lock_guard_upload = dvd_arc_clone_for_spawn.lock().await;
                                if let Some(server_config) = &dvd_lock_guard_upload.config.server {
                                    let movie_name = dvd_lock_guard_upload.find_main_feature()
                                        .map(|title| format!("Movie_Title{}", title.number))
                                        .unwrap_or_else(|| "DVD_Rip".to_string());
                                    
                                    let server_config_clone = server_config.clone(); // Clone server_config to move into upload loop
                                    drop(dvd_lock_guard_upload); // Drop lock before iterating tasks for upload

                                    for (i, task) in tasks.iter().enumerate() {
                                        if task.output_path.exists() {
                                            let upload_task = upload::create_upload_task(
                                                &task.output_path,
                                                &server_config_clone, // Use cloned server_config
                                                &movie_name
                                            );
                                            
                                            info!("Uploading {} to {}", 
                                                  task.output_path.display(), 
                                                  upload_task.remote_path);
                                            
                                            let mut state_guard_upload_progress = app_state_clone_for_spawn.lock().await;
                                            state_guard_upload_progress.status = AppStatus::Ripping { 
                                                completed: task_count + i, 
                                                total: task_count * 2 
                                            };
                                            drop(state_guard_upload_progress);
                                            
                                            if let Err(e) = upload::upload_file(&upload_task).await {
                                                warn!("Upload failed for {}: {}", 
                                                      task.output_path.display(), e);
                                            }
                                        }
                                    }
                                } else {
                                    warn!("Upload requested but no server configuration found");
                                    drop(dvd_lock_guard_upload); // Ensure lock is dropped if server_config is None
                                }
                            }
                            
                            let mut state_guard_final = app_state_clone_for_spawn.lock().await;
                            state_guard_final.status = AppStatus::Completed;
                        });
                        
                        Ok(())
                    },
                    |result: std::result::Result<(), AppError>| match result {
                        Ok(_) => Message::StatusUpdate("Ripping in progress...".to_string()),
                        Err(e) => {
                            warn!("Failed to start ripping: {}", e.to_string());
                            Message::Error(e.to_string())
                        },
                    },
                )
            }
            
            Message::CancelRipping => {
                self.is_ripping = false;
                self.status_message = "Ripping cancelled".to_string();
                
                // Note: We can't actually cancel the ripping process directly
                // in the current design. We would need to add cancellation
                // tokens to make this work properly.
                
                Command::none()
            }
            
            Message::StatusUpdate(message) => {
                self.status_message = message;
                Command::none()
            }
            
            Message::ScanComplete(titles) => {
                self.is_scanning = false;
                self.scan_complete = true;
                self.titles = titles;
                self.status_message = format!("Found {} titles", self.titles.len());
                self.selected_titles.clear();
                
                Command::none()
            }
            
            Message::RipProgress(progress) => {
                self.rip_progress = progress;
                Command::none()
            }
            
            Message::UploadProgress(progress) => {
                self.upload_progress = progress;
                self.status_message = format!("Uploading: {}%", (progress * 100.0) as u32);
                Command::none()
            }
            
            Message::RipComplete => {
                self.is_ripping = false;
                self.status_message = "Ripping completed".to_string();
                Command::none()
            }
            
            // Only one Message::Error case needed, combining them
            Message::Error(message) => {
                self.error_message = Some(message.clone());
                self.is_scanning = false;
                self.is_ripping = false;
                self.is_uploading = false;
                self.status_message = format!("Error: {}", message);
                warn!("GUI error: {}", message);
                Command::none()
            }
            
            Message::DismissError => {
                self.error_message = None;
                Command::none()
            }
            
            Message::ToggleConfigPanel => {
                self.show_config_panel = !self.show_config_panel;
                Command::none()
            }
            
            Message::ToggleServerConfig => {
                self.show_server_config = !self.show_server_config;
                Command::none()
            }

            Message::HandbrakePathChanged(value) => {
                self.handbrake_cli_path_input = value;
                Command::none()
            }
            
            Message::EncodeAlgoChanged(value) => {
                self.encode_algo = value;
                Command::none()
            }
            
            Message::ThreadCountChanged(value) => {
                self.thread_count_input = value;
                Command::none()
            }
            
            Message::ToggleEjectAfterRip(value) => {
                self.eject_after_rip = value;
                Command::none()
            }
            
            Message::ServerHostChanged(value) => {
                self.server_host = value;
                Command::none()
            }
            
            Message::ServerUsernameChanged(value) => {
                self.server_username = value;
                Command::none()
            }
            
            Message::ServerPasswordChanged(value) => {
                self.server_password = value;
                Command::none()
            }
            
            Message::ServerPathChanged(value) => {
                self.server_path = value;
                Command::none()
            }
            
            Message::SaveConfig => {
                let app_state_clone: Arc<Mutex<AppState>> = Arc::clone(&self.app_state);
                let mut new_config = {
                    // Clone the existing config to modify it
                    let state_guard = self.app_state.try_lock().expect("Failed to lock app_state for config save");
                    state_guard.config.clone()
                };

                new_config.handbrake_path = if self.handbrake_cli_path_input.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(&self.handbrake_cli_path_input))
                };
                new_config.encode_algo = self.encode_algo.clone();

                match self.thread_count_input.parse::<usize>() {
                    Ok(count) if count > 0 => {
                        new_config.thread_count = count;
                    }
                    Ok(_) | Err(_) if self.thread_count_input.is_empty() => {
                         // If empty, use default from num_cpus or keep existing
                         // For simplicity, let's keep the existing loaded value if input is empty or invalid
                         // Or, explicitly set to default:
                         // new_config.thread_count = num_cpus::get().max(1);
                         // For now, we'll just report an error if it's not a valid positive number
                         self.error_message = Some(format!("Invalid thread count '{}'. Please enter a positive number.", self.thread_count_input));
                         return Command::none(); // Don't save if invalid
                    }
                    _ => {
                        self.error_message = Some(format!("Invalid thread count '{}'. Please enter a positive number.", self.thread_count_input));
                        return Command::none(); // Don't save if invalid
                    }
                }

                new_config.eject_after_rip = self.eject_after_rip;
                new_config.output_dir = PathBuf::from(&self.output_path); // Save output path too

                if self.upload_to_server {
                    if self.server_host.is_empty() || self.server_username.is_empty() || self.server_path.is_empty() {
                        self.error_message = Some("Server host, username, and path are required for upload.".to_string());
                        // Optionally, don't save server config or disable upload_to_server
                        new_config.server = None;
                    } else {
                        new_config.server = Some(ServerConfig {
                            host: self.server_host.clone(),
                            username: self.server_username.clone(),
                            password: if self.server_password.is_empty() { None } else { Some(self.server_password.clone()) },
                            path: self.server_path.clone(),
                        });
                    }
                } else {
                    new_config.server = None;
                }
                
                let config_to_save = new_config.clone(); // Clone for the async block
                Command::perform(
                    async move {
                        let mut state = app_state_clone.lock().await;
                        state.config = config_to_save;
                        state.config.save().map_err(|e| e.to_string())
                    },
                    |result: std::result::Result<(), String>| match result {
                        Ok(_) => Message::StatusUpdate("Configuration saved successfully".to_string()),
                        Err(e) => Message::Error(format!("Failed to save configuration: {}", e)),
                    }
                )
            }
            
            Message::UploadComplete => {
                self.is_uploading = false;
                self.status_message = "Upload completed".to_string();
                Command::none()
            }

            Message::None => {
                // Do nothing for the None variant
                Command::none()
            }
            
            Message::Tick => {
                // Poll application state for updates
                let app_state_clone: Arc<Mutex<AppState>> = Arc::clone(&self.app_state);
                
                Command::perform(
                    async move {
                        let state = app_state_clone.lock().await;
                        match state.status {
                            AppStatus::Scanning => Some(Message::StatusUpdate("Scanning...".to_string())),
                            AppStatus::Ripping { completed, total } => {
                                if total > 0 {
                                    Some(Message::RipProgress(completed as f32 / total as f32)) // Use RipProgress
                                } else {
                                    Some(Message::StatusUpdate("Preparing to rip...".to_string()))
                                }
                            }
                            AppStatus::Uploading { progress } => Some(Message::UploadProgress(progress)),
                            AppStatus::Completed => Some(Message::StatusUpdate("Process completed".to_string())),
                            AppStatus::Error(ref e) => Some(Message::Error(e.clone())),
                            _ => None, // No specific message for other states like Idle or Ready
                        }
                    }, 
                    |msg_option| msg_option.unwrap_or(Message::None) // Handle Option<Message>
                )
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let app_state_clone: Arc<Mutex<AppState>> = Arc::clone(&self.app_state);
        if self.is_ripping || self.is_scanning || self.is_uploading {
            subscription::unfold(
                "app_status_poll",
                AppStatusPollState::Initial,
                move |state| {
                    let app_state_for_async_block: Arc<Mutex<AppState>> = Arc::clone(&app_state_clone);
                    async move {
                        match state {
                            AppStatusPollState::Initial => {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                                (Message::Tick, AppStatusPollState::Polling)
                            }
                            AppStatusPollState::Polling => {
                                let app_state_guard = app_state_for_async_block.lock().await;
                                let current_status = app_state_guard.status.clone();
                                let next_message = match &current_status {
                                    AppStatus::Ripping { completed, total } => {
                                        if *total > 0 {
                                            Message::RipProgress(*completed as f32 / *total as f32)
                                        } else {
                                            Message::StatusUpdate("Preparing to rip...".to_string())
                                        }
                                    }
                                    AppStatus::Uploading { progress } => Message::UploadProgress(*progress),
                                    AppStatus::Completed => Message::StatusUpdate("All tasks completed.".to_string()),
                                    AppStatus::Error(e) => Message::Error(e.clone()),
                                    _ => Message::None,
                                };
                                
                                // If the process is truly finished, transition to Finished state
                                if matches!(&current_status, AppStatus::Completed | AppStatus::Idle | AppStatus::Error(_)) {
                                    if !matches!(&current_status, AppStatus::Error(_)) {
                                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                        return (next_message, AppStatusPollState::Finished);
                                    }
                                }
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                (next_message, AppStatusPollState::Polling)
                            }
                            AppStatusPollState::Finished => {
                                // Continue returning the last message without further polling
                                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                (Message::None, AppStatusPollState::Finished)
                            }
                        }
                    }
                }
            )
        } else {
            Subscription::none()
        }
    }

    fn view(&self) -> Element<Message> {
        let title_text = text("DVD Ripper") // Renamed to avoid conflict
            .size(30)
            .width(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Center);
            
        // Input and output path selection
        let input_row = row![
            text("DVD Path:").width(Length::Fixed(100.0)),
            text_input("Enter DVD path", &self.input_path)
                .on_input(Message::InputPathChanged)
                .padding(5)
                .width(Length::Fill),
            button("Browse").on_press(Message::BrowseInput).padding(5),
        ]
        .spacing(10)
        .align_items(iced::alignment::Alignment::Center);
        
        let output_row = row![
            text("Output:").width(Length::Fixed(100.0)),
            text_input("Enter output directory", &self.output_path)
                .on_input(Message::OutputPathChanged)
                .padding(5)
                .width(Length::Fill),
            button("Browse").on_press(Message::BrowseOutput).padding(5),
        ]
        .spacing(10)
        .align_items(iced::alignment::Alignment::Center);
        
        // Scan and controls
        let scan_button = if self.is_scanning {
            button("Scanning...")
                .padding(10)
                .width(Length::Fixed(120.0))
        } else {
            button("Scan DVD")
                .on_press(Message::ScanDvd)
                .padding(10)
                .width(Length::Fixed(120.0))
        };
        
        let rip_button = if self.is_ripping {
            button("Ripping...")
                .padding(10)
                .width(Length::Fixed(120.0))
        } else if self.scan_complete {
            button("Start Ripping")
                .on_press(Message::StartRipping)
                .padding(10)
                .width(Length::Fixed(120.0))
        } else {
            button("Start Ripping")
                .padding(10)
                .width(Length::Fixed(120.0))
        };
        
        let cancel_button = if self.is_ripping {
            button("Cancel")
                .on_press(Message::CancelRipping)
                .padding(10)
                .width(Length::Fixed(120.0))
        } else {
            button("Cancel")
                .padding(10)
                .width(Length::Fixed(120.0))
        };
        
        // Configuration button
        let config_button = button(
            if self.show_config_panel { "Hide Config" } else { "Show Config" }
        )
        .on_press(Message::ToggleConfigPanel)
        .padding(10)
        .width(Length::Fixed(120.0));
        
        let action_row = row![
            scan_button,
            rip_button,
            cancel_button,
            config_button,
        ]
        .spacing(10)
        .align_items(iced::alignment::Alignment::Center);
        
        // Options
        let main_feature_checkbox = checkbox(
            "Main Feature Only",
            self.main_feature_only,
            Message::ToggleMainFeature
        );
        
        let chapter_split_checkbox = checkbox(
            "Split Chapters",
            self.chapter_split,
            Message::ToggleChapterSplit
        );
        
        let upload_checkbox = checkbox(
            "Upload to Server",
            self.upload_to_server,
            Message::ToggleUpload
        );
        
        let options_row = row![
            main_feature_checkbox,
            chapter_split_checkbox,
            upload_checkbox,
        ]
        .spacing(20)
        .align_items(iced::alignment::Alignment::Center);
        
        // Configuration panel (only shown when toggled)
        let config_panel: Element<Message> = if self.show_config_panel {
            let handbrake_path_row: iced::widget::Row<'_, Message, iced::Renderer> = row![
                text("HandBrakeCLI Path:").width(Length::Fixed(150.0)),
                text_input("Enter HandBrakeCLI path or name", &self.handbrake_cli_path_input)
                    .on_input(Message::HandbrakePathChanged)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_items(iced::alignment::Alignment::Center);

            let encoder_row = row![
                text("Encode Algorithm:").width(Length::Fixed(150.0)),
                text_input("e.g., x264, x265", &self.encode_algo)
                    .on_input(Message::EncodeAlgoChanged)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_items(iced::alignment::Alignment::Center);
            
            let threads_row = row![
                text("Ripping Threads:").width(Length::Fixed(150.0)),
                text_input("Number of threads", &self.thread_count_input)
                    .on_input(Message::ThreadCountChanged)
                    .width(Length::Fill),
            ]
            .spacing(10)
            .align_items(iced::alignment::Alignment::Center);
            
            let eject_checkbox = checkbox(
                "Eject DVD after ripping",
                self.eject_after_rip,
                Message::ToggleEjectAfterRip
            );
            
            let server_toggle = checkbox(
                "Configure Server for Upload",
                self.show_server_config,
                |_is_checked| Message::ToggleServerConfig // Fixed: added underscore to indicate unused variable
            );
            
            // Server configuration panel (only shown when toggled)
            let server_panel = if self.show_server_config {
                let host_row = row![
                    text("Server Host:").width(Length::Fixed(120.0)),
                    text_input("Enter server host", &self.server_host)
                        .on_input(Message::ServerHostChanged)
                        .width(Length::Fill),
                ].spacing(10).align_items(iced::alignment::Alignment::Center);

                let user_row = row![
                    text("Username:").width(Length::Fixed(120.0)),
                    text_input("Enter username", &self.server_username)
                        .on_input(Message::ServerUsernameChanged)
                        .width(Length::Fill),
                ].spacing(10).align_items(iced::alignment::Alignment::Center);

                let pass_row = row![
                    text("Password:").width(Length::Fixed(120.0)),
                    text_input("Enter password (optional)", &self.server_password)
                        .on_input(Message::ServerPasswordChanged)
                        .password()
                        .width(Length::Fill),
                ].spacing(10).align_items(iced::alignment::Alignment::Center);
                
                let path_row = row![
                    text("Remote Path:").width(Length::Fixed(120.0)),
                    text_input("Enter remote directory path", &self.server_path)
                        .on_input(Message::ServerPathChanged)
                        .width(Length::Fill),
                ].spacing(10).align_items(iced::alignment::Alignment::Center);

                column![
                    host_row,
                    user_row,
                    pass_row,
                    path_row,
                ].spacing(10)
            } else {
                column![]
            };
            
            let save_button = button("Save Configuration")
                .on_press(Message::SaveConfig)
                .padding(10);
                
            let config_column = column![
                handbrake_path_row,
                encoder_row,
                threads_row,
                eject_checkbox,
                server_toggle,
                server_panel,
                save_button,
            ]
            .spacing(10)
            .padding(10)
            .width(Length::Fill);

            container(config_column) // Wrap the column in a container to apply style
                .style(iced::theme::Container::Box)
                .into() // Convert container to Element
        } else {
            container(column![]).into() // Return an empty container element
        };
        
        // Title selection list (only shown when scan is complete)
        let titles_list = if self.scan_complete && !self.titles.is_empty() && !self.main_feature_only {
            let mut list_column = Column::new().spacing(5);
            
            list_column = list_column.push(
                text("Select Titles to Rip:")
                    .size(16)
                    .width(Length::Fill),
            );
            
            for title in &self.titles {
                let is_selected = self.selected_titles.contains(&title.number);
                
                let duration = utils::format_duration(title.duration);
                let title_row = row![
                    checkbox(
                        "", 
                        is_selected,
                        move |checked| Message::TitleSelected(title.number, checked)
                    ),
                    text(format!(
                        "Title {}: {}x{}, {}", 
                        title.number,
                        title.size.width,
                        title.size.height,
                        duration
                    )),
                ]
                .spacing(10)
                .align_items(iced::alignment::Alignment::Center);
                
                list_column = list_column.push(title_row);
            }
            
            scrollable(list_column)
                .height(Length::Fill)
                .width(Length::Fill)
        } else {
            scrollable(
                text(if self.scan_complete {
                    if self.main_feature_only {
                        "Main feature will be selected automatically."
                    } else {
                        "No titles found. Try scanning a different DVD."
                    }
                } else {
                    "Scan a DVD to see titles."
                })
                .width(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
            )
            .height(Length::Fill)
            .width(Length::Fill)
        };
        
        // Progress bar
        let progress = if self.is_ripping {
            progress_bar(0.0..=1.0, self.rip_progress)
                .height(Length::Fixed(20.0))
        } else {
            progress_bar(0.0..=1.0, 0.0)
                .height(Length::Fixed(20.0))
        };
        
        // Status message
        let status = text(&self.status_message)
            .width(Length::Fill)
            .horizontal_alignment(iced::alignment::Horizontal::Center);
        
        // Error message container (only shown when there's an error)
        let error_container = if let Some(error) = &self.error_message {
            let error_text = text(error)
                .width(Length::Fill)
                .horizontal_alignment(iced::alignment::Horizontal::Center)
                .size(16)
                .style(Color::from_rgb(0.8, 0.0, 0.0)); // Corrected: .color to .style for iced 0.10+
                
            let dismiss_button = button("Dismiss")
                .on_press(Message::DismissError)
                .padding(5)
                .width(Length::Fixed(100.0));
                
            container(
                column![
                    error_text,
                    dismiss_button,
                ]
                .spacing(10)
                .align_items(iced::alignment::Alignment::Center)
            )
            .padding(10)
            .width(Length::Fill)
            .style(iced::theme::Container::Box)
        } else {
            container(
                text("")
                    .width(Length::Fill)
            )
            .width(Length::Fill)
            .height(Length::Fixed(0.0))
        };
        
        // Main layout
        let content = column![
            title_text, // Use renamed variable
            error_container,
            input_row,
            output_row,
            action_row,
            options_row,
            config_panel, // Use the new container
            titles_list,
            progress,
            status,
        ]
        .padding(20)
        .spacing(20)
        .width(Length::Fill)
        .height(Length::Fill);
        
        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

/// Open a folder selection dialog
async fn browse_for_folder(title_str: &str) -> Result<PathBuf> { // Changed title to title_str
    let dialog = rfd::AsyncFileDialog::new()
        .set_title(title_str) // Use title_str
        .pick_folder()
        .await;

    dialog
        .map(|handle| handle.path().to_path_buf())
        .ok_or_else(|| AppError::UserError(format!("No folder selected: {}", title_str))) // Use title_str
}

#[derive(Clone)]
enum AppStatusPollState {
    Initial,
    Polling,
    Finished,
}