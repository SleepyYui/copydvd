use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use iced::{
    Application, Command, Element, Settings, Theme,
    widget::{button, column, container, progress_bar, row, text, text_input, scrollable, Column, Container, checkbox},
    executor, Length, Subscription,
};
use iced_futures::subscription;
use tracing::{debug, info, warn};

use crate::app::{AppState, AppStatus};
use crate::config::Config;
use crate::dvd::{Dvd, Title};
use crate::error::{AppError, Result};
use crate::utils;
use crate::upload;

/// Run the GUI application
pub async fn run(app_state: Arc<Mutex<AppState>>) -> Result<()> {
    let gui_settings = Settings {
        window: iced::window::Settings {
            size: (800, 600),
            position: iced::window::Position::Centered,
            min_size: Some((640, 480)),
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: None,
        },
        id: None,
        flags: app_state,
        default_font: None,
        default_text_size: 16,
        text_multithreading: true,
        antialiasing: true,
        exit_on_close_request: true,
        try_opengles_first: false,
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
    titles: Vec<Title>,
}

#[derive(Debug, Clone)]
enum Message {
    InputPathChanged(String),
    OutputPathChanged(String),
    BrowseInput,
    BrowseOutput,
    ScanDvd,
    TitleSelected(usize, bool),
    ToggleMainFeature(bool),
    ToggleChapterSplit(bool),
    ToggleUpload(bool),
    StartRipping,
    CancelRipping,
    StatusUpdate(String),
    ScanComplete(Vec<Title>),
    RipProgress(f32),
    UploadProgress(f32),
    RipComplete,
    UploadComplete,
    Error(String),
    Tick,
}

impl Application for DvdRipperGui {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Arc<Mutex<AppState>>;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        let app_state = flags;
        
        let gui = Self {
            app_state,
            input_path: String::new(),
            output_path: String::new(),
            selected_titles: Vec::new(),
            chapter_split: false,
            main_feature_only: false,
            upload_to_server: false,
            is_scanning: false,
            is_ripping: false,
            is_uploading: false,
            scan_complete: false,
            rip_progress: 0.0,
            upload_progress: 0.0,
            status_message: "Ready".to_string(),
            titles: Vec::new(),
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
                        |result| match result {
                            Ok(_) => unreachable!(),
                            Err(e) => Message::Error(e),
                        },
                    );
                }
                
                self.is_scanning = true;
                self.scan_complete = false;
                self.status_message = "Scanning DVD...".to_string();
                
                let input_path = PathBuf::from(&self.input_path);
                let app_state = Arc::clone(&self.app_state);
                
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
                        
                        // Update app state with DVD
                        let mut state = app_state.lock().await;
                        let dvd = Arc::new(Mutex::new(dvd.clone()));
                        state.dvd = Some(dvd);
                        
                        // Return titles for display
                        Ok(dvd.titles.clone())
                    },
                    |result| match result {
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
                        |result| match result {
                            Ok(_) => unreachable!(),
                            Err(e) => Message::Error(e),
                        },
                    );
                }
                
                if !self.main_feature_only && self.selected_titles.is_empty() {
                    return Command::perform(
                        async { Err("Please select at least one title to rip".to_string()) },
                        |result| match result {
                            Ok(_) => unreachable!(),
                            Err(e) => Message::Error(e),
                        },
                    );
                }
                
                self.is_ripping = true;
                self.status_message = "Starting ripping process...".to_string();
                self.rip_progress = 0.0;
                
                let app_state = Arc::clone(&self.app_state);
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
                        let mut state = app_state.lock().await;
                        
                        // Get DVD from app state
                        let dvd = state.dvd.as_ref()
                            .ok_or_else(|| "DVD not scanned".to_string())?
                            .clone();
                        
                        let selected_titles = if main_feature_only {
                            // Find main feature (longest title)
                            let dvd_lock = dvd.lock().await;
                            dvd_lock.find_main_feature()
                                .map(|title| vec![title.number])
                                .ok_or_else(|| "No main feature found".to_string())?
                        } else {
                            selected_titles.ok_or_else(|| "No titles selected".to_string())?
                        };
                        
                        // Create ripping tasks
                        let dvd_lock = dvd.lock().await;
                        let tasks = dvd_lock.create_rip_tasks(output_path, Some(selected_titles), chapter_split);
                        let task_count = tasks.len();
                        
                        // Update app state
                        state.rip_tasks = tasks.clone();
                        state.status = AppStatus::Ripping { completed: 0, total: task_count };
                        
                        // Start ripping process in a background task
                        tokio::spawn(async move {
                            let mut completed = 0;
                            
                            for task in tasks.clone() {
                                // Rip the title
                                let result = dvd.lock().await.rip_title(&task).await;
                                
                                if let Err(e) = result {
                                    warn!("Failed to rip title {}: {}", task.title.number, e);
                                }
                                
                                // Update progress
                                completed += 1;
                                
                                // Update app state
                                let mut state = app_state.lock().await;
                                state.status = AppStatus::Ripping {
                                    completed,
                                    total: task_count,
                                };
                            }
                            
                            // Eject if configured
                            if dvd.lock().await.config.eject_after_rip {
                                if let Err(e) = dvd.lock().await.eject().await {
                                    warn!("Failed to eject DVD: {}", e);
                                }
                            }
                            
                            // Upload if requested
                            if upload_to_server {
                                let dvd_lock = dvd.lock().await;
                                if let Some(server_config) = &dvd_lock.config.server {
                                    // Get a descriptive name for the DVD
                                    let movie_name = dvd_lock.find_main_feature()
                                        .map(|title| format!("Movie_Title{}", title.number))
                                        .unwrap_or_else(|| "DVD_Rip".to_string());
                                        
                                    // Perform uploads
                                    for (i, task) in tasks.iter().enumerate() {
                                        if task.output_path.exists() {
                                            let upload_task = upload::create_upload_task(
                                                &task.output_path,
                                                server_config,
                                                &movie_name
                                            );
                                            
                                            info!("Uploading {} to {}", 
                                                  task.output_path.display(), 
                                                  upload_task.remote_path);
                                            
                                            // Update state with uploading status
                                            let mut state = app_state.lock().await;
                                            state.status = AppStatus::Ripping { 
                                                completed: task_count + i, 
                                                total: task_count * 2 
                                            };
                                            drop(state);
                                            
                                            if let Err(e) = upload::upload_file(&upload_task).await {
                                                warn!("Upload failed for {}: {}", 
                                                      task.output_path.display(), e);
                                            }
                                        }
                                    }
                                } else {
                                    warn!("Upload requested but no server configuration found");
                                }
                            }
                            
                            // Set status to completed
                            let mut state = app_state.lock().await;
                            state.status = AppStatus::Completed;
                        });
                        
                        Ok(())
                    },
                    |result| match result {
                        Ok(_) => Message::StatusUpdate("Ripping in progress...".to_string()),
                        Err(e) => {
                            warn!("Failed to start ripping: {}", e);
                            Message::Error(e)
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
            
            Message::UploadComplete => {
                self.is_uploading = false;
                self.status_message = "Upload completed".to_string();
                Command::none()
            }
            
            Message::Error(message) => {
                self.is_scanning = false;
                self.is_ripping = false;
                self.status_message = format!("Error: {}", message);
                warn!("GUI error: {}", message);
                Command::none()
            }
            
            Message::Tick => {
                // Poll application state for updates
                let app_state = Arc::clone(&self.app_state);
                
                Command::perform(
                    async move {
                        let state = app_state.lock().await;
                        state.status.clone()
                    },
                    |status| {
                        match status {
                            AppStatus::Ripping { completed, total } => {
                                // When total is doubled, we're in the upload phase
                                if total > completed && total / 2 >= completed {
                                    // Still ripping
                                    let progress = completed as f32 / (total / 2) as f32;
                                    Message::RipProgress(progress)
                                } else if total > completed {
                                    // Now uploading
                                    let base = (total / 2) as f32;
                                    let upload_progress = (completed as f32 - base) / base;
                                    Message::UploadProgress(upload_progress)
                                } else {
                                    Message::RipComplete
                                }
                            }
                            AppStatus::Completed => Message::RipComplete,
                            AppStatus::Error(msg) => Message::Error(msg),
                            _ => Message::Tick, // No change
                        }
                    },
                )
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        // Poll the app state periodically when ripping is in progress
        if self.is_ripping {
            subscription::unfold(0, |state| async move {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                (Message::Tick, state + 1)
            })
        } else {
            Subscription::none()
        }
    }

    fn view(&self) -> Element<Message> {
        let title = text("DVD Ripper")
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
        
        let action_row = row![
            scan_button,
            rip_button,
            cancel_button,
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
        
        // Main layout
        let content = column![
            title,
            input_row,
            output_row,
            action_row,
            options_row,
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
async fn browse_for_folder(title: &str) -> Result<PathBuf, String> {
    let dialog = rfd::AsyncFileDialog::new()
        .set_title(title)
        .pick_folder()
        .await;
        
    dialog.ok_or_else(|| "No folder selected".to_string())
}