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
    
    // Wrap content in scroll area to prevent overflow
    egui::ScrollArea::both().show(ui, |ui| {
        // Main workflow section
        render_workflow_section(ui, ui_state, app_state.clone());
        
        ui.add_space(Layout::SPACING_LARGE);
        
        // Title selection if available
        if !ui_state.titles.is_empty() {
            render_title_selection(ui, ui_state);
        }
    });
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
                if let Ok(mut _state) = app_state.try_lock() {
                    // TODO: Implement DVD scanning
                    notify_info("DVD scan started");
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
                notify_success(&format!("Starting to rip {} titles", selected_count));
                // TODO: Start ripping process
            }
        });
    });
}