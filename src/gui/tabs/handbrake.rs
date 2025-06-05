use crate::config::Config;
use crate::gui::icons::svg_icon;
use crate::gui::notifications::{notify_error, notify_info, notify_success};
use crate::gui::state::ui_state::send_handbrake_ui_update;
use crate::gui::state::{HandBrakeOperationStatus, UiState};
use crate::gui::theme::{
    full_width_button, grouped_section, status_indicator, styled_panel, BasicTheme, Layout,
    StatusType,
};
use std::sync::{Arc, Mutex};

pub fn render_handbrake_tab(
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    _config: Arc<Mutex<Config>>,
) {
    ui.heading("HandBrake Management");
    ui.separator();

    // HandBrake Status Section
    render_handbrake_status(ui, ui_state);

    ui.add_space(Layout::SPACING_LARGE);

    // Management Settings Section
    render_management_settings(ui, ui_state);

    ui.add_space(Layout::SPACING_LARGE);

    // Download Settings Section
    render_download_settings(ui, ui_state);

    ui.add_space(Layout::SPACING_LARGE);

    // Cache Management Section
    render_cache_management(ui, ui_state);

    // Cache clear confirmation dialog (outside scroll area for proper positioning)
    if ui_state.show_cache_clear_dialog {
        egui::Window::new("Clear Cache")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("Are you sure you want to clear the HandBrake cache?");
                ui.label("This will remove all downloaded HandBrake files.");

                ui.add_space(Layout::SPACING);

                if ui.button("Yes, Clear Cache").clicked() {
                    clear_handbrake_cache(ui_state);
                    ui_state.show_cache_clear_dialog = false;
                }

                ui.add_space(Layout::SPACING_SMALL);

                if ui.button("Cancel").clicked() {
                    ui_state.show_cache_clear_dialog = false;
                }
            });
    }
}

fn render_handbrake_status(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "HandBrake Status", |ui| {
            // Always display version section prominently at the top
            ui.horizontal(|ui| {
                ui.label("Version:");
                if let Some(version) = &ui_state.handbrake_version {
                    ui.label(
                        egui::RichText::new(version)
                            .strong()
                            .color(egui::Color32::from_rgb(0, 150, 0)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("Not detected")
                            .color(egui::Color32::from_rgb(150, 150, 150)),
                    );
                }
            });
            ui.add_space(Layout::SPACING_SMALL);

            match &ui_state.handbrake_status {
                HandBrakeOperationStatus::Idle => {
                    status_indicator(ui, "Ready", StatusType::Info);
                }
                HandBrakeOperationStatus::CheckingStatus => {
                    status_indicator(ui, "Checking status...", StatusType::Warning);
                }
                HandBrakeOperationStatus::Downloading { progress } => {
                    status_indicator(ui, "Downloading...", StatusType::Warning);

                    ui.add_space(Layout::SPACING_SMALL);

                    // Simple progress bar
                    let progress_rect = ui
                        .allocate_space(egui::Vec2::new(ui.available_width(), 20.0))
                        .1;
                    ui.painter().rect_filled(
                        progress_rect,
                        egui::Rounding::same(Layout::ROUNDING),
                        BasicTheme::SURFACE,
                    );

                    let fill_width = progress_rect.width() * progress;
                    let fill_rect = egui::Rect::from_min_size(
                        progress_rect.min,
                        egui::Vec2::new(fill_width, progress_rect.height()),
                    );
                    ui.painter().rect_filled(
                        fill_rect,
                        egui::Rounding::same(Layout::ROUNDING),
                        BasicTheme::ACCENT,
                    );

                    ui.label(format!("{:.1}%", progress * 100.0));
                }
                HandBrakeOperationStatus::Extracting => {
                    status_indicator(ui, "Extracting...", StatusType::Warning);
                }
                HandBrakeOperationStatus::Installing => {
                    status_indicator(ui, "Installing...", StatusType::Warning);
                }
                HandBrakeOperationStatus::VerifyingInstallation => {
                    status_indicator(ui, "Verifying installation...", StatusType::Warning);
                }
                HandBrakeOperationStatus::ClearingCache => {
                    status_indicator(ui, "Clearing cache...", StatusType::Warning);
                }
                HandBrakeOperationStatus::Error(err) => {
                    status_indicator(ui, &format!("Error: {}", err), StatusType::Error);
                }
            }

            ui.add_space(Layout::SPACING);

            ui.horizontal(|ui| {
                svg_icon(ui, "search", 16.0, egui::Color32::from_rgb(100, 150, 255));
                if full_width_button(ui, "Check Status").clicked() {
                    check_handbrake_availability(ui_state);
                }
            });

            ui.add_space(Layout::SPACING_SMALL);

            ui.horizontal(|ui| {
                svg_icon(ui, "download", 16.0, egui::Color32::from_rgb(100, 200, 100));
                if full_width_button(ui, "Download HandBrake").clicked() {
                    download_handbrake(ui_state);
                }
            });

            ui.add_space(Layout::SPACING_SMALL);

            ui.horizontal(|ui| {
                svg_icon(ui, "check", 16.0, egui::Color32::from_rgb(100, 200, 100));
                if full_width_button(ui, "Verify Installation").clicked() {
                    verify_handbrake(ui_state);
                }
            });
        });
    });
}

