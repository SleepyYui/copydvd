use crate::gui::state::Tab;
use crate::gui::theme::{ModernTheme, StyleConstants};
use egui::{Color32, Rounding, Stroke, Vec2, Pos2, Shape, Align2, FontId};

/// Render stunning glassmorphism tab bar with advanced effects
pub fn render_tab_bar(ui: &mut egui::Ui, active_tab: &mut Tab) {
    let tab_height = StyleConstants::TAB_HEIGHT;
    let tabs = Tab::all();

    // Background gradient for tab bar
    let tab_bar_rect = ui.allocate_space(Vec2::new(ui.available_width(), tab_height)).1;
    
    // Gradient background
    ui.painter().rect_filled(
        tab_bar_rect,
        Rounding::ZERO,
        ModernTheme::GLASS_SURFACE,
    );

    ui.allocate_ui_at_rect(tab_bar_rect, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(StyleConstants::SPACING_XL);
            
            for (index, tab) in tabs.iter().enumerate() {
                let is_active = *active_tab == *tab;
                let tab_width = 140.0;

                let response = ui.allocate_response(
                    Vec2::new(tab_width, tab_height - 12.0), 
                    egui::Sense::click()
                );
                let rect = response.rect;

                if response.clicked() {
                    *active_tab = *tab;
                }

                // Tab styling with glassmorphism
                let (bg_color, border_color, glow) = if is_active {
                    (ModernTheme::GLASS_ELEVATED, ModernTheme::NEON_BLUE, true)
                } else if response.hovered() {
                    (ModernTheme::GLASS_SURFACE, ModernTheme::BORDER_BRIGHT, false)
                } else {
                    (Color32::TRANSPARENT, Color32::TRANSPARENT, false)
                };

                // Glow effect for active tab
                if glow {
                    ui.painter().rect_filled(
                        rect.expand(2.0),
                        Rounding::same(StyleConstants::ROUNDING_LG + 2.0),
                        Color32::from_rgba_premultiplied(0, 150, 255, 30),
                    );
                }

                // Tab background with glassmorphism
                if bg_color != Color32::TRANSPARENT {
                    ui.painter().rect_filled(
                        rect,
                        Rounding::same(StyleConstants::ROUNDING_LG),
                        bg_color,
                    );
                }

                // Tab border
                if border_color != Color32::TRANSPARENT {
                    ui.painter().rect_stroke(
                        rect,
                        Rounding::same(StyleConstants::ROUNDING_LG),
                        Stroke::new(1.5, border_color),
                    );
                }

                // Active tab indicator (bottom accent line)
                if is_active {
                    let indicator_rect = egui::Rect::from_min_size(
                        rect.min + Vec2::new(StyleConstants::SPACING_MD, rect.height() - 3.0),
                        Vec2::new(rect.width() - StyleConstants::SPACING_MD * 2.0, 3.0),
                    );
                    ui.painter().rect_filled(
                        indicator_rect,
                        Rounding::same(1.5),
                        ModernTheme::NEON_BLUE,
                    );
                }

                // Tab content
                let text_color = if is_active {
                    ModernTheme::TEXT_BRIGHT
                } else if response.hovered() {
                    ModernTheme::TEXT_PRIMARY
                } else {
                    ModernTheme::TEXT_SECONDARY
                };

                let content_rect = rect.shrink(StyleConstants::SPACING_MD);

                // Draw advanced geometric icon
                let icon_center = content_rect.left_center() + Vec2::new(20.0, 0.0);
                draw_advanced_tab_icon(ui, *tab, icon_center, text_color, is_active);

                // Tab label with enhanced typography
                let text_pos = content_rect.left_center() + Vec2::new(45.0, 0.0);
                ui.painter().text(
                    text_pos,
                    Align2::LEFT_CENTER,
                    tab.name(),
                    FontId::proportional(13.0),
                    text_color,
                );

                // Add spacing between tabs
                if index < tabs.len() - 1 {
                    ui.add_space(StyleConstants::SPACING_SM);
                }
            }
        });
    });

    // Bottom border with gradient effect
    let border_rect = egui::Rect::from_min_size(
        tab_bar_rect.min + Vec2::new(0.0, tab_bar_rect.height() - 1.0),
        Vec2::new(tab_bar_rect.width(), 1.0),
    );
    ui.painter().rect_filled(
        border_rect,
        Rounding::ZERO,
        ModernTheme::BORDER_BRIGHT,
    );

    ui.add_space(StyleConstants::SPACING_LG);
}

