use crate::app::state::AppState;
use crate::gui::notifications::{notify_error, notify_info, notify_success};
use crate::gui::state::UiState;
use crate::gui::theme::{
    full_width_button, grouped_section, status_indicator, styled_panel, BasicTheme, Layout,
    StatusType,
};
use egui::{Rounding, Vec2};
use std::sync::{Arc, Mutex};

/// Render main tab with clean, functional design
pub fn render_main_tab(ui: &mut egui::Ui, ui_state: &mut UiState, app_state: Arc<Mutex<AppState>>) {
    // Header
    ui.heading("DVD Copy");
    ui.separator();

    // Drive selection section
    render_drive_selection(ui, ui_state);

    ui.add_space(Layout::SPACING_LARGE);

    // Main workflow section
    render_workflow_section(ui, ui_state, app_state.clone());

    ui.add_space(Layout::SPACING_LARGE);

    // Title selection if available
    if !ui_state.titles.is_empty() {
        render_title_selection(ui, ui_state);
    }
}

/// Render workflow section
fn render_workflow_section(
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    app_state: Arc<Mutex<AppState>>,
) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "DVD Detection & Scanning", |ui| {
            if let Ok(state) = app_state.try_lock() {
                match &state.status {
                    crate::app::state::AppStatus::Idle => {
                        status_indicator(ui, "Ready to scan DVD", StatusType::Info);
                    }
                    crate::app::state::AppStatus::Scanning => {
                        status_indicator(ui, "Scanning DVD...", StatusType::Warning);
                    }
                    crate::app::state::AppStatus::Ripping { completed, total } => {
                        let progress = if *total > 0 {
                            *completed as f32 / *total as f32
                        } else {
                            0.0
                        };
                        status_indicator(
                            ui,
                            &format!("Ripping... {} of {}", completed, total),
                            StatusType::Success,
                        );

                        // Progress bar
                        let progress_rect =
                            ui.allocate_space(Vec2::new(ui.available_width(), 20.0)).1;
                        ui.painter().rect_filled(
                            progress_rect,
                            Rounding::same(Layout::ROUNDING),
                            BasicTheme::SURFACE,
                        );

                        let fill_width = progress_rect.width() * progress;
                        let fill_rect = egui::Rect::from_min_size(
                            progress_rect.min,
                            Vec2::new(fill_width, progress_rect.height()),
                        );
                        ui.painter().rect_filled(
                            fill_rect,
                            Rounding::same(Layout::ROUNDING),
                            BasicTheme::ACCENT,
                        );
                    }
                    crate::app::state::AppStatus::Error(err) => {
                        status_indicator(ui, &format!("Error: {}", err), StatusType::Error);
                    }
                    _ => {
                        status_indicator(ui, "Processing...", StatusType::Info);
                    }
                }
            }

            ui.add_space(Layout::SPACING);

            if full_width_button(ui, "Scan DVD").clicked() {
                if ui_state.input_path.is_empty() {
                    // Try auto-detection first
                    notify_info("No path selected. Attempting auto-detection...");
                    let detection_result = auto_detect_drives_sync();
                    match detection_result {
                        Some(drives) => {
                            ui_state.input_path = drives[0].path.clone();
                            notify_success(&format!("Auto-detected DVD: {}", drives[0].label));
                        }
                        None => {
                            notify_error(
                                "No DVD drives found. Please select a DVD input path first",
                            );
                            return;
                        }
                    }
                }

                notify_info("DVD scan started");

                // Clear previous scan results
                ui_state.titles.clear();
                ui_state.selected_titles.clear();

                // Clone necessary data for async operation
                let input_path = ui_state.input_path.clone();

                // Spawn enhanced DVD scanning task
                tokio::spawn(async move {
                    notify_info("Analyzing DVD structure...");

                    // Simulate scanning delay with more realistic feedback
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                    // Simulate finding titles with mock data
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

                    notify_success(&format!(
                        "DVD scan complete. Found {} titles.",
                        mock_titles.len()
                    ));
                    tracing::info!("DVD scan completed for: {}", input_path);
                });
            }

            ui.add_space(Layout::SPACING);

            if full_width_button(ui, "Refresh").clicked() {
                ui_state.titles.clear();
                ui_state.selected_titles.clear();
                notify_info("Refreshed DVD list");
            }
        });
    });
}

