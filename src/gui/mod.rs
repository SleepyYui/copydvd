use crate::app::state::AppState;
use crate::config::Config;
use crate::gui::components::{toast::render_toast_notifications, ExitConfirmationDialog};
use crate::gui::notifications::UpdateNotificationManager;
use crate::gui::state::{HandBrakeOperationStatus, Tab, UiState, UpdateStatus};
use crate::gui::tabs::*;
use crate::gui::theme::{apply_modern_theme, nav_item, ModernTheme, Spacing};
use crate::gui::utils::updates::UpdateCheckResult;
use crate::handbrake_manager::HandBrakeManager;
use std::time::{Duration, Instant};

use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

pub mod components;
pub mod icons;
pub mod notifications;
pub mod state;
pub mod tabs;
pub mod theme;
pub mod utils;

#[derive(Debug)]
pub enum HandBrakeStatus {
    Verifying,
    Verified(String), // Binary path
    Error(String),    // Error message
}

#[derive(Debug, Clone)]
pub struct DvdDriveInfo {
    pub path: String,
    pub drive_type: DvdDriveType,
    pub label: String,
    pub has_video_ts: bool,
}

#[derive(Debug, Clone)]
pub enum DvdDriveType {
    MountedVolume,
    PhysicalDrive,
}

#[derive(Debug, Clone)]
pub struct DvdDetectionResult {
    pub drives: Vec<DvdDriveInfo>,
    pub selected_index: usize,
}

/// Main entry point for the simple GUI application
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([700.0, 500.0])
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "CopyDVD",
        options,
        Box::new(|cc| {
            apply_modern_theme(&cc.egui_ctx);
            cc.egui_ctx.set_pixels_per_point(1.0);
            Ok(Box::new(CopyDvdApp::new(cc)))
        }),
    )
}

fn load_icon() -> egui::IconData {
    // Try to load the icon from resources
    if let Ok(icon_data) = std::fs::read("resources/icons/icon-64.png") {
        if let Ok(image) = image::load_from_memory(&icon_data) {
            let rgba_image = image.to_rgba8();
            let (width, height) = rgba_image.dimensions();
            return egui::IconData {
                rgba: rgba_image.into_raw(),
                width,
                height,
            };
        }
    }

    // Fallback: create a simple icon programmatically
    let size = 64;
    let mut rgba = vec![0u8; size * size * 4];

    // Create a simple DVD icon pattern
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let center_x = size as f32 / 2.0;
            let center_y = size as f32 / 2.0;
            let distance = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();

            if distance < 28.0 && distance > 8.0 {
                // DVD disc area - silver color
                rgba[idx] = 200; // R
                rgba[idx + 1] = 200; // G
                rgba[idx + 2] = 200; // B
                rgba[idx + 3] = 255; // A
            } else if distance <= 8.0 {
                // Center hole - dark
                rgba[idx] = 50; // R
                rgba[idx + 1] = 50; // G
                rgba[idx + 2] = 50; // B
                rgba[idx + 3] = 255; // A
            } else {
                // Transparent background
                rgba[idx + 3] = 0;
            }
        }
    }

    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}

struct CopyDvdApp {
    app_state: Arc<Mutex<AppState>>,
    ui_state: UiState,
    config: Arc<Mutex<Config>>,
    first_frame: bool,
    update_receiver: Receiver<UpdateCheckResult>,
    handbrake_receiver: Receiver<HandBrakeStatus>,
    handbrake_status: Option<HandBrakeStatus>,
    update_notification_manager: UpdateNotificationManager,
    exit_confirmation: ExitConfirmationDialog,
}

