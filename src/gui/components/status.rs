use crate::app::state::AppStatus;
use crate::gui::theme::{ModernTheme, StyleConstants, glass_card, neon_progress_bar, status_indicator};
use egui::{Rounding, Vec2};

/// Display error message with modern styling
pub fn error_display(ui: &mut egui::Ui, error_message: &mut String) {
    if !error_message.is_empty() {
        let error_text = error_message.clone();
        let mut should_clear = false;
        
        glass_card(ui, true, |ui| {
            ui.horizontal(|ui| {
                // Error icon
                ui.painter().circle_filled(
                    ui.next_widget_position() + Vec2::new(8.0, 8.0),
                    6.0,
                    ModernTheme::ERROR,
                );
                ui.add_space(20.0);
                
                ui.vertical(|ui| {
                    ui.colored_label(ModernTheme::ERROR, "Error");
                    ui.colored_label(ModernTheme::TEXT_PRIMARY, &error_text);
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(
                        egui::Button::new("✕")
                            .fill(ModernTheme::ERROR)
                            .rounding(Rounding::same(StyleConstants::ROUNDING_FULL))
                    ).clicked() {
                        should_clear = true;
                    }
                });
            });
        });
        ui.add_space(StyleConstants::SPACING_MD);
        
        if should_clear {
            error_message.clear();
        }
    }
}

/// Display info message with glassmorphism
pub fn info_display(ui: &mut egui::Ui, message: &str) {
    glass_card(ui, false, |ui| {
        ui.horizontal(|ui| {
            status_indicator(ui, "Info", ModernTheme::INFO, false);
            ui.add_space(StyleConstants::SPACING_MD);
            ui.colored_label(ModernTheme::TEXT_PRIMARY, message);
        });
    });
}

/// Display success message with neon effects
pub fn success_display(ui: &mut egui::Ui, message: &str) {
    glass_card(ui, true, |ui| {
        ui.horizontal(|ui| {
            status_indicator(ui, "Success", ModernTheme::SUCCESS, true);
            ui.add_space(StyleConstants::SPACING_MD);
            ui.colored_label(ModernTheme::TEXT_PRIMARY, message);
        });
    });
}

/// Display warning message
pub fn warning_display(ui: &mut egui::Ui, message: &str) {
    glass_card(ui, true, |ui| {
        ui.horizontal(|ui| {
            status_indicator(ui, "Warning", ModernTheme::WARNING, true);
            ui.add_space(StyleConstants::SPACING_MD);
            ui.colored_label(ModernTheme::TEXT_PRIMARY, message);
        });
    });
}

/// Status bar for current operations
pub fn status_bar(ui: &mut egui::Ui, status: &AppStatus, message: &str) {
    let (color, text, glow) = match status {
        AppStatus::Idle => (ModernTheme::TEXT_MUTED, "Ready", false),
        AppStatus::Scanning => (ModernTheme::NEON_BLUE, "Scanning", true),
        AppStatus::ScanComplete(_) => (ModernTheme::NEON_GREEN, "Scan Complete", true),
        AppStatus::Ripping { .. } => (ModernTheme::NEON_PURPLE, "Processing", true),
        AppStatus::RipComplete => (ModernTheme::NEON_GREEN, "Complete", true),
        AppStatus::Uploading { .. } => (ModernTheme::NEON_PINK, "Uploading", true),
        AppStatus::UploadComplete => (ModernTheme::NEON_GREEN, "Upload Complete", true),
        AppStatus::Completed => (ModernTheme::NEON_GREEN, "All Complete", true),
        AppStatus::Error(_) => (ModernTheme::ERROR, "Error", true),
        _ => (ModernTheme::TEXT_SECONDARY, "Active", false),
    };
    
    glass_card(ui, glow, |ui| {
        ui.horizontal(|ui| {
            status_indicator(ui, text, color, glow);
            
            if !message.is_empty() {
                ui.add_space(StyleConstants::SPACING_MD);
                ui.colored_label(ModernTheme::TEXT_SECONDARY, message);
            }
        });
    });
}

/// Progress indicator for ongoing operations
pub fn progress_indicator(ui: &mut egui::Ui, status: &AppStatus) {
    match status {
        AppStatus::Uploading { progress } => {
            neon_progress_bar(ui, *progress, ModernTheme::NEON_PINK, 8.0, Some(&format!("{:.0}%", progress * 100.0)));
        }
        AppStatus::Ripping { completed, total } => {
            let progress = *completed as f32 / *total as f32;
            neon_progress_bar(ui, progress, ModernTheme::NEON_PURPLE, 8.0, Some(&format!("{}/{}", completed, total)));
        }
        _ => {}
    }
}

/// Create a status dot indicator
pub fn status_dot(ui: &mut egui::Ui, status_type: StatusType) {
    let color = match status_type {
        StatusType::Success => ModernTheme::NEON_GREEN,
        StatusType::Warning => ModernTheme::WARNING,
        StatusType::Error => ModernTheme::ERROR,
        StatusType::Info => ModernTheme::NEON_BLUE,
        StatusType::Normal => ModernTheme::TEXT_PRIMARY,
        StatusType::Muted => ModernTheme::TEXT_MUTED,
    };
    
    ui.painter().circle_filled(
        ui.next_widget_position() + Vec2::new(6.0, 8.0),
        4.0,
        color
    );
    ui.allocate_space(Vec2::new(12.0, 16.0));
}

/// Task status with visual feedback
pub fn task_status(ui: &mut egui::Ui, task_name: &str, status: TaskStatus) {
    let (color, status_text, glow) = match status {
        TaskStatus::Pending => (ModernTheme::TEXT_MUTED, "Pending", false),
        TaskStatus::Running => (ModernTheme::NEON_BLUE, "Running", true),
        TaskStatus::Completed => (ModernTheme::NEON_GREEN, "Completed", true),
        TaskStatus::Failed => (ModernTheme::ERROR, "Failed", true),
        TaskStatus::Cancelled => (ModernTheme::WARNING, "Cancelled", false),
    };
    
    ui.horizontal(|ui| {
        status_indicator(ui, status_text, color, glow);
        ui.add_space(StyleConstants::SPACING_MD);
        ui.colored_label(ModernTheme::TEXT_PRIMARY, task_name);
    });
}

/// Task status enumeration
#[derive(Debug, Clone, Copy)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Status type enumeration for colored indicators
#[derive(Debug, Clone, Copy)]
pub enum StatusType {
    Success,
    Warning,
    Error,
    Info,
    Normal,
    Muted,
}

/// Create a card container with glassmorphism
pub fn card_container<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    glass_card(ui, false, add_contents)
}

/// Create a section header with enhanced styling
pub fn section<R>(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    ui.horizontal(|ui| {
        ui.colored_label(ModernTheme::TEXT_BRIGHT, 
            egui::RichText::new(title)
                .size(16.0)
                .strong()
        );
    });
    
    ui.add_space(StyleConstants::SPACING_SM);
    let response = add_contents(ui);
    ui.add_space(StyleConstants::SPACING_MD);
    
    response
}

/// Create a styled progress bar
pub fn styled_progress_bar(
    ui: &mut egui::Ui,
    progress: f32,
    text: Option<&str>,
) {
    neon_progress_bar(ui, progress, ModernTheme::NEON_BLUE, 8.0, text);
}