fn render_management_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Management Settings", |ui| {
            ui.checkbox(
                &mut ui_state.config_temp.auto_download,
                "Auto-download HandBrake if not found",
            );
            ui.checkbox(
                &mut ui_state.config_temp.prefer_system,
                "Use system HandBrake if available",
            );
            ui.checkbox(
                &mut ui_state.config_temp.verify_on_startup,
                "Verify HandBrake on startup",
            );

            ui.add_space(Layout::SPACING);

            ui.label("HandBrake executable path:");

            ui.horizontal(|ui| {
                ui.add_sized(
                    [200.0, 20.0],
                    egui::TextEdit::singleline(&mut ui_state.config_temp.handbrake_path),
                );

                if ui.button("Browse").clicked() {
                    browse_for_handbrake(ui_state);
                }
            });
        });
    });
}

fn render_download_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Download Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Max cache size (MB):");
                ui.add_sized(
                    [100.0, 20.0],
                    egui::TextEdit::singleline(&mut ui_state.config_temp.max_cache_size_mb),
                );
            });

            if let Some((cache_dir, cache_size)) = &ui_state.config_temp.cache_info {
                ui.add_space(Layout::SPACING);
                ui.label(format!("Current cache: {}", cache_size));
                ui.label(format!("Location: {}", cache_dir));
            }

            ui.add_space(Layout::SPACING);

            if full_width_button(ui, "Refresh Cache Info").clicked() {
                refresh_cache_info(ui_state);
            }

            ui.add_space(Layout::SPACING_SMALL);

            if full_width_button(ui, "Open HandBrake Cache Folder").clicked() {
                open_cache_folder(ui_state);
            }
        });
    });
}

fn render_cache_management(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Cache Management", |ui| {
            ui.label("Manage HandBrake cache and downloads");

            ui.add_space(Layout::SPACING);

            if full_width_button(ui, "Clear Cache").clicked() {
                ui_state.show_cache_clear_dialog = true;
            }
        });
    });
}

