use crate::dvd::types::Title;
use crate::handbrake_manager::HandBrakeManager;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// UI-specific state that doesn't belong in the core app state
#[derive(Debug)]
pub struct UiState {
    /// Currently active tab
    pub active_tab: Tab,

    /// Input and output paths
    pub input_path: String,
    pub output_path: String,

    /// Title selection state
    pub selected_titles: Vec<bool>,
    pub titles: Vec<Title>,

    /// Options
    pub main_feature_only: bool,
    pub chapter_split: bool,
    pub upload_to_server: bool,

    /// Status messages
    #[allow(dead_code)]
    pub status_message: String,
    pub error_message: String,

    /// Configuration fields (temporary storage before saving)
    pub config_temp: ConfigTemp,

    /// Dialog states
    #[allow(dead_code)]
    pub show_about_dialog: bool,
    pub show_update_dialog: bool,
    pub show_cache_clear_dialog: bool,

    /// Update checking state
    pub update_status: UpdateStatus,
    pub checking_updates: bool,

    /// HandBrake manager for download/management operations
    pub handbrake_manager: Arc<Mutex<HandBrakeManager>>,

    /// HandBrake operation status
    pub handbrake_status: HandBrakeOperationStatus,

    /// HandBrake version information
    pub handbrake_version: Option<String>,

    /// HandBrake verification message visibility timeout
    pub handbrake_verification_visible_until: Option<Instant>,

    /// Simple channel for UI updates from async HandBrake operations
    pub handbrake_ui_receiver:
        Option<std::sync::mpsc::Receiver<(HandBrakeOperationStatus, Option<String>)>>,

    /// Download progress for async operations
    #[allow(dead_code)]
    pub download_progress: Option<Arc<std::sync::Mutex<f32>>>,

    /// HandBrake phase progress for complex operations
    #[allow(dead_code)]
    pub handbrake_phase_progress: Option<Arc<std::sync::Mutex<(HandBrakeOperationStatus, f32)>>>,
}

// Global sender for simple HandBrake UI updates
static mut HANDBRAKE_UI_SENDER: Option<
    std::sync::mpsc::Sender<(HandBrakeOperationStatus, Option<String>)>,
> = None;

pub fn init_handbrake_ui_sender(
    sender: std::sync::mpsc::Sender<(HandBrakeOperationStatus, Option<String>)>,
) {
    unsafe {
        HANDBRAKE_UI_SENDER = Some(sender);
    }
}

#[allow(static_mut_refs)]
pub fn send_handbrake_ui_update(status: HandBrakeOperationStatus, version: Option<String>) {
    unsafe {
        if let Some(ref sender) = HANDBRAKE_UI_SENDER {
            let _ = sender.send((status, version));
        }
    }
}

/// Available tabs in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Main,
    Config,
    Server,
    HandBrake,
    About,
}

impl Tab {
    pub fn name(&self) -> &'static str {
        match self {
            Tab::Main => "Main",
            Tab::Config => "Config",
            Tab::Server => "Server",
            Tab::HandBrake => "HandBrake",
            Tab::About => "About",
        }
    }

    #[allow(dead_code, reason = "Icons removed from UI")]
    pub fn icon(&self) -> &'static str {
        match self {
            Tab::Main => "play",
            Tab::Config => "gear",
            Tab::Server => "upload",
            Tab::HandBrake => "wrench",
            Tab::About => "info",
        }
    }

    pub fn all() -> Vec<Tab> {
        vec![
            Tab::Main,
            Tab::Config,
            Tab::Server,
            Tab::HandBrake,
            Tab::About,
        ]
    }
}

/// Temporary configuration storage for UI editing
#[derive(Debug, Clone)]
pub struct ConfigTemp {
    pub handbrake_path: String,
    pub encode_algo: String,
    pub video_codec: String,
    pub thread_count: String,
    pub eject_after_rip: bool,

    // HandBrake management
    pub auto_download: bool,
    pub prefer_system: bool,
    pub max_cache_size_mb: String,
    pub verify_on_startup: bool,
    pub cache_info: Option<(String, String)>,

    // Server config
    pub server_host: String,
    pub server_username: String,
    pub server_password: String,
    pub server_path: String,

    // Advanced encoding options
    pub gpu_acceleration: bool,
    pub two_pass_encoding: bool,
    #[allow(dead_code)]
    pub fast_start: bool,
    pub custom_args: String,

    // File management options
    pub organize_by_date: bool,
    pub auto_cleanup: bool,
    pub naming_pattern: String,

    // Server transfer options
    pub compress_transfer: bool,
    pub resume_uploads: bool,
    pub preserve_permissions: bool,
    pub delete_after_upload: bool,
}

