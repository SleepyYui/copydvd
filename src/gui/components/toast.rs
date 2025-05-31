use crate::gui::state::{ToastNotification, ToastType};
use crate::gui::theme::{ModernTheme, StyleConstants};
use egui::{Color32, Pos2, Rect, RichText, Rounding, Stroke, Vec2, Context};

pub fn render_toast_notifications(ctx: &Context, toasts: &mut Vec<ToastNotification>) {
    // Remove expired toasts
    toasts.retain(|toast| !toast.is_expired());
    
    if toasts.is_empty() {
        return;
    }
    
    // Calculate toast area (top-right corner)
    let screen_rect = ctx.screen_rect();
    let toast_width = 320.0;
    let toast_height = 60.0;
    let margin = 20.0;
    let spacing = 10.0;
    
    // Create overlay window for toasts
    egui::Area::new("toast_notifications".into())
        .fixed_pos(Pos2::new(screen_rect.max.x - toast_width - margin, screen_rect.min.y + margin))
        .movable(false)
        .interactable(false)
        .show(ctx, |ui| {
            // Render toasts from top to bottom
            for (index, toast) in toasts.iter().enumerate() {
                let y_offset = (index as f32) * (toast_height + spacing);
                let toast_rect = Rect::from_min_size(
                    Pos2::new(0.0, y_offset),
                    Vec2::new(toast_width, toast_height),
                );
                
                render_single_toast(ui, toast, toast_rect);
            }
        });
}

fn render_single_toast(ui: &mut egui::Ui, toast: &ToastNotification, rect: Rect) {
    let remaining_ratio = toast.remaining_ratio();
    
    // Fade out animation
    let alpha = if remaining_ratio < 0.2 {
        (remaining_ratio / 0.2 * 255.0) as u8
    } else {
        255
    };
    
    // Toast colors based on type
    let (bg_color, border_color, icon_color, text_color) = match toast.toast_type {
        ToastType::Success => (
            Color32::from_rgba_premultiplied(34, 197, 94, alpha),
            Color32::from_rgba_premultiplied(22, 163, 74, alpha),
            Color32::from_rgba_premultiplied(255, 255, 255, alpha),
            Color32::from_rgba_premultiplied(255, 255, 255, alpha),
        ),
        ToastType::Error => (
            Color32::from_rgba_premultiplied(239, 68, 68, alpha),
            Color32::from_rgba_premultiplied(220, 38, 38, alpha),
            Color32::from_rgba_premultiplied(255, 255, 255, alpha),
            Color32::from_rgba_premultiplied(255, 255, 255, alpha),
        ),
        ToastType::Warning => (
            Color32::from_rgba_premultiplied(245, 158, 11, alpha),
            Color32::from_rgba_premultiplied(217, 119, 6, alpha),
            Color32::from_rgba_premultiplied(255, 255, 255, alpha),
            Color32::from_rgba_premultiplied(255, 255, 255, alpha),
        ),
        ToastType::Info => (
            Color32::from_rgba_premultiplied(59, 130, 246, alpha),
            Color32::from_rgba_premultiplied(37, 99, 235, alpha),
            Color32::from_rgba_premultiplied(255, 255, 255, alpha),
            Color32::from_rgba_premultiplied(255, 255, 255, alpha),
        ),
    };
    
    // Background
    ui.painter().rect_filled(
        rect,
        Rounding::same(8.0),
        bg_color,
    );
    
    // Border
    ui.painter().rect_stroke(
        rect,
        Rounding::same(8.0),
        Stroke::new(1.0, border_color),
    );
    
    // Progress bar at bottom
    let progress_height = 3.0;
    let progress_rect = Rect::from_min_size(
        rect.min + Vec2::new(0.0, rect.height() - progress_height),
        Vec2::new(rect.width() * remaining_ratio, progress_height),
    );
    ui.painter().rect_filled(
        progress_rect,
        Rounding::same(1.5),
        Color32::from_rgba_premultiplied(255, 255, 255, (alpha as f32 * 0.8) as u8),
    );
    
    // Content area
    ui.allocate_ui_at_rect(rect, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            
            // Icon
            let icon = match toast.toast_type {
                ToastType::Success => "✓",
                ToastType::Error => "✕",
                ToastType::Warning => "⚠",
                ToastType::Info => "ⓘ",
            };
            
            ui.vertical_centered(|ui| {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(icon)
                        .size(16.0)
                        .color(icon_color)
                        .strong(),
                );
            });
            
            ui.add_space(8.0);
            
            // Message
            ui.vertical(|ui| {
                ui.add_space(6.0);
                
                // Type label
                let type_label = match toast.toast_type {
                    ToastType::Success => "Success",
                    ToastType::Error => "Error",
                    ToastType::Warning => "Warning",
                    ToastType::Info => "Info",
                };
                
                ui.label(
                    RichText::new(type_label)
                        .size(11.0)
                        .color(Color32::from_rgba_premultiplied(255, 255, 255, (alpha as f32 * 0.8) as u8))
                        .strong(),
                );
                
                // Message text
                ui.label(
                    RichText::new(&toast.message)
                        .size(13.0)
                        .color(text_color),
                );
            });
        });
    });
}

pub fn add_success_toast(toasts: &mut Vec<ToastNotification>, message: String) {
    toasts.push(ToastNotification::new(message, ToastType::Success));
}

pub fn add_error_toast(toasts: &mut Vec<ToastNotification>, message: String) {
    toasts.push(ToastNotification::new(message, ToastType::Error));
}

pub fn add_warning_toast(toasts: &mut Vec<ToastNotification>, message: String) {
    toasts.push(ToastNotification::new(message, ToastType::Warning));
}

pub fn add_info_toast(toasts: &mut Vec<ToastNotification>, message: String) {
    toasts.push(ToastNotification::new(message, ToastType::Info));
}