fn check_handbrake_availability(ui_state: &mut UiState) {
    ui_state.handbrake_status = HandBrakeOperationStatus::CheckingStatus;
    ui_state.handbrake_version = None;

    // Spawn async task to verify HandBrake
    let manager = ui_state.handbrake_manager.clone();
    tokio::spawn(async move {
        // Send UI update for checking status
        send_handbrake_ui_update(HandBrakeOperationStatus::CheckingStatus, None);
        // Send OS notification for checking status
        notify_info("Checking HandBrake status...");

        let result = {
            let mut guard = manager.lock().await;
            guard.verify_handbrake().await
        };

        match result {
            Ok(_) => {
                // Get version information
                let version = {
                    let guard = manager.lock().await;
                    guard.get_version()
                };

                // Send UI update for success
                send_handbrake_ui_update(HandBrakeOperationStatus::Idle, version.clone());

                // Send OS notification for success
                if let Some(version) = &version {
                    notify_success(&format!("HandBrake {} ready", version));
                } else {
                    notify_success("HandBrake ready");
                }
            }
            Err(e) => {
                let error_msg = format!("HandBrake check failed: {}", e);

                // Send UI update for error
                send_handbrake_ui_update(HandBrakeOperationStatus::Error(error_msg.clone()), None);

                // Send OS notification for error
                notify_error(&error_msg);
            }
        }
    });
}

fn download_handbrake(ui_state: &mut UiState) {
    // Reset status if it's stuck
    if matches!(
        ui_state.handbrake_status,
        HandBrakeOperationStatus::CheckingStatus
    ) {
        ui_state.handbrake_status = HandBrakeOperationStatus::Idle;
        return;
    }

    ui_state.handbrake_status = HandBrakeOperationStatus::CheckingStatus;
    ui_state.handbrake_version = None;

    // Spawn async task to download HandBrake
    let manager = ui_state.handbrake_manager.clone();
    tokio::spawn(async move {
        // Send UI update for checking status
        send_handbrake_ui_update(HandBrakeOperationStatus::CheckingStatus, None);
        // Send OS notification for checking status
        notify_info("Checking HandBrake status...");

        // First check if HandBrake already exists
        let exists = {
            let guard = manager.lock().await;
            guard.handbrake_exists()
        };

        if exists {
            let result = {
                let mut guard = manager.lock().await;
                guard.verify_handbrake().await
            };

            match result {
                Ok(_) => {
                    // Get version information after verification
                    let version = {
                        let guard = manager.lock().await;
                        guard.get_version()
                    };

                    // Send UI update for success
                    send_handbrake_ui_update(HandBrakeOperationStatus::Idle, version.clone());

                    // Send OS notification for success
                    if let Some(v) = &version {
                        notify_success(&format!("HandBrake {} ready", v));
                    } else {
                        notify_success("HandBrake ready");
                    }
                }
                Err(_e) => {
                    // If verification fails, proceed with download
                    // Send UI update for downloading
                    send_handbrake_ui_update(
                        HandBrakeOperationStatus::Downloading { progress: 0.0 },
                        None,
                    );
                    // Send OS notification for downloading
                    notify_info("Downloading HandBrake...");

                    let download_result = {
                        let mut guard = manager.lock().await;
                        guard.get_handbrake_path().await
                    };
                    match download_result {
                        Ok(_) => {
                            // Get version information after download
                            let version = {
                                let guard = manager.lock().await;
                                guard.get_version()
                            };

                            // Send UI update for success
                            send_handbrake_ui_update(
                                HandBrakeOperationStatus::Idle,
                                version.clone(),
                            );

                            // Send OS notification for success
                            if let Some(v) = &version {
                                notify_success(&format!("HandBrake {} ready", v));
                            } else {
                                notify_success("HandBrake ready");
                            }
                        }
                        Err(e) => {
                            let error_msg = format!("HandBrake download failed: {}", e);
                            // Send UI update for error
                            send_handbrake_ui_update(
                                HandBrakeOperationStatus::Error(error_msg.clone()),
                                None,
                            );
                            // Send OS notification for error
                            notify_error(&error_msg);
                        }
                    }
                }
            }
        } else {
            // Send UI update for downloading
            send_handbrake_ui_update(
                HandBrakeOperationStatus::Downloading { progress: 0.0 },
                None,
            );
            // Send OS notification for downloading
            notify_info("Downloading HandBrake...");

            let result = {
                let mut guard = manager.lock().await;
                guard.get_handbrake_path().await
            };

            match result {
                Ok(_) => {
                    // Get version information after download
                    let version = {
                        let guard = manager.lock().await;
                        guard.get_version()
                    };

                    // Send UI update for success
                    send_handbrake_ui_update(HandBrakeOperationStatus::Idle, version.clone());

                    // Send OS notification for success
                    if let Some(v) = version {
                        notify_success(&format!("HandBrake {} ready", v));
                    } else {
                        notify_success("HandBrake ready");
                    }
                }
                Err(e) => {
                    let error_msg = format!("HandBrake download failed: {}", e);
                    // Send UI update for error
                    send_handbrake_ui_update(
                        HandBrakeOperationStatus::Error(error_msg.clone()),
                        None,
                    );
                    // Send OS notification for error
                    notify_error(&error_msg);
                }
            }
        }
    });
}