/// Update checking status
#[derive(Debug, Clone)]
pub enum UpdateStatus {
    Unknown,
    UpToDate,
    UpdateAvailable {
        #[allow(dead_code)]
        version: String,
        #[allow(dead_code)]
        url: String,
    },
    Error(#[allow(dead_code)] String),
}

/// HandBrake operation status
#[derive(Debug, Clone)]
pub enum HandBrakeOperationStatus {
    Idle,
    CheckingStatus,
    #[allow(dead_code)]
    Downloading {
        progress: f32,
    },
    #[allow(dead_code)]
    Extracting,
    #[allow(dead_code)]
    Installing,
    VerifyingInstallation,
    ClearingCache,
    #[allow(dead_code)]
    Error(String),
}

impl Default for UiState {
    fn default() -> Self {
        let handbrake_manager = HandBrakeManager::new()
            .map(|hm| Arc::new(Mutex::new(hm)))
            .unwrap_or_else(|_| Arc::new(Mutex::new(HandBrakeManager::default())));

        Self {
            active_tab: Tab::Main,
            input_path: String::new(),
            output_path: String::new(),
            selected_titles: Vec::new(),
            titles: Vec::new(),
            main_feature_only: false,
            chapter_split: false,
            upload_to_server: false,
            status_message: "Ready".to_string(),
            error_message: String::new(),
            config_temp: ConfigTemp::default(),
            show_about_dialog: false,
            show_update_dialog: false,
            show_cache_clear_dialog: false,
            update_status: UpdateStatus::Unknown,
            checking_updates: false,
            handbrake_manager,
            handbrake_status: HandBrakeOperationStatus::Idle,
            handbrake_version: None,
            handbrake_verification_visible_until: None,
            handbrake_ui_receiver: None,
            download_progress: None,
            handbrake_phase_progress: None,
        }
    }
}

impl Default for ConfigTemp {
    fn default() -> Self {
        Self {
            handbrake_path: String::new(),
            encode_algo: "MP4".to_string(),
            video_codec: "H.264".to_string(),
            thread_count: "0".to_string(),
            eject_after_rip: true,
            auto_download: true,
            prefer_system: true,
            max_cache_size_mb: "100".to_string(),
            verify_on_startup: true,
            cache_info: None,
            server_host: String::new(),
            server_username: String::new(),
            server_password: String::new(),
            server_path: String::new(),
            gpu_acceleration: false,
            two_pass_encoding: false,
            fast_start: true,
            custom_args: String::new(),
            organize_by_date: false,
            auto_cleanup: true,
            naming_pattern: "{title} - {date}".to_string(),
            compress_transfer: true,
            resume_uploads: true,
            preserve_permissions: true,
            delete_after_upload: false,
        }
    }
}

impl UiState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start HandBrake download operation
    #[allow(dead_code)]
    pub async fn download_handbrake(&mut self) -> Result<(), String> {
        self.handbrake_status = HandBrakeOperationStatus::Downloading { progress: 0.0 };

        let manager = self.handbrake_manager.clone();
        let result = {
            let mut guard = manager.lock().await;
            guard.get_handbrake_path().await
        };
        match result {
            Ok(_) => {
                self.handbrake_status = HandBrakeOperationStatus::Idle;
                self.set_status("HandBrake downloaded successfully".to_string());
                Ok(())
            }
            Err(e) => {
                self.handbrake_status = HandBrakeOperationStatus::Error(e.to_string());
                self.set_error(format!("Failed to download HandBrake: {}", e));
                Err(e.to_string())
            }
        }
    }

    /// Verify HandBrake installation
    #[allow(dead_code)]
    pub async fn verify_handbrake(&mut self) -> Result<String, String> {
        self.handbrake_status = HandBrakeOperationStatus::VerifyingInstallation;

        let manager = self.handbrake_manager.clone();
        let result = {
            let mut guard = manager.lock().await;
            guard.verify_handbrake().await
        };
        match result {
            Ok(path) => {
                self.handbrake_status = HandBrakeOperationStatus::Idle;
                self.set_status("HandBrake verification successful".to_string());
                Ok(path)
            }
            Err(e) => {
                self.handbrake_status = HandBrakeOperationStatus::Error(e.to_string());
                self.set_error(format!("HandBrake verification failed: {}", e));
                Err(e.to_string())
            }
        }
    }