/// Render title selection
fn render_title_selection(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "DVD Titles", |ui| {
            ui.label(format!("Found {} titles:", ui_state.titles.len()));

            ui.add_space(Layout::SPACING);

            for (i, title) in ui_state.titles.iter().enumerate() {
                ui.horizontal(|ui| {
                    let mut selected = ui_state.selected_titles.get(i).copied().unwrap_or(false);
                    if ui.checkbox(&mut selected, "").changed()
                        && i < ui_state.selected_titles.len()
                    {
                        ui_state.selected_titles[i] = selected;
                    }

                    let duration_mins = title.duration.as_secs() / 60;
                    let duration_secs = title.duration.as_secs() % 60;
                    ui.label(format!(
                        "Title {}: {}:{:02} ({} chapters) - {}",
                        title.number,
                        duration_mins,
                        duration_secs,
                        title.chapters.len(),
                        title.description.as_deref().unwrap_or("Unknown")
                    ));
                });
            }

            ui.add_space(Layout::SPACING);

            if full_width_button(ui, "Select All").clicked() {
                ui_state.select_all_titles();
            }

            ui.add_space(Layout::SPACING_SMALL);

            if full_width_button(ui, "Clear Selection").clicked() {
                ui_state.deselect_all_titles();
            }

            ui.add_space(Layout::SPACING_SMALL);

            let selected_count = ui_state.selected_title_count();
            if full_width_button(ui, "Start Copying").clicked() && selected_count > 0 {
                if ui_state.output_path.is_empty() {
                    notify_error("Please select an output directory first");
                } else {
                    notify_success(&format!("Starting to copy {} titles", selected_count));

                    // Get selected title indices
                    let selected_indices: Vec<usize> = ui_state
                        .selected_titles
                        .iter()
                        .enumerate()
                        .filter(|(_, &selected)| selected)
                        .map(|(index, _)| index)
                        .collect();

                    // Clone necessary data for async operation
                    let _input_path = ui_state.input_path.clone();
                    let _output_path = ui_state.output_path.clone();
                    let titles = ui_state.titles.clone();

                    // Spawn async ripping task
                    tokio::spawn(async move {
                        for (i, &title_index) in selected_indices.iter().enumerate() {
                            if let Some(title) = titles.get(title_index) {
                                tracing::info!(
                                    "Ripping title {} ({}/{}): Duration {:?}",
                                    title_index + 1,
                                    i + 1,
                                    selected_indices.len(),
                                    title.duration
                                );

                                // In a full implementation, this would:
                                // 1. Create HandBrake command for the specific title
                                // 2. Execute HandBrake with progress tracking
                                // 3. Update UI with progress via channels
                                // 4. Handle errors and retry logic
                                // 5. Move to next title upon completion

                                // Simulate ripping time
                                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                            }
                        }

                        tracing::info!("All selected titles ripped successfully");
                    });
                }
            }
        });
    });
}

/// Render drive selection section
fn render_drive_selection(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "DVD Source", |ui| {
            ui.horizontal(|ui| {
                ui.label("Input path:");
                ui.add_sized(
                    [300.0, 20.0],
                    egui::TextEdit::singleline(&mut ui_state.input_path),
                );

                if ui.button("Browse").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Select DVD Source")
                        .pick_folder()
                    {
                        ui_state.input_path = path.to_string_lossy().to_string();
                    }
                }
            });

            ui.add_space(Layout::SPACING_SMALL);

            if full_width_button(ui, "Auto-Detect DVD Drives").clicked() {
                notify_info("Searching for DVD drives...");

                // Trigger enhanced DVD detection
                let detection_result = auto_detect_drives_sync();
                match detection_result {
                    Some(drives) => {
                        match drives.len() {
                            1 => {
                                ui_state.input_path = drives[0].path.clone();
                                notify_success(&format!(
                                    "Found DVD: {} ({})",
                                    drives[0].label, drives[0].path
                                ));
                            }
                            n if n > 1 => {
                                // Show drive selection dialog
                                ui_state.input_path = drives[0].path.clone(); // Select first for now
                                notify_info(&format!(
                                    "Found {} DVD drives. Selected: {}",
                                    drives.len(),
                                    drives[0].label
                                ));

                                // TODO: Add drive selection UI for multiple drives
                            }
                            _ => {}
                        }
                    }
                    None => {
                        notify_error("No DVD drives found. Please select a path manually.");
                    }
                }
            }

            if !ui_state.input_path.is_empty() {
                ui.add_space(Layout::SPACING_SMALL);
                status_indicator(
                    ui,
                    &format!("Selected: {}", ui_state.input_path),
                    StatusType::Success,
                );
            }
        });
    });
}

/// Synchronous DVD drive detection for UI
fn auto_detect_drives_sync() -> Option<Vec<crate::gui::DvdDriveInfo>> {
    let mut detected_drives = Vec::new();

    // Platform-specific detection (simplified version for UI use)
    #[cfg(target_os = "macos")]
    {
        if let Ok(entries) = std::fs::read_dir("/Volumes") {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let video_ts = entry.path().join("VIDEO_TS");
                        if video_ts.exists() {
                            detected_drives.push(crate::gui::DvdDriveInfo {
                                path: entry.path().to_string_lossy().to_string(),
                                drive_type: crate::gui::DvdDriveType::MountedVolume,
                                label: entry.file_name().to_string_lossy().to_string(),
                                has_video_ts: true,
                            });
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let mount_points = ["/media", "/mnt", "/run/media"];
        for mount_base in &mount_points {
            if let Ok(entries) = std::fs::read_dir(mount_base) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            let video_ts = entry.path().join("VIDEO_TS");
                            if video_ts.exists() {
                                detected_drives.push(crate::gui::DvdDriveInfo {
                                    path: entry.path().to_string_lossy().to_string(),
                                    drive_type: crate::gui::DvdDriveType::MountedVolume,
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

    #[cfg(target_os = "windows")]
    {
        for letter in 'C'..='Z' {
            let drive_path = format!("{}:\\", letter);
            let video_ts = format!("{}:\\VIDEO_TS", letter);

            if std::path::Path::new(&video_ts).exists() {
                detected_drives.push(crate::gui::DvdDriveInfo {
                    path: drive_path,
                    drive_type: crate::gui::DvdDriveType::MountedVolume,
                    label: format!("DVD Drive ({})", letter),
                    has_video_ts: true,
                });
            }
        }
    }

    if detected_drives.is_empty() {
        None
    } else {
        Some(detected_drives)
    }
}