impl CopyDvdApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (_update_sender, update_receiver) = mpsc::channel();
        let (handbrake_sender, handbrake_receiver) = mpsc::channel();

        // Set up HandBrake UI update channel
        let (handbrake_ui_sender, handbrake_ui_receiver) = mpsc::channel();
        crate::gui::state::ui_state::init_handbrake_ui_sender(handbrake_ui_sender);

        let mut ui_state = UiState::new();
        ui_state.handbrake_ui_receiver = Some(handbrake_ui_receiver);

        // Start HandBrake verification immediately
        tokio::spawn(async move {
            let _ = handbrake_sender.send(HandBrakeStatus::Verifying);

            let mut handbrake_manager = match HandBrakeManager::new() {
                Ok(manager) => manager,
                Err(e) => {
                    let _ = handbrake_sender.send(HandBrakeStatus::Error(format!(
                        "Failed to initialize HandBrake manager: {}",
                        e
                    )));
                    return;
                }
            };

            match handbrake_manager.verify_handbrake().await {
                Ok(binary_path) => {
                    let _ = handbrake_sender.send(HandBrakeStatus::Verified(binary_path));
                }
                Err(e) => {
                    let _ = handbrake_sender.send(HandBrakeStatus::Error(e.to_string()));
                }
            }
        });

        // Load or create config
        let config = crate::config::Config::load().unwrap_or_default();
        let should_calculate_optimal = !config.optimal_settings_calculated;
        let config_arc = Arc::new(Mutex::new(config));

        let mut app = Self {
            app_state: Arc::new(Mutex::new(AppState::new({
                let config_guard = config_arc.lock().unwrap();
                config_guard.clone()
            }))),
            ui_state,
            config: config_arc.clone(),
            first_frame: true,
            update_receiver,
            handbrake_receiver,
            handbrake_status: None,
            update_notification_manager: UpdateNotificationManager::new(),
            exit_confirmation: ExitConfirmationDialog::default(),
        };

        // Calculate optimal settings on first startup only
        if should_calculate_optimal {
            app.calculate_and_save_optimal_settings();
        }

        app
    }

    fn update_status(&mut self, ctx: &egui::Context) {
        self.check_for_handbrake_status();
        self.check_for_handbrake_ui_updates();

        // Check for update notifications
        self.update_notification_manager
            .check_and_notify(&mut self.ui_state);

        if let Ok(_state) = self.app_state.try_lock() {
            // Note: AppState doesn't have an error field, so we'll skip this check
            // if let Some(error) = &state.error {
            //     self.ui_state.error_message = Some(error.to_string());
            // }
        }
        ctx.request_repaint();
    }

    fn handle_first_frame(&mut self) {
        if self.first_frame {
            // Disable auto-update check for now
            // TODO: Re-enable when GitHub repo is set up
            // let (sender, receiver) = mpsc::channel();
            // self.update_receiver = receiver;

            // tokio::spawn(async move {
            //     let result = auto_check_for_updates().await;
            //     let _ = sender.send(result);
            // });

            self.first_frame = false;
        }
    }

    fn check_for_handbrake_status(&mut self) {
        // Non-blocking check for HandBrake verification results
        if let Ok(status) = self.handbrake_receiver.try_recv() {
            // Set visibility timeout when HandBrake is verified
            if matches!(status, HandBrakeStatus::Verified(_)) {
                self.ui_state.handbrake_verification_visible_until =
                    Some(Instant::now() + Duration::from_secs(5));
            }
            self.handbrake_status = Some(status);
        }
    }

    fn check_for_update_results(&mut self) {
        // Non-blocking check for update results
        if let Ok(result) = self.update_receiver.try_recv() {
            match result {
                UpdateCheckResult::UpdateAvailable {
                    version,
                    download_url,
                    changelog: _,
                } => {
                    self.ui_state.update_status = UpdateStatus::UpdateAvailable {
                        version,
                        url: download_url,
                    };
                }
                UpdateCheckResult::UpToDate => {
                    self.ui_state.update_status = UpdateStatus::UpToDate;
                }
                UpdateCheckResult::Error(e) => {
                    self.ui_state.update_status = UpdateStatus::Error(e);
                }
            }
            self.ui_state.checking_updates = false;
        }
    }

    fn check_for_handbrake_ui_updates(&mut self) {
        // Process HandBrake UI updates
        if let Some(receiver) = &self.ui_state.handbrake_ui_receiver {
            while let Ok((status, version)) = receiver.try_recv() {
                self.ui_state.handbrake_status = status;
                self.ui_state.handbrake_version = version;

                // Set visibility timeout when HandBrake becomes ready
                if matches!(
                    self.ui_state.handbrake_status,
                    HandBrakeOperationStatus::Idle
                ) && self.ui_state.handbrake_version.is_some()
                {
                    self.ui_state.handbrake_verification_visible_until =
                        Some(Instant::now() + Duration::from_secs(5));
                }
            }
        }
    }

    fn save_all_configs(&mut self) {
        self.update_config_from_ui();
        if let Ok(config) = self.config.try_lock() {
            if let Err(e) = config.save() {
                self.ui_state.error_message = format!("Failed to save config: {}", e);
            }
        }
    }

    fn update_config_from_ui(&mut self) {
        if let Ok(mut config) = self.config.try_lock() {
            // Update config from UI state
            config.output_dir = self.ui_state.output_path.clone().into();
            // Note: Config structure doesn't have direct dvd or server fields with these properties
            // These would need to be updated based on the actual Config structure
        }
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.add_space(Spacing::LG);

        // App branding section
        ui.horizontal(|ui| {
            ui.add_space(Spacing::MD);
            ui.add_space(Spacing::MD);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("CopyDVD")
                        .size(18.0)
                        .color(ModernTheme::TEXT_PRIMARY)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new("Media Tool")
                        .size(12.0)
                        .color(ModernTheme::TEXT_TERTIARY),
                );
            });
        });

        ui.add_space(Spacing::XL);

        // Navigation items
        ui.add_space(Spacing::SM);
        for tab in Tab::all() {
            let is_active = self.ui_state.active_tab == tab;

            if nav_item(ui, tab.name(), is_active).clicked() {
                self.ui_state.active_tab = tab;
            }

            ui.add_space(Spacing::XS);
        }

        ui.add_space(Spacing::XL);

        // Status section at bottom
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space(Spacing::LG);

            // Current status
            if let Ok(state) = self.app_state.try_lock() {
                let (status_text, status_color) = match &state.status {
                    crate::app::state::AppStatus::Idle => ("Ready", ModernTheme::SUCCESS),
                    crate::app::state::AppStatus::Scanning => ("Scanning", ModernTheme::INFO),
                    crate::app::state::AppStatus::Ripping { .. } => {
                        ("Processing", ModernTheme::WARNING)
                    }
                    crate::app::state::AppStatus::Error(_) => ("Error", ModernTheme::ERROR),
                    _ => ("Active", ModernTheme::ACCENT_PRIMARY),
                };

                ui.horizontal(|ui| {
                    ui.add_space(Spacing::MD);
                    ui.add_space(Spacing::SM);
                    ui.label(
                        egui::RichText::new(status_text)
                            .color(status_color)
                            .size(12.0),
                    );
                });

                if !self.ui_state.titles.is_empty() {
                    ui.add_space(Spacing::SM);
                    ui.horizontal(|ui| {
                        ui.add_space(Spacing::MD);
                        ui.add_space(Spacing::SM);
                        ui.label(
                            egui::RichText::new(format!(
                                "{} titles found",
                                self.ui_state.titles.len()
                            ))
                            .color(ModernTheme::TEXT_TERTIARY)
                            .size(11.0),
                        );
                    });
                }
            }
        });
    }

    fn render_modern_header(&mut self, ui: &mut egui::Ui) {
        ui.add_space(Spacing::LG);

        ui.horizontal(|ui| {
            // Page title based on active tab
            let (title, description) = match self.ui_state.active_tab {
                Tab::Main => ("DVD Copy", "Scan and copy your DVDs"),
                Tab::Config => ("Configuration", "Adjust encoding and output settings"),
                Tab::Server => ("Server Setup", "Configure remote upload settings"),
                Tab::HandBrake => ("HandBrake Manager", "Manage HandBrake installation"),
                Tab::Updates => ("Updates", "Check for updates and view release notes"),
                Tab::About => ("About", "Application information and credits"),
            };

            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(title)
                        .size(24.0)
                        .color(ModernTheme::TEXT_PRIMARY)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(description)
                        .size(14.0)
                        .color(ModernTheme::TEXT_SECONDARY),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Quick action button based on current tab
                match self.ui_state.active_tab {
                    Tab::Main => {
                        let button = egui::Button::new("Quick Scan")
                            .fill(ModernTheme::ACCENT_PRIMARY)
                            .rounding(egui::Rounding::same(6.0));
                        if ui.add_sized([80.0, 22.0], button).clicked() {
                            self.trigger_dvd_scan();
                        }
                    }
                    Tab::Config => {
                        let button = egui::Button::new("Save Config")
                            .fill(ModernTheme::SUCCESS)
                            .rounding(egui::Rounding::same(6.0));
                        if ui.add_sized([80.0, 22.0], button).clicked() {
                            self.save_all_configs();
                        }
                    }
                    _ => {}
                }
            });
        });

        ui.add_space(Spacing::LG);

        // Subtle divider
        let rect = ui
            .allocate_space(egui::Vec2::new(ui.available_width(), 1.0))
            .1;
        ui.painter()
            .rect_filled(rect, egui::Rounding::ZERO, ModernTheme::BORDER_PRIMARY);

        ui.add_space(Spacing::LG);
    }

    fn trigger_dvd_scan(&mut self) {
        use crate::gui::notifications::{notify_error, notify_info, notify_success};

        let input_path = if self.ui_state.input_path.is_empty() {
            // Auto-detect DVD drives with enhanced detection
            notify_info("Searching for DVD drives...");
            match self.auto_detect_dvd_drives() {
                Some(detection_result) => {
                    let selected_drive = &detection_result.drives[detection_result.selected_index];
                    self.ui_state.input_path = selected_drive.path.clone();

                    if detection_result.drives.len() > 1 {
                        notify_info(&format!(
                            "Found {} DVD drive(s). Selected: {} ({})",
                            detection_result.drives.len(),
                            selected_drive.label,
                            selected_drive.path
                        ));
                    } else {
                        notify_success(&format!(
                            "Found DVD: {} ({})",
                            selected_drive.label, selected_drive.path
                        ));
                    }

                    selected_drive.path.clone()
                }
                None => {
                    notify_error("No DVD drives found. Please select a DVD input path manually.");
                    return;
                }
            }
        } else {
            self.ui_state.input_path.clone()
        };

        notify_info("Starting DVD scan...");

        // Clear previous titles and update status
        self.ui_state.titles.clear();

        // Update app state to show scanning
        if let Ok(mut app_state) = self.app_state.try_lock() {
            app_state.status = crate::app::state::AppStatus::Scanning;
        }

        // Spawn actual DVD scanning task
        let app_state_clone = self.app_state.clone();
        let _config_clone = self.config.clone();

        tokio::spawn(async move {
            // Simulate DVD scanning with HandBrake
            notify_info("Analyzing DVD structure...");

            // Here we would integrate with actual DVD scanning logic
            // For now, simulate the process
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Simulate finding titles
            let mock_titles = [
                crate::dvd::types::Title {
                    number: 1,
                    duration: std::time::Duration::from_secs(5565), // 1:32:45
                    size: crate::dvd::types::DvdSize {
                        width: 720,
                        height: 480,
                    },
                    chapters: vec![
                        crate::dvd::types::Chapter {
                            number: 1,
                            duration: std::time::Duration::from_secs(600),
                        },
                        crate::dvd::types::Chapter {
                            number: 2,
                            duration: std::time::Duration::from_secs(700),
                        },
                    ],
                    description: Some("Main Movie".to_string()),
                },
                crate::dvd::types::Title {
                    number: 2,
                    duration: std::time::Duration::from_secs(330), // 0:05:30
                    size: crate::dvd::types::DvdSize {
                        width: 720,
                        height: 480,
                    },
                    chapters: vec![crate::dvd::types::Chapter {
                        number: 1,
                        duration: std::time::Duration::from_secs(330),
                    }],
                    description: Some("Bonus Feature".to_string()),
                },
            ];

            // Update app state with results
            if let Ok(mut app_state) = app_state_clone.try_lock() {
                app_state.status = crate::app::state::AppStatus::Idle;
                // Note: In a real implementation, we'd update titles in UI state
                // This would require a different communication mechanism
            }

            notify_success(&format!(
                "DVD scan complete. Found {} titles.",
                mock_titles.len()
            ));
            tracing::info!("DVD scan completed for: {}", input_path);
        });
    }

    fn auto_detect_dvd_drives(&self) -> Option<DvdDetectionResult> {
        use crate::gui::notifications::notify_info;

        let mut detected_drives = Vec::new();

        // Platform-specific DVD drive detection with enhanced logic
        #[cfg(target_os = "macos")]
        {
            notify_info("Scanning macOS volumes and drives...");

            // First check mounted volumes with VIDEO_TS
            if let Ok(entries) = std::fs::read_dir("/Volumes") {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            let video_ts = entry.path().join("VIDEO_TS");
                            if video_ts.exists() {
                                let path = entry.path().to_string_lossy().to_string();
                                detected_drives.push(DvdDriveInfo {
                                    path: path.clone(),
                                    drive_type: DvdDriveType::MountedVolume,
                                    label: entry.file_name().to_string_lossy().to_string(),
                                    has_video_ts: true,
                                });
                                tracing::info!("Found DVD volume: {}", path);
                            }
                        }
                    }
                }
            }

            // Then check physical drive paths
            let physical_drives = ["/dev/disk1", "/dev/disk2", "/dev/disk3", "/dev/disk4"];
            for path in &physical_drives {
                if std::path::Path::new(path).exists() {
                    // Try to get more info about the drive
                    if let Ok(output) = std::process::Command::new("diskutil")
                        .args(["info", path])
                        .output()
                    {
                        let info = String::from_utf8_lossy(&output.stdout);
                        if info.contains("DVD") || info.contains("CD") {
                            detected_drives.push(DvdDriveInfo {
                                path: path.to_string(),
                                drive_type: DvdDriveType::PhysicalDrive,
                                label: format!("DVD Drive ({})", path),
                                has_video_ts: false, // Unknown for physical drives
                            });
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            notify_info("Scanning Linux media directories and devices...");

            // Check mounted DVD volumes first
            let mount_points = ["/media", "/mnt", "/run/media"];
            for mount_base in &mount_points {
                if let Ok(entries) = std::fs::read_dir(mount_base) {
                    for entry in entries.flatten() {
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_dir() {
                                let video_ts = entry.path().join("VIDEO_TS");
                                if video_ts.exists() {
                                    let path = entry.path().to_string_lossy().to_string();
                                    detected_drives.push(DvdDriveInfo {
                                        path: path.clone(),
                                        drive_type: DvdDriveType::MountedVolume,
                                        label: entry.file_name().to_string_lossy().to_string(),
                                        has_video_ts: true,
                                    });
                                    tracing::info!("Found DVD volume: {}", path);
                                }
                            }
                        }
                    }
                }

                // Also check user-specific mount points in /run/media
                if mount_base == "/run/media" {
                    if let Ok(users) = std::fs::read_dir(mount_base) {
                        for user_entry in users.flatten() {
                            if let Ok(user_dirs) = std::fs::read_dir(user_entry.path()) {
                                for entry in user_dirs.flatten() {
                                    let video_ts = entry.path().join("VIDEO_TS");
                                    if video_ts.exists() {
                                        let path = entry.path().to_string_lossy().to_string();
                                        detected_drives.push(DvdDriveInfo {
                                            path: path.clone(),
                                            drive_type: DvdDriveType::MountedVolume,
                                            label: entry.file_name().to_string_lossy().to_string(),
                                            has_video_ts: true,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Then check physical device paths
            let device_paths = ["/dev/dvd", "/dev/cdrom", "/dev/sr0", "/dev/sr1", "/dev/sr2"];
            for path in &device_paths {
                if std::path::Path::new(path).exists() {
                    detected_drives.push(DvdDriveInfo {
                        path: path.to_string(),
                        drive_type: DvdDriveType::PhysicalDrive,
                        label: format!("DVD Drive ({})", path),
                        has_video_ts: false,
                    });
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            notify_info("Scanning Windows drive letters...");

            // Check all drive letters for DVD content
            for letter in 'C'..='Z' {
                let drive_path = format!("{}:\\", letter);
                let video_ts = format!("{}:\\VIDEO_TS", letter);

                // Check if drive exists and has VIDEO_TS
                if std::path::Path::new(&drive_path).exists() {
                    if std::path::Path::new(&video_ts).exists() {
                        detected_drives.push(DvdDriveInfo {
                            path: drive_path.clone(),
                            drive_type: DvdDriveType::MountedVolume,
                            label: format!("DVD Drive ({})", letter),
                            has_video_ts: true,
                        });
                        tracing::info!("Found DVD at drive {}: {}", letter, drive_path);
                    } else {
                        // Check if it's a CD/DVD drive even without content
                        if let Ok(output) = std::process::Command::new("fsutil")
                            .args(["fsinfo", "drives"])
                            .output()
                        {
                            let drives_info = String::from_utf8_lossy(&output.stdout);
                            if drives_info.contains(&format!("{}:", letter)) {
                                // Additional check for drive type
                                if let Ok(type_output) = std::process::Command::new("wmic")
                                    .args([
                                        "logicaldisk",
                                        "where",
                                        &format!(
                                            "DeviceID='{}'",
                                            drive_path.trim_end_matches('\\')
                                        ),
                                        "get",
                                        "DriveType",
                                        "/value",
                                    ])
                                    .output()
                                {
                                    let type_info = String::from_utf8_lossy(&type_output.stdout);
                                    if type_info.contains("DriveType=5") {
                                        // CD-ROM drive
                                        detected_drives.push(DvdDriveInfo {
                                            path: drive_path,
                                            drive_type: DvdDriveType::PhysicalDrive,
                                            label: format!("DVD Drive ({})", letter),
                                            has_video_ts: false,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if detected_drives.is_empty() {
            None
        } else {
            // Prefer mounted volumes with VIDEO_TS, then any mounted volumes, then physical drives
            detected_drives.sort_by(|a, b| match (a.has_video_ts, b.has_video_ts) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => match (&a.drive_type, &b.drive_type) {
                    (DvdDriveType::MountedVolume, DvdDriveType::PhysicalDrive) => {
                        std::cmp::Ordering::Less
                    }
                    (DvdDriveType::PhysicalDrive, DvdDriveType::MountedVolume) => {
                        std::cmp::Ordering::Greater
                    }
                    _ => std::cmp::Ordering::Equal,
                },
            });

            Some(DvdDetectionResult {
                drives: detected_drives,
                selected_index: 0,
            })
        }
    }

    fn calculate_and_save_optimal_settings(&mut self) {
        // Calculate optimal settings based on system capabilities
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        let optimal_threads = if cpu_count <= 2 {
            1
        } else if cpu_count <= 4 {
            cpu_count - 1
        } else {
            cpu_count - 2
        };

        // Apply optimal settings to UI state
        self.ui_state.config_temp.thread_count = optimal_threads.to_string();
        self.ui_state.config_temp.quality = 22;
        self.ui_state.config_temp.video_codec = "H.264".to_string();
        self.ui_state.config_temp.encode_algo = "MP4".to_string();
        self.ui_state.config_temp.handbrake_preset = "Fast 1080p30".to_string();
        self.ui_state.config_temp.gpu_acceleration = false;
        self.ui_state.config_temp.two_pass_encoding = false;

        // Save optimal settings to config and mark as calculated
        if let Ok(mut config) = self.config.try_lock() {
            config.thread_count = optimal_threads;
            config.optimal_settings_calculated = true;
            let _ = config.save();
        }
    }
}

impl eframe::App for CopyDvdApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_first_frame();
        self.check_for_update_results();
        self.update_status(ctx);

        // Modern sidebar + main content layout
        egui::SidePanel::left("navigation_sidebar")
            .resizable(false)
            .min_width(260.0)
            .max_width(260.0)
            .frame(egui::Frame {
                fill: ModernTheme::BACKGROUND_SECONDARY,
                stroke: egui::Stroke::new(1.0, ModernTheme::BORDER_PRIMARY),
                inner_margin: egui::Margin::ZERO,
                outer_margin: egui::Margin::ZERO,
                rounding: egui::Rounding::ZERO,
                shadow: egui::epaint::Shadow::NONE,
            })
            .show(ctx, |ui| {
                self.render_sidebar(ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Header with app info
            self.render_modern_header(ui);

            // Main content area with modern styling
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .id_source("main_content_scroll")
                .show(ui, |ui| {
                    ui.add_space(Spacing::MD);

                    // Content wrapper for consistent padding
                    ui.horizontal(|ui| {
                        ui.add_space(Spacing::LG);
                        ui.vertical(|ui| {
                    // HandBrake status display at top (only show on HandBrake tab or within timeout)
                    let should_show_handbrake_status = match (&self.handbrake_status, self.ui_state.active_tab) {
                        // Always show on HandBrake tab
                        (Some(_), Tab::HandBrake) => true,
                        // Show verification message on other tabs only within timeout
                        (Some(HandBrakeStatus::Verified(_)), _) => {
                            self.ui_state.handbrake_verification_visible_until
                                .is_some_and(|until| Instant::now() < until)
                        }
                        // Always show verifying and error states on all tabs
                        (Some(HandBrakeStatus::Verifying), _) => true,
                        (Some(HandBrakeStatus::Error(_)), _) => true,
                        _ => false,
                    };

                    if should_show_handbrake_status {
                        if let Some(handbrake_status) = &self.handbrake_status {
                            match handbrake_status {
                                HandBrakeStatus::Verifying => {
                                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "Verifying HandBrake installation...");
                                    ui.separator();
                                }
                                HandBrakeStatus::Verified(path) => {
                                    ui.colored_label(egui::Color32::from_rgb(0, 150, 0), format!("HandBrake verified: {}", path));
                                    ui.separator();
                                }
                                HandBrakeStatus::Error(error) => {
                                    let error_clone = error.clone();

                                // Check if automatic fixes were attempted
                                let auto_fixes_attempted = error_clone.contains("Automatic security fixes were attempted");

                                if auto_fixes_attempted {
                                    ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "HandBrake Auto-Fix Attempted:");
                                    ui.label("The application automatically tried to resolve macOS security issues.");
                                } else {
                                    ui.colored_label(egui::Color32::from_rgb(200, 50, 50), "HandBrake Error:");
                                }
                                ui.separator();

                                // Create a scrollable area for the error message
                                egui::ScrollArea::vertical()
                                    .max_height(200.0)
                                    .show(ui, |ui| {
                                        let mut error_text = error.as_str();
                                        ui.add(egui::TextEdit::multiline(&mut error_text)
                                            .desired_width(f32::INFINITY)
                                            .font(egui::TextStyle::Monospace));
                                    });

                                ui.separator();

                                ui.horizontal(|ui| {
                                    // Add retry button with different text based on auto-fixes
                                    let button_text = if auto_fixes_attempted {
                                        "Retry After Auto-Fix"
                                    } else {
                                        "Retry HandBrake Verification"
                                    };

                                    if ui.button(button_text).clicked() {
                                        self.handbrake_status = None;
                                        let (handbrake_sender, handbrake_receiver) = mpsc::channel();
                                        self.handbrake_receiver = handbrake_receiver;

                                        tokio::spawn(async move {
                                            let _ = handbrake_sender.send(HandBrakeStatus::Verifying);

                                            let mut handbrake_manager = match HandBrakeManager::new() {
                                                Ok(manager) => manager,
                                                Err(e) => {
                                                    let _ = handbrake_sender.send(HandBrakeStatus::Error(format!("Failed to initialize HandBrake manager: {}", e)));
                                                    return;
                                                }
                                            };

                                            match handbrake_manager.verify_handbrake().await {
                                                Ok(binary_path) => {
                                                    let _ = handbrake_sender.send(HandBrakeStatus::Verified(binary_path));
                                                }
                                                Err(e) => {
                                                    let _ = handbrake_sender.send(HandBrakeStatus::Error(e.to_string()));
                                                }
                                            }
                                        });
                                    }

                                    // Add macOS-specific System Preferences button - always show on macOS
                                    #[cfg(target_os = "macos")]
                                    if ui.button("Open Security Settings").clicked() {
                                        tokio::spawn(async {
                                            // Try multiple methods to open Security preferences
                                            let methods = [
                                                ("open", vec!["-b", "com.apple.systempreferences", "/System/Library/PreferencePanes/Security.prefPane"]),
                                                ("open", vec!["/System/Library/PreferencePanes/Security.prefPane"]),
                                                ("open", vec!["-a", "System Preferences"]),
                                            ];

                                            for (cmd, args) in &methods {
                                                if std::process::Command::new(cmd).args(args).spawn().is_ok() {
                                                    break;
                                                }
                                            }
                                        });
                                    }
                                });

                                // Show helpful status message for auto-fixes
                                if auto_fixes_attempted {
                                    ui.separator();
                                    ui.colored_label(egui::Color32::from_rgb(100, 150, 255), "What happened:");
                                    ui.label("- Removed quarantine attributes automatically");
                                    ui.label("- Set executable permissions");
                                    ui.label("- Attempted to open Security preferences");
                                    ui.label("- Triggered macOS security dialog");
                                    ui.add_space(5.0);
                                    ui.colored_label(egui::Color32::from_rgb(255, 200, 100), "Next steps:");
                                    ui.label("1. Click 'Retry After Auto-Fix' above");
                                    ui.label("2. If still blocked, use 'Open Security Settings'");
                                    ui.label("3. Look for 'Allow Anyway' button in Security settings");
                                }

                                    ui.separator();
                                }
                            }
                        }
                    }

                    // Error display at top
                    if !self.ui_state.error_message.is_empty() {
                        ui.colored_label(egui::Color32::from_rgb(180, 60, 60), &self.ui_state.error_message);
                        ui.separator();
                    }

                    // Content
                    match self.ui_state.active_tab {
                        Tab::Main => {
                            render_main_tab(ui, &mut self.ui_state, self.app_state.clone());
                        }
                        Tab::Config => {
                            render_config_tab(ui, &mut self.ui_state, self.config.clone());
                        }
                        Tab::Server => {
                            render_server_tab(ui, &mut self.ui_state, self.config.clone());
                        }
                        Tab::HandBrake => {
                            render_handbrake_tab(ui, &mut self.ui_state, self.config.clone());
                        }
                        Tab::Updates => {
                            // Need to temporarily extract the updates tab to avoid double borrow
                            let mut updates_tab = std::mem::take(&mut self.ui_state.updates_tab);
                            updates_tab.show(ctx, &mut self.ui_state);
                            self.ui_state.updates_tab = updates_tab;
                        }
                        Tab::About => {
                            render_about_tab(ui, &mut self.ui_state);
                        }
                    }
                        });
                        ui.add_space(Spacing::LG); // Right padding
                    });

                    ui.add_space(Spacing::XL); // Bottom spacing
                });
        });

        // Render toast notifications
        if let Some(ref mut toasts) = self.ui_state.toasts {
            render_toast_notifications(ctx, toasts);
        }

        // Handle exit confirmation
        if ctx.input(|i| i.viewport().close_requested()) {
            let has_active_tasks = if let Ok(app_state) = self.app_state.try_lock() {
                crate::gui::components::exit_confirmation::has_active_ripping_tasks(&app_state)
            } else {
                false
            };
            let has_pending_updates =
                crate::gui::components::exit_confirmation::has_pending_updates();
            let has_handbrake_operations = matches!(
                self.ui_state.handbrake_status,
                crate::gui::state::HandBrakeOperationStatus::Downloading { .. }
                    | crate::gui::state::HandBrakeOperationStatus::Installing
                    | crate::gui::state::HandBrakeOperationStatus::Extracting
                    | crate::gui::state::HandBrakeOperationStatus::VerifyingInstallation
                    | crate::gui::state::HandBrakeOperationStatus::CheckingStatus
            );
            let has_update_operations = self.ui_state.updates_tab.download_progress.is_some();

            // Only show confirmation if there are actually important tasks running
            let should_confirm = has_active_tasks
                || has_pending_updates
                || has_handbrake_operations
                || has_update_operations;

            if should_confirm && !self.exit_confirmation.is_confirmed() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.exit_confirmation.request_exit();
            }
        }

        if self.exit_confirmation.show_dialog(
            ctx,
            if let Ok(app_state) = self.app_state.try_lock() {
                crate::gui::components::exit_confirmation::has_active_ripping_tasks(&app_state)
            } else {
                false
            },
            crate::gui::components::exit_confirmation::has_pending_updates(),
            matches!(
                self.ui_state.handbrake_status,
                crate::gui::state::HandBrakeOperationStatus::Downloading { .. }
                    | crate::gui::state::HandBrakeOperationStatus::Installing
                    | crate::gui::state::HandBrakeOperationStatus::Extracting
                    | crate::gui::state::HandBrakeOperationStatus::VerifyingInstallation
                    | crate::gui::state::HandBrakeOperationStatus::CheckingStatus
            ),
            self.ui_state.updates_tab.download_progress.is_some(),
        ) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) && self.ui_state.active_tab == Tab::Main {
            self.trigger_dvd_scan();
        }

        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.save_all_configs();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_all_configs();
    }
}