fn verify_handbrake(ui_state: &mut UiState) {
    ui_state.handbrake_status = HandBrakeOperationStatus::VerifyingInstallation;
    ui_state.handbrake_version = None;

    // Spawn async task to verify HandBrake
    let manager = ui_state.handbrake_manager.clone();
    tokio::spawn(async move {
        // Send UI update for verifying status
        send_handbrake_ui_update(HandBrakeOperationStatus::VerifyingInstallation, None);
        // Send OS notification for verifying status
        notify_info("Verifying HandBrake installation...");

        let result = {
            let mut guard = manager.lock().await;
            guard.verify_handbrake().await
        };

        match result {
            Ok(_path) => {
                // Get version information
                let version = {
                    let guard = manager.lock().await;
                    guard.get_version()
                };

                // Send UI update for success
                send_handbrake_ui_update(HandBrakeOperationStatus::Idle, version.clone());

                // Send OS notification for success
                if let Some(v) = &version {
                    notify_success(&format!("HandBrake {} verified", v));
                } else {
                    notify_success("HandBrake verified successfully");
                }
            }
            Err(e) => {
                let error_msg = format!("HandBrake verification failed: {}", e);
                // Send UI update for error
                send_handbrake_ui_update(HandBrakeOperationStatus::Error(error_msg.clone()), None);
                // Send OS notification for error
                notify_error(&error_msg);
            }
        }
    });
}

fn browse_for_handbrake(ui_state: &mut UiState) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Select HandBrake Executable")
        .pick_file()
    {
        ui_state.config_temp.handbrake_path = path.to_string_lossy().to_string();
    }
}

fn refresh_cache_info(ui_state: &mut UiState) {
    let manager = ui_state.handbrake_manager.clone();
    tokio::spawn(async move {
        let result = {
            let guard = manager.lock().await;
            guard.get_cache_info()
        };

        match result {
            Ok((_cache_dir, size)) => {
                let size_mb = size as f64 / 1024.0 / 1024.0;
                notify_success(&format!("Cache info refreshed: {:.1} MB", size_mb));
            }
            Err(e) => notify_error(&format!("Failed to get cache info: {}", e)),
        }
    });
}

fn clear_handbrake_cache(ui_state: &mut UiState) {
    ui_state.handbrake_status = HandBrakeOperationStatus::ClearingCache;

    let manager = ui_state.handbrake_manager.clone();
    tokio::spawn(async move {
        let result = {
            let guard = manager.lock().await;
            guard.clear_cache()
        };

        match result {
            Ok(()) => notify_success("HandBrake cache cleared successfully"),
            Err(e) => notify_error(&format!("Failed to clear cache: {}", e)),
        }
    });
}

fn open_cache_folder(ui_state: &mut UiState) {
    let manager = ui_state.handbrake_manager.clone();
    tokio::spawn(async move {
        let result = {
            let guard = manager.lock().await;
            guard.get_cache_info()
        };

        match result {
            Ok((cache_dir, _)) => {
                let _ = open::that(cache_dir);
            }
            Err(e) => notify_error(&format!("Failed to open cache folder: {}", e)),
        }
    });
}
