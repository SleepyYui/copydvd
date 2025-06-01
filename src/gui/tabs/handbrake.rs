use crate::config::Config;
use crate::gui::state::{UiState, HandBrakeOperationStatus};
use crate::gui::theme::{BasicTheme, Layout, styled_panel, grouped_section, full_width_button, status_indicator, StatusType};
use crate::gui::notifications::{notify_success, notify_error, notify_info};
use std::sync::{Arc, Mutex};

pub fn render_handbrake_tab(ui: &mut egui::Ui, ui_state: &mut UiState, config: Arc<Mutex<Config>>) {
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
                    let progress_rect = ui.allocate_space(egui::Vec2::new(ui.available_width(), 20.0)).1;
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
            
            if full_width_button(ui, "Check Status").clicked() {
                check_handbrake_availability(ui_state);
            }
            
            ui.add_space(Layout::SPACING_SMALL);
            
            if full_width_button(ui, "Download HandBrake").clicked() {
                download_handbrake(ui_state);
            }
            
            ui.add_space(Layout::SPACING_SMALL);
            
            if full_width_button(ui, "Verify Installation").clicked() {
                verify_handbrake(ui_state);
            }
        });
    });
}

fn render_management_settings(ui: &mut egui::Ui, ui_state: &mut UiState) {
    styled_panel(ui, |ui| {
        grouped_section(ui, "Management Settings", |ui| {
            ui.checkbox(&mut ui_state.config_temp.auto_download, "Auto-download HandBrake if not found");
            ui.checkbox(&mut ui_state.config_temp.prefer_system, "Use system HandBrake if available");
            ui.checkbox(&mut ui_state.config_temp.verify_on_startup, "Verify HandBrake on startup");
            
            ui.add_space(Layout::SPACING);
            
            ui.label("HandBrake executable path:");
            
            ui.horizontal(|ui| {
                ui.add_sized([200.0, 20.0], egui::TextEdit::singleline(&mut ui_state.config_temp.handbrake_path));
                
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
                ui.add_sized([100.0, 20.0], egui::TextEdit::singleline(&mut ui_state.config_temp.max_cache_size_mb));
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
    notify_info("Checking HandBrake availability...");
    
    // Spawn async task to verify HandBrake
    let manager = ui_state.handbrake_manager.clone();
    tokio::spawn(async move {
        let result = {
            let mut guard = manager.lock().await;
            guard.verify_handbrake().await
        };
        
        match result {
            Ok(_) => notify_success("HandBrake is available and ready"),
            Err(e) => notify_error(&format!("HandBrake check failed: {}", e)),
        }
    });
}

fn download_handbrake(ui_state: &mut UiState) {
    // Reset status if it's stuck
    if matches!(ui_state.handbrake_status, HandBrakeOperationStatus::CheckingStatus) {
        ui_state.handbrake_status = HandBrakeOperationStatus::Idle;
        notify_info("Reset status and retrying...");
        return;
    }
    
    ui_state.handbrake_status = HandBrakeOperationStatus::CheckingStatus;
    notify_info("Checking HandBrake status...");
    
    // Spawn async task to download HandBrake
    let manager = ui_state.handbrake_manager.clone();
    tokio::spawn(async move {
        // First check if HandBrake already exists
        let exists = {
            let guard = manager.lock().await;
            guard.handbrake_exists()
        };
        
        if exists {
            notify_info("HandBrake already exists, verifying installation...");
            let result = {
                let mut guard = manager.lock().await;
                guard.verify_handbrake().await
            };
            
            match result {
                Ok(_) => {
                    notify_success("HandBrake is already installed and verified");
                }
                Err(_e) => {
                    notify_info("Existing HandBrake has issues, re-downloading...");
                    // If verification fails, proceed with download
                    let download_result = {
                        let mut guard = manager.lock().await;
                        guard.get_handbrake_path().await
                    };
                    match download_result {
                        Ok(_) => notify_success("HandBrake downloaded and installed successfully"),
                        Err(e) => notify_error(&format!("HandBrake download failed: {}", e)),
                    }
                }
            }
        } else {
            notify_info("HandBrake not found, starting download...");
            let result = {
                let mut guard = manager.lock().await;
                guard.get_handbrake_path().await
            };
            
            match result {
                Ok(_) => notify_success("HandBrake downloaded and installed successfully"),
                Err(e) => notify_error(&format!("HandBrake download failed: {}", e)),
            }
        }
    });
}

fn verify_handbrake(ui_state: &mut UiState) {
    ui_state.handbrake_status = HandBrakeOperationStatus::VerifyingInstallation;
    notify_info("Verifying HandBrake installation...");
    
    // Spawn async task to verify HandBrake
    let manager = ui_state.handbrake_manager.clone();
    tokio::spawn(async move {
        let result = {
            let mut guard = manager.lock().await;
            guard.verify_handbrake().await
        };
        
        match result {
            Ok(path) => notify_success(&format!("HandBrake verified at: {}", path)),
            Err(e) => notify_error(&format!("HandBrake verification failed: {}", e)),
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
            Ok((cache_dir, size)) => {
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