use crate::dvd::types::Title;

/// UI-specific state that doesn't belong in the core app state
#[derive(Debug, Clone)]
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
    pub status_message: String,
    pub error_message: String,

    /// Configuration fields (temporary storage before saving)
    pub config_temp: ConfigTemp,

    /// Dialog states
    pub show_about_dialog: bool,
    pub show_update_dialog: bool,
    pub show_cache_clear_dialog: bool,

    /// Update checking state
    pub update_status: UpdateStatus,
    pub checking_updates: bool,
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
}

/// Update checking status
#[derive(Debug, Clone)]
pub enum UpdateStatus {
    Unknown,
    UpToDate,
    UpdateAvailable { version: String, url: String },
    Error(String),
}

impl Default for UiState {
    fn default() -> Self {
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
        }
    }
}

impl Default for ConfigTemp {
    fn default() -> Self {
        Self {
            handbrake_path: String::new(),
            encode_algo: "x264".to_string(),
            thread_count: "4".to_string(),
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
        }
    }
}

impl UiState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update selected titles when titles list changes
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
    pub fn toggle_title(&mut self, index: usize) {
        if index < self.selected_titles.len() {
            self.selected_titles[index] = !self.selected_titles[index];
        }
    }

    /// Set error message and clear after delay
    pub fn set_error(&mut self, message: String) {
        self.error_message = message;
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error_message.clear();
    }

    /// Set status message
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
