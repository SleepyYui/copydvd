use crate::dvd::types::Title;
use crate::handbrake_manager::HandBrakeManager;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};

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

    /// Channel for receiving HandBrake status updates
    pub handbrake_update_receiver: Option<mpsc::UnboundedReceiver<HandBrakeUpdate>>,

    /// Download progress for async operations
    #[allow(dead_code)]
    pub download_progress: Option<Arc<std::sync::Mutex<f32>>>,

    /// HandBrake phase progress for complex operations
    #[allow(dead_code)]
    pub handbrake_phase_progress: Option<Arc<std::sync::Mutex<(HandBrakeOperationStatus, f32)>>>,

    /// Toast notifications for better user feedback
    #[allow(dead_code)]
    pub toast_notifications: Vec<ToastNotification>,
}

#[derive(Debug, Clone)]
pub struct HandBrakeUpdate {
    pub status: HandBrakeOperationStatus,
    pub version: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToastNotification {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

#[allow(dead_code)]
impl ToastNotification {
    pub fn new(message: String, toast_type: ToastType) -> Self {
        Self {
            message,
            toast_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(5), // Default 5 seconds
        }
    }

    pub fn with_duration(message: String, toast_type: ToastType, duration: Duration) -> Self {
        Self {
            message,
            toast_type,
            created_at: Instant::now(),
            duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }

    pub fn remaining_ratio(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32();
        (total - elapsed) / total
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

// Global sender for HandBrake updates - not ideal but necessary for async communication
static mut HANDBRAKE_UPDATE_SENDER: Option<mpsc::UnboundedSender<HandBrakeUpdate>> = None;

pub fn init_handbrake_update_sender(sender: mpsc::UnboundedSender<HandBrakeUpdate>) {
    unsafe {
        HANDBRAKE_UPDATE_SENDER = Some(sender);
    }
}

pub fn send_handbrake_update(update: HandBrakeUpdate) {
    unsafe {
        if let Some(ref sender) = HANDBRAKE_UPDATE_SENDER {
            let _ = sender.send(update);
        }
    }
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

    #[allow(dead_code)]
    pub fn icon(&self) -> &'static str {
        match self {
            Tab::Main => "",
            Tab::Config => "",
            Tab::Server => "",
            Tab::HandBrake => "",
            Tab::About => "",
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
            handbrake_update_receiver: None,
            download_progress: None,
            handbrake_phase_progress: None,
            toast_notifications: Vec::new(),
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

    /// Add a toast notification
    #[allow(dead_code)]
    pub fn add_toast(&mut self, message: String, toast_type: ToastType) {
        self.toast_notifications
            .push(ToastNotification::new(message, toast_type));
    }

    /// Add a toast notification with custom duration
    #[allow(dead_code)]
    pub fn add_toast_with_duration(
        &mut self,
        message: String,
        toast_type: ToastType,
        duration: Duration,
    ) {
        self.toast_notifications
            .push(ToastNotification::with_duration(
                message, toast_type, duration,
            ));
    }

    /// Clean up expired toast notifications
    #[allow(dead_code)]
    pub fn cleanup_expired_toasts(&mut self) {
        self.toast_notifications.retain(|toast| !toast.is_expired());
    }

    /// Clear all toast notifications
    #[allow(dead_code)]
    pub fn clear_toasts(&mut self) {
        self.toast_notifications.clear();
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