/// Draw advanced geometric icons for tabs with neon effects
fn draw_advanced_tab_icon(ui: &mut egui::Ui, tab: Tab, center: Pos2, color: Color32, active: bool) {
    let painter = ui.painter();
    let size = 12.0;
    let glow_color = if active {
        Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 80)
    } else {
        Color32::TRANSPARENT
    };

    match tab {
        Tab::Main => {
            // Modern play button with glow effect
            if active {
                painter.circle_filled(center, size + 2.0, glow_color);
            }
            
            let points = [
                center + Vec2::new(-size * 0.4, -size * 0.6),
                center + Vec2::new(-size * 0.4, size * 0.6),
                center + Vec2::new(size * 0.5, 0.0),
            ];
            painter.add(Shape::convex_polygon(points.to_vec(), color, Stroke::NONE));
        }
        Tab::Config => {
            // Advanced gear with multiple teeth
            if active {
                painter.circle_filled(center, size + 4.0, glow_color);
            }
            
            // Main gear body
            painter.circle_filled(center, size * 0.7, color);
            painter.circle_filled(center, size * 0.35, ModernTheme::BACKGROUND_MAIN);
            
            // Gear teeth as small rectangles
            for i in 0..8 {
                let angle = i as f32 * std::f32::consts::PI / 4.0;
                let tooth_pos = center + Vec2::new(
                    size * 0.9 * angle.cos(),
                    size * 0.9 * angle.sin()
                );
                painter.rect_filled(
                    egui::Rect::from_center_size(tooth_pos, Vec2::splat(3.0)),
                    Rounding::same(1.0),
                    color
                );
            }
        }
        Tab::Server => {
            // Modern server stack with glow
            if active {
                painter.rect_filled(
                    egui::Rect::from_center_size(center, Vec2::new(size * 2.0, size * 1.6)),
                    Rounding::same(4.0),
                    glow_color
                );
            }
            
            for i in 0..3 {
                let y_offset = (i as f32 - 1.0) * size * 0.5;
                let server_rect = egui::Rect::from_center_size(
                    center + Vec2::new(0.0, y_offset),
                    Vec2::new(size * 1.4, size * 0.35)
                );
                
                painter.rect_filled(
                    server_rect,
                    Rounding::same(2.0),
                    color
                );
                
                // Server indicator dots
                for j in 0..2 {
                    let dot_x = server_rect.left() + size * 0.2 + j as f32 * size * 0.15;
                    painter.circle_filled(
                        Pos2::new(dot_x, server_rect.center().y),
                        1.0,
                        ModernTheme::BACKGROUND_MAIN
                    );
                }
            }
        }
        Tab::HandBrake => {
            // Advanced film strip with perforations
            if active {
                painter.rect_filled(
                    egui::Rect::from_center_size(center, Vec2::new(size * 2.0, size * 1.2)),
                    Rounding::same(3.0),
                    glow_color
                );
            }
            
            // Main film strip
            painter.rect_filled(
                egui::Rect::from_center_size(center, Vec2::new(size * 1.6, size * 0.9)),
                Rounding::same(2.0),
                color
            );
            
            // Film perforations (top and bottom)
            for i in 0..4 {
                let x_offset = (i as f32 - 1.5) * size * 0.3;
                
                // Top perforations
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        center + Vec2::new(x_offset, -size * 0.6),
                        Vec2::new(size * 0.15, size * 0.25)
                    ),
                    Rounding::same(1.0),
                    ModernTheme::BACKGROUND_MAIN
                );
                
                // Bottom perforations
                painter.rect_filled(
                    egui::Rect::from_center_size(
                        center + Vec2::new(x_offset, size * 0.6),
                        Vec2::new(size * 0.15, size * 0.25)
                    ),
                    Rounding::same(1.0),
                    ModernTheme::BACKGROUND_MAIN
                );
            }
            
            // Central frame indicator
            painter.rect_stroke(
                egui::Rect::from_center_size(center, Vec2::new(size * 0.6, size * 0.4)),
                Rounding::same(1.0),
                Stroke::new(1.0, ModernTheme::BACKGROUND_MAIN)
            );
        }
        Tab::About => {
            // Modern info icon with glow
            if active {
                painter.circle_filled(center, size + 2.0, glow_color);
            }
            
            // Main circle
            painter.circle_filled(center, size * 0.7, color);
            
            // Info dot
            painter.circle_filled(
                center + Vec2::new(0.0, -size * 0.3), 
                size * 0.12, 
                ModernTheme::BACKGROUND_MAIN
            );
            
            // Info line
            painter.rect_filled(
                egui::Rect::from_center_size(
                    center + Vec2::new(0.0, size * 0.15),
                    Vec2::new(size * 0.24, size * 0.5)
                ),
                Rounding::same(2.0),
                ModernTheme::BACKGROUND_MAIN
            );
        }
    }
}

/// Create a tab content area with glassmorphism scrolling
pub fn tab_content_area<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    // Enhanced scroll area with glassmorphism
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            
            // Add top spacing
            ui.add_space(StyleConstants::SPACING_XL);
            
            // Content with enhanced styling
            let response = add_contents(ui);
            
            // Add bottom spacing
            ui.add_space(StyleConstants::SPACING_XXXL);
            
            response
        }).inner
}