    /// Clear HandBrake cache
    #[allow(dead_code)]
    pub async fn clear_handbrake_cache(&mut self) -> Result<(), String> {
        self.handbrake_status = HandBrakeOperationStatus::ClearingCache;

        let manager = self.handbrake_manager.clone();
        let result = {
            let guard = manager.lock().await;
            guard.clear_cache()
        };
        match result {
            Ok(()) => {
                self.handbrake_status = HandBrakeOperationStatus::Idle;
                self.set_status("HandBrake cache cleared successfully".to_string());
                Ok(())
            }
            Err(e) => {
                self.handbrake_status = HandBrakeOperationStatus::Error(e.to_string());
                self.set_error(format!("Failed to clear cache: {}", e));
                Err(e.to_string())
            }
        }
    }

    /// Refresh cache information
    #[allow(dead_code)]
    pub async fn refresh_cache_info(&mut self) {
        let manager = self.handbrake_manager.clone();
        let result = {
            let guard = manager.lock().await;
            guard.get_cache_info()
        };
        match result {
            Ok((cache_dir, size)) => {
                let size_mb = size as f64 / 1024.0 / 1024.0;
                self.config_temp.cache_info = Some((
                    cache_dir.to_string_lossy().to_string(),
                    format!("{:.1} MB", size_mb),
                ));
            }
            Err(e) => {
                self.set_error(format!("Failed to get cache info: {}", e));
            }
        }
    }

    /// Update selected titles when titles list changes
    #[allow(dead_code)]
    pub fn update_titles(&mut self, new_titles: Vec<Title>) {
        if self.titles.len() != new_titles.len() {
            self.selected_titles = vec![false; new_titles.len()];
        }
        self.titles = new_titles;
    }

    /// Get number of selected titles
    pub fn selected_title_count(&self) -> usize {
        self.selected_titles
            .iter()
            .filter(|&&selected| selected)
            .count()
    }

    /// Select all titles
    pub fn select_all_titles(&mut self) {
        self.selected_titles.fill(true);
    }

    /// Deselect all titles
    pub fn deselect_all_titles(&mut self) {
        self.selected_titles.fill(false);
    }

    /// Toggle title selection
    #[allow(dead_code)]
    pub fn toggle_title(&mut self, index: usize) {
        if index < self.selected_titles.len() {
            self.selected_titles[index] = !self.selected_titles[index];
        }
    }

    /// Set error message and clear after delay
    #[allow(dead_code)]
    pub fn set_error(&mut self, message: String) {
        self.error_message = message;
    }

    /// Clear error message
    #[allow(dead_code)]
    pub fn clear_error(&mut self) {
        self.error_message.clear();
    }

    /// Set status message
    #[allow(dead_code)]
    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
    }

    /// Load config into temporary storage
    pub fn load_config_temp(&mut self, config: &crate::config::Config) {
        self.config_temp.handbrake_path = config
            .handbrake_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        self.config_temp.encode_algo = config.encode_algo.clone();
        self.config_temp.video_codec = config.video_codec.clone();
        self.config_temp.thread_count = config.thread_count.to_string();
        self.config_temp.eject_after_rip = config.eject_after_rip;

        self.config_temp.auto_download = config.handbrake_management.auto_download;
        self.config_temp.prefer_system = config.handbrake_management.prefer_system;
        self.config_temp.max_cache_size_mb =
            config.handbrake_management.max_cache_size_mb.to_string();
        self.config_temp.verify_on_startup = config.handbrake_management.verify_on_startup;

        if let Some(server) = &config.server {
            self.config_temp.server_host = server.host.clone();
            self.config_temp.server_username = server.username.clone();
            self.config_temp.server_password = server.password.clone().unwrap_or_default();
            self.config_temp.server_path = server.path.clone();
        }

        self.output_path = config.output_dir.to_string_lossy().to_string();
    }
}

/// Toast notification types
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code, reason = "Future feature for in-app toast notifications")]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

/// Toast notification structure
#[derive(Debug, Clone)]
#[allow(dead_code, reason = "Future feature for in-app toast notifications")]
pub struct ToastNotification {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl ToastNotification {
    #[allow(dead_code, reason = "Future feature for in-app toast notifications")]
    pub fn new(message: String, toast_type: ToastType) -> Self {
        let duration = match toast_type {
            ToastType::Success => Duration::from_secs(3),
            ToastType::Info => Duration::from_secs(2),
            ToastType::Warning => Duration::from_secs(4),
            ToastType::Error => Duration::from_secs(4),
        };

        Self {
            message,
            toast_type,
            created_at: Instant::now(),
            duration,
        }
    }

    #[allow(dead_code, reason = "Future feature for in-app toast notifications")]
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }

    #[allow(dead_code, reason = "Future feature for in-app toast notifications")]
    pub fn remaining_ratio(&self) -> f32 {
        let elapsed = self.created_at.elapsed();
        if elapsed >= self.duration {
            0.0
        } else {
            1.0 - (elapsed.as_secs_f32() / self.duration.as_secs_f32())
        }
    }
}
