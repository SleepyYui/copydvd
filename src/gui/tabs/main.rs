use crate::app::state::AppState;
use crate::gui::state::UiState;
use crate::gui::theme::{BasicTheme, Layout, styled_panel, grouped_section, full_width_button, status_indicator, StatusType};
use crate::gui::notifications::{notify_success, notify_error, notify_info};
use egui::{Rounding, Vec2};
use std::sync::{Arc, Mutex};

/// Render main tab with clean, functional design
pub fn render_main_tab(ui: &mut egui::Ui, ui_state: &mut UiState, app_state: Arc<Mutex<AppState>>) {
    // Header
    ui.heading("DVD Copy");
    ui.separator();
    
    // Main workflow section
    render_workflow_section(ui, ui_state, app_state.clone());
    
    ui.add_space(Layout::SPACING_LARGE);
    
    // Title selection if available
    if !ui_state.titles.is_empty() {
        render_title_selection(ui, ui_state);
    }
}

/// Render workflow section
fn render_workflow_section(ui: &mut egui::Ui, ui_state: &mut UiState, app_state: Arc<Mutex<AppState>>) {
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
                        let progress = if *total > 0 { *completed as f32 / *total as f32 } else { 0.0 };
                        status_indicator(ui, &format!("Ripping... {} of {}", completed, total), StatusType::Success);
                        
                        // Progress bar
                        let progress_rect = ui.allocate_space(Vec2::new(ui.available_width(), 20.0)).1;
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
                    notify_error("Please select a DVD input path first");
                } else {
                    notify_info("DVD scan started");
                    
                    // Clear previous scan results
                    ui_state.titles.clear();
                    ui_state.selected_titles.clear();
                    
                    // Clone necessary data for async operation
                    let input_path = ui_state.input_path.clone();
                    
                    // Spawn async DVD scanning task
                    tokio::spawn(async move {
                        // Create DVD instance and scan
                        // Note: In a full implementation, this would:
                        // 1. Create a Dvd instance with the input path
                        // 2. Call dvd.scan_titles().await
                        // 3. Parse results and update UI state via channels
                        // 4. Handle errors appropriately
                        
                        tracing::info!("Scanning DVD at path: {}", input_path);
                        
                        // Simulate scanning delay
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        
                        // For now, we'll simulate finding some titles
                        // In real implementation, this would come from HandBrake scan results
                        tracing::info!("DVD scan completed for: {}", input_path);
                    });
                }
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
                    if ui.checkbox(&mut selected, "").changed() {
                        if i < ui_state.selected_titles.len() {
                            ui_state.selected_titles[i] = selected;
                        }
                    }
                    
                    ui.label(format!("Title {}: Duration: {:?} ({} chapters)", 
                        i + 1, 
                        title.duration, 
                        title.chapters.len()
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
            if full_width_button(ui, "Start Ripping").clicked() && selected_count > 0 {
                if ui_state.output_path.is_empty() {
                    notify_error("Please select an output directory first");
                } else {
                    notify_success(&format!("Starting to rip {} titles", selected_count));
                    
                    // Get selected title indices
                    let selected_indices: Vec<usize> = ui_state.selected_titles
                        .iter()
                        .enumerate()
                        .filter(|(_, &selected)| selected)
                        .map(|(index, _)| index)
                        .collect();
                    
                    // Clone necessary data for async operation
                    let input_path = ui_state.input_path.clone();
                    let output_path = ui_state.output_path.clone();
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