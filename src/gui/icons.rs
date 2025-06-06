use egui::{Color32, ColorImage, Context, TextureHandle};
use std::collections::HashMap;

/// SVG icon manager for the application
#[allow(dead_code, reason = "Future feature for custom SVG icon rendering")]
pub struct IconManager {
    icons: HashMap<String, TextureHandle>,
}

#[allow(dead_code, reason = "Future feature for custom SVG icon rendering")]
impl IconManager {
    pub fn new() -> Self {
        Self {
            icons: HashMap::new(),
        }
    }

    /// Load and cache an icon by name
    pub fn get_icon(
        &mut self,
        ctx: &Context,
        name: &str,
        size: f32,
        color: Color32,
    ) -> Option<&TextureHandle> {
        let key = format!("{}_{}_{}_{}", name, size, color.r(), color.g());

        if !self.icons.contains_key(&key) {
            if let Some(texture) = self.create_icon_texture(ctx, name, size, color) {
                self.icons.insert(key.clone(), texture);
            } else {
                return None;
            }
        }

        self.icons.get(&key)
    }

    /// Create an icon texture from geometric shapes
    fn create_icon_texture(
        &self,
        ctx: &Context,
        name: &str,
        size: f32,
        color: Color32,
    ) -> Option<TextureHandle> {
        let pixels = self.create_icon_pixels(name, size as usize, color);
        let color_image =
            ColorImage::from_rgba_unmultiplied([size as usize, size as usize], &pixels);

        Some(ctx.load_texture(
            format!("icon_{}", name),
            color_image,
            egui::TextureOptions::default(),
        ))
    }

    /// Create colored pixels for an icon using geometric shapes
    fn create_icon_pixels(&self, name: &str, size: usize, color: Color32) -> Vec<u8> {
        let mut pixels = vec![0u8; size * size * 4];
        let center_x = size as f32 / 2.0;
        let center_y = size as f32 / 2.0;

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                let px = x as f32;
                let py = y as f32;

                let should_draw = match name {
                    "check" => {
                        // Draw a checkmark
                        let t = px / size as f32;
                        let check_y = if t < 0.5 {
                            center_y + (t - 0.25) * size as f32 * 0.4
                        } else {
                            center_y + (0.75 - t) * size as f32 * 0.8
                        };
                        (py - check_y).abs() < 2.0
                            && px > size as f32 * 0.2
                            && px < size as f32 * 0.8
                    }
                    "cross" => {
                        // Draw an X
                        let dx = px - center_x;
                        let dy = py - center_y;
                        ((dx - dy).abs() < 2.0 || (dx + dy).abs() < 2.0)
                            && dx.abs() < size as f32 * 0.3
                            && dy.abs() < size as f32 * 0.3
                    }
                    "warning" => {
                        // Draw a triangle
                        let rel_y = py / size as f32;
                        let triangle_width = rel_y * size as f32 * 0.8;
                        let x_from_center = (px - center_x).abs();
                        x_from_center <= triangle_width / 2.0
                            && py > size as f32 * 0.2
                            && py < size as f32 * 0.8
                    }
                    "info" => {
                        // Draw an i
                        let distance_from_center =
                            ((px - center_x).powi(2) + (py - center_y).powi(2)).sqrt();
                        let radius = size as f32 * 0.3;
                        distance_from_center < radius
                    }
                    "refresh" => {
                        // Draw circular arrow
                        let distance = ((px - center_x).powi(2) + (py - center_y).powi(2)).sqrt();
                        let inner_radius = size as f32 * 0.2;
                        let outer_radius = size as f32 * 0.35;
                        distance > inner_radius && distance < outer_radius
                    }
                    "search" => {
                        // Draw magnifying glass
                        let circle_distance =
                            ((px - center_x * 0.8).powi(2) + (py - center_y * 0.8).powi(2)).sqrt();
                        let circle_radius = size as f32 * 0.2;
                        let handle_distance =
                            ((px - center_x * 1.4).powi(2) + (py - center_y * 1.4).powi(2)).sqrt();
                        (circle_distance > circle_radius - 2.0
                            && circle_distance < circle_radius + 2.0)
                            || (handle_distance < 3.0 && px > center_x && py > center_y)
                    }
                    "wrench" => {
                        // Draw wrench shape
                        let in_handle = px > size as f32 * 0.1
                            && px < size as f32 * 0.3
                            && py > size as f32 * 0.3
                            && py < size as f32 * 0.7;
                        let in_head =
                            ((px - center_x * 1.3).powi(2) + (py - center_y * 0.7).powi(2)).sqrt()
                                < size as f32 * 0.15;
                        in_handle || in_head
                    }
                    "lightning" => {
                        // Draw lightning bolt
                        let rel_x = px / size as f32;
                        let rel_y = py / size as f32;
                        let in_bolt =
                            (rel_y < 0.5 && rel_x > 0.3 - rel_y * 0.2 && rel_x < 0.7 - rel_y * 0.2)
                                || (rel_y >= 0.5
                                    && rel_x > 0.3 + (rel_y - 0.5) * 0.2
                                    && rel_x < 0.7 + (rel_y - 0.5) * 0.2);
                        in_bolt && rel_y > 0.1 && rel_y < 0.9
                    }
                    "lightbulb" => {
                        // Draw light bulb
                        let bulb_distance =
                            ((px - center_x).powi(2) + (py - center_y * 0.8).powi(2)).sqrt();
                        let base_in = px > center_x - size as f32 * 0.1
                            && px < center_x + size as f32 * 0.1
                            && py > size as f32 * 0.7
                            && py < size as f32 * 0.9;
                        bulb_distance < size as f32 * 0.25 || base_in
                    }
                    "arrow-right" => {
                        // Draw right arrow
                        let in_shaft = py > center_y - 3.0
                            && py < center_y + 3.0
                            && px > size as f32 * 0.2
                            && px < size as f32 * 0.7;
                        let in_head = (px - center_x * 1.3).abs() + (py - center_y).abs()
                            < size as f32 * 0.15
                            && px > center_x;
                        in_shaft || in_head
                    }
                    _ => {
                        // Default circle
                        let distance = ((px - center_x).powi(2) + (py - center_y).powi(2)).sqrt();
                        distance < size as f32 * 0.3
                    }
                };

                if should_draw {
                    pixels[idx] = color.r(); // R
                    pixels[idx + 1] = color.g(); // G
                    pixels[idx + 2] = color.b(); // B
                    pixels[idx + 3] = color.a(); // A
                } else {
                    pixels[idx + 3] = 0; // Transparent
                }
            }
        }

        pixels
    }
}

/// Convenience function to render an icon
#[allow(dead_code, reason = "Future feature for custom SVG icon rendering")]
pub fn render_icon(
    ui: &mut egui::Ui,
    icon_manager: &mut IconManager,
    name: &str,
    size: f32,
    color: Color32,
) {
    if let Some(texture) = icon_manager.get_icon(ui.ctx(), name, size, color) {
        ui.add(egui::Image::from_texture(texture).fit_to_exact_size(egui::Vec2::splat(size)));
    } else {
        // Fallback to simple text
        ui.label(name);
    }
}

/// Simple icon rendering function that creates icons from geometric shapes
#[allow(dead_code, reason = "SVG icons removed from UI")]
pub fn svg_icon(ui: &mut egui::Ui, name: &str, size: f32, color: Color32) {
    // Create icon directly without using the manager to avoid lifetime issues
    let pixels = create_icon_pixels_direct(name, size as usize, color);
    let color_image =
        egui::ColorImage::from_rgba_unmultiplied([size as usize, size as usize], &pixels);

    let texture = ui.ctx().load_texture(
        format!("direct_icon_{}_{}", name, size),
        color_image,
        egui::TextureOptions::default(),
    );

    ui.add(egui::Image::from_texture(&texture).fit_to_exact_size(egui::Vec2::splat(size)));
}

/// Create icon pixels directly without using the IconManager
fn create_icon_pixels_direct(name: &str, size: usize, color: egui::Color32) -> Vec<u8> {
    let mut pixels = vec![0u8; size * size * 4];
    let center_x = size as f32 / 2.0;
    let center_y = size as f32 / 2.0;

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let px = x as f32;
            let py = y as f32;

            let should_draw = match name {
                "check" => {
                    // Draw a perfect checkmark - two connected lines
                    let thickness = size as f32 * 0.1;

                    // First line: bottom-left to center-bottom (downward stroke)
                    let line1_start_x = center_x - size as f32 * 0.25;
                    let line1_start_y = center_y - size as f32 * 0.05;
                    let line1_end_x = center_x - size as f32 * 0.05;
                    let line1_end_y = center_y + size as f32 * 0.15;

                    // Second line: center-bottom to top-right (upward stroke)
                    let line2_start_x = line1_end_x;
                    let line2_start_y = line1_end_y;
                    let line2_end_x = center_x + size as f32 * 0.3;
                    let line2_end_y = center_y - size as f32 * 0.2;

                    // Check if point is on first line
                    let on_line1 = {
                        let line_vec_x = line1_end_x - line1_start_x;
                        let line_vec_y = line1_end_y - line1_start_y;
                        let point_vec_x = px - line1_start_x;
                        let point_vec_y = py - line1_start_y;

                        let line_len = (line_vec_x.powi(2) + line_vec_y.powi(2)).sqrt();
                        if line_len > 0.0 {
                            let t = (point_vec_x * line_vec_x + point_vec_y * line_vec_y)
                                / (line_len * line_len);
                            if (0.0..=1.0).contains(&t) {
                                let closest_x = line1_start_x + t * line_vec_x;
                                let closest_y = line1_start_y + t * line_vec_y;
                                let distance =
                                    ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt();
                                distance < thickness
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };

                    // Check if point is on second line
                    let on_line2 = {
                        let line_vec_x = line2_end_x - line2_start_x;
                        let line_vec_y = line2_end_y - line2_start_y;
                        let point_vec_x = px - line2_start_x;
                        let point_vec_y = py - line2_start_y;

                        let line_len = (line_vec_x.powi(2) + line_vec_y.powi(2)).sqrt();
                        if line_len > 0.0 {
                            let t = (point_vec_x * line_vec_x + point_vec_y * line_vec_y)
                                / (line_len * line_len);
                            if (0.0..=1.0).contains(&t) {
                                let closest_x = line2_start_x + t * line_vec_x;
                                let closest_y = line2_start_y + t * line_vec_y;
                                let distance =
                                    ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt();
                                distance < thickness
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };

                    on_line1 || on_line2
                }
                "cross" => {
                    // Draw a beautiful X
                    let dx = px - center_x;
                    let dy = py - center_y;
                    let thickness = size as f32 * 0.06;
                    let radius = size as f32 * 0.35;
                    ((dx - dy).abs() < thickness || (dx + dy).abs() < thickness)
                        && dx.abs() < radius
                        && dy.abs() < radius
                }
                "warning" => {
                    // Draw a triangle
                    let rel_y = py / size as f32;
                    let triangle_width = rel_y * size as f32 * 0.8;
                    let x_from_center = (px - center_x).abs();
                    x_from_center <= triangle_width / 2.0
                        && py > size as f32 * 0.2
                        && py < size as f32 * 0.8
                }
                "info" => {
                    // Draw a circle
                    let distance_from_center =
                        ((px - center_x).powi(2) + (py - center_y).powi(2)).sqrt();
                    let radius = size as f32 * 0.3;
                    distance_from_center < radius
                }
                "refresh" => {
                    // Draw circular arrow
                    let distance = ((px - center_x).powi(2) + (py - center_y).powi(2)).sqrt();
                    let inner_radius = size as f32 * 0.2;
                    let outer_radius = size as f32 * 0.35;
                    distance > inner_radius && distance < outer_radius
                }
                "search" => {
                    // Draw magnifying glass
                    let circle_distance =
                        ((px - center_x * 0.8).powi(2) + (py - center_y * 0.8).powi(2)).sqrt();
                    let circle_radius = size as f32 * 0.2;
                    let handle_distance =
                        ((px - center_x * 1.4).powi(2) + (py - center_y * 1.4).powi(2)).sqrt();
                    (circle_distance > circle_radius - 2.0 && circle_distance < circle_radius + 2.0)
                        || (handle_distance < 3.0 && px > center_x && py > center_y)
                }
                "wrench" => {
                    // Draw wrench shape
                    let in_handle = px > size as f32 * 0.1
                        && px < size as f32 * 0.3
                        && py > size as f32 * 0.3
                        && py < size as f32 * 0.7;
                    let in_head = ((px - center_x * 1.3).powi(2) + (py - center_y * 0.7).powi(2))
                        .sqrt()
                        < size as f32 * 0.15;
                    in_handle || in_head
                }
                "lightning" => {
                    // Draw lightning bolt
                    let rel_x = px / size as f32;
                    let rel_y = py / size as f32;
                    let in_bolt =
                        (rel_y < 0.5 && rel_x > 0.3 - rel_y * 0.2 && rel_x < 0.7 - rel_y * 0.2)
                            || (rel_y >= 0.5
                                && rel_x > 0.3 + (rel_y - 0.5) * 0.2
                                && rel_x < 0.7 + (rel_y - 0.5) * 0.2);
                    in_bolt && rel_y > 0.1 && rel_y < 0.9
                }
                "lightbulb" => {
                    // Draw light bulb
                    let bulb_distance =
                        ((px - center_x).powi(2) + (py - center_y * 0.8).powi(2)).sqrt();
                    let base_in = px > center_x - size as f32 * 0.1
                        && px < center_x + size as f32 * 0.1
                        && py > size as f32 * 0.7
                        && py < size as f32 * 0.9;
                    bulb_distance < size as f32 * 0.25 || base_in
                }
                "arrow-right" => {
                    // Draw beautiful right arrow
                    let shaft_thickness = size as f32 * 0.08;
                    let in_shaft = py > center_y - shaft_thickness
                        && py < center_y + shaft_thickness
                        && px > size as f32 * 0.2
                        && px < size as f32 * 0.65;

                    // Arrow head as triangle
                    let head_x = px - size as f32 * 0.65;
                    let head_y = py - center_y;
                    let in_head = head_x > 0.0
                        && head_x < size as f32 * 0.2
                        && head_y.abs() < head_x * 0.8 + shaft_thickness;

                    in_shaft || in_head
                }
                "play" => {
                    // Draw play triangle
                    let triangle_height = size as f32 * 0.6;
                    let triangle_width = size as f32 * 0.5;
                    let start_x = center_x - triangle_width * 0.3;
                    let rel_x = px - start_x;
                    let rel_y = py - (center_y - triangle_height * 0.5);

                    rel_x > 0.0
                        && rel_x < triangle_width
                        && rel_y > 0.0
                        && rel_y < triangle_height
                        && rel_y > rel_x * 0.8
                        && rel_y < triangle_height - rel_x * 0.8
                }
                "pause" => {
                    // Draw pause bars
                    let bar_width = size as f32 * 0.12;
                    let bar_height = size as f32 * 0.6;
                    let spacing = size as f32 * 0.15;
                    let start_y = center_y - bar_height * 0.5;
                    let left_bar = px > center_x - spacing - bar_width
                        && px < center_x - spacing
                        && py > start_y
                        && py < start_y + bar_height;
                    let right_bar = px > center_x + spacing
                        && px < center_x + spacing + bar_width
                        && py > start_y
                        && py < start_y + bar_height;
                    left_bar || right_bar
                }
                "stop" => {
                    // Draw stop square
                    let square_size = size as f32 * 0.5;
                    let start_x = center_x - square_size * 0.5;
                    let start_y = center_y - square_size * 0.5;
                    px > start_x
                        && px < start_x + square_size
                        && py > start_y
                        && py < start_y + square_size
                }
                "folder" => {
                    // Draw folder icon
                    let folder_width = size as f32 * 0.7;
                    let folder_height = size as f32 * 0.5;
                    let tab_width = size as f32 * 0.25;
                    let tab_height = size as f32 * 0.1;
                    let start_x = center_x - folder_width * 0.5;
                    let start_y = center_y - folder_height * 0.3;

                    // Main folder body
                    let in_body = px > start_x
                        && px < start_x + folder_width
                        && py > start_y + tab_height
                        && py < start_y + folder_height;

                    // Folder tab
                    let in_tab = px > start_x
                        && px < start_x + tab_width
                        && py > start_y
                        && py < start_y + tab_height;

                    in_body || in_tab
                }
                "gear" => {
                    // Draw gear/settings icon
                    let outer_radius = size as f32 * 0.35;
                    let inner_radius = size as f32 * 0.15;
                    let distance = ((px - center_x).powi(2) + (py - center_y).powi(2)).sqrt();

                    // Create gear teeth effect
                    let angle = (py - center_y).atan2(px - center_x);
                    let teeth_count = 8.0;
                    let tooth_angle = (angle * teeth_count).sin();
                    let effective_radius = if tooth_angle > 0.5 {
                        outer_radius * 1.1
                    } else {
                        outer_radius * 0.9
                    };

                    distance > inner_radius && distance < effective_radius
                }
                "download" => {
                    // Draw download arrow
                    let shaft_width = size as f32 * 0.08;
                    let shaft_height = size as f32 * 0.4;
                    let arrow_width = size as f32 * 0.2;
                    let arrow_height = size as f32 * 0.15;

                    // Vertical shaft
                    let in_shaft = px > center_x - shaft_width
                        && px < center_x + shaft_width
                        && py > center_y - shaft_height
                        && py < center_y + shaft_height * 0.5;

                    // Down arrow head
                    let arrow_y = py - (center_y + shaft_height * 0.3);
                    let arrow_x = (px - center_x).abs();
                    let in_arrow = arrow_y > 0.0
                        && arrow_y < arrow_height
                        && arrow_x < arrow_width - arrow_y * (arrow_width / arrow_height);

                    in_shaft || in_arrow
                }
                "upload" => {
                    // Draw upload arrow
                    let shaft_width = size as f32 * 0.08;
                    let shaft_height = size as f32 * 0.4;
                    let arrow_width = size as f32 * 0.2;
                    let arrow_height = size as f32 * 0.15;

                    // Vertical shaft
                    let in_shaft = px > center_x - shaft_width
                        && px < center_x + shaft_width
                        && py > center_y - shaft_height * 0.5
                        && py < center_y + shaft_height;

                    // Up arrow head
                    let arrow_y = (center_y - shaft_height * 0.3) - py;
                    let arrow_x = (px - center_x).abs();
                    let in_arrow = arrow_y > 0.0
                        && arrow_y < arrow_height
                        && arrow_x < arrow_width - arrow_y * (arrow_width / arrow_height);

                    in_shaft || in_arrow
                }
                "heart" => {
                    // Draw heart icon
                    let heart_size = size as f32 * 0.3;
                    let left_center_x = center_x - heart_size * 0.3;
                    let right_center_x = center_x + heart_size * 0.3;
                    let circle_y = center_y - heart_size * 0.2;

                    let left_circle = ((px - left_center_x).powi(2) + (py - circle_y).powi(2))
                        .sqrt()
                        < heart_size * 0.5;
                    let right_circle = ((px - right_center_x).powi(2) + (py - circle_y).powi(2))
                        .sqrt()
                        < heart_size * 0.5;

                    // Bottom triangle
                    let triangle_tip_y = center_y + heart_size * 0.6;
                    let triangle_x = (px - center_x).abs();
                    let triangle_y_offset = py - circle_y;
                    let in_triangle = py > circle_y
                        && py < triangle_tip_y
                        && triangle_x
                            < heart_size * 0.8 * (1.0 - triangle_y_offset / (heart_size * 0.8));

                    left_circle || right_circle || in_triangle
                }
                "star" => {
                    // Draw 5-pointed star
                    let outer_radius = size as f32 * 0.35;
                    let inner_radius = size as f32 * 0.15;
                    let distance = ((px - center_x).powi(2) + (py - center_y).powi(2)).sqrt();
                    let angle = (py - center_y).atan2(px - center_x) + std::f32::consts::PI;
                    let star_angle = (angle * 5.0 / (2.0 * std::f32::consts::PI)) % 1.0;
                    let in_point = star_angle < 0.5;
                    let effective_radius = if in_point { outer_radius } else { inner_radius };
                    distance < effective_radius
                }
                "home" => {
                    // Draw house icon
                    let house_width = size as f32 * 0.5;
                    let house_height = size as f32 * 0.4;
                    let roof_height = size as f32 * 0.25;
                    let start_x = center_x - house_width * 0.5;
                    let roof_y = center_y - house_height * 0.5;
                    let house_y = roof_y + roof_height;

                    // House body
                    let in_house = px > start_x
                        && px < start_x + house_width
                        && py > house_y
                        && py < house_y + house_height;

                    // Triangular roof
                    let roof_x_offset = (px - center_x).abs();
                    let roof_y_offset = py - roof_y;
                    let in_roof = roof_y_offset > 0.0
                        && roof_y_offset < roof_height
                        && roof_x_offset < house_width * 0.5 * (1.0 - roof_y_offset / roof_height);

                    in_house || in_roof
                }
                "menu" => {
                    // Draw hamburger menu
                    let line_width = size as f32 * 0.6;
                    let line_height = size as f32 * 0.08;
                    let spacing = size as f32 * 0.12;
                    let start_x = center_x - line_width * 0.5;

                    let line1 = px > start_x
                        && px < start_x + line_width
                        && py > center_y - spacing - line_height
                        && py < center_y - spacing;
                    let line2 = px > start_x
                        && px < start_x + line_width
                        && py > center_y - line_height * 0.5
                        && py < center_y + line_height * 0.5;
                    let line3 = px > start_x
                        && px < start_x + line_width
                        && py > center_y + spacing
                        && py < center_y + spacing + line_height;

                    line1 || line2 || line3
                }
                "save" => {
                    // Draw floppy disk save icon
                    let disk_size = size as f32 * 0.6;
                    let start_x = center_x - disk_size * 0.5;
                    let start_y = center_y - disk_size * 0.5;
                    let notch_size = size as f32 * 0.1;

                    // Main disk body
                    let in_body = px > start_x
                        && px < start_x + disk_size
                        && py > start_y
                        && py < start_y + disk_size;

                    // Remove top-right notch
                    let in_notch = px > start_x + disk_size - notch_size
                        && px < start_x + disk_size
                        && py > start_y
                        && py < start_y + notch_size;

                    // Label area
                    let label_height = size as f32 * 0.15;
                    let in_label = px > start_x + size as f32 * 0.1
                        && px < start_x + disk_size - size as f32 * 0.1
                        && py > start_y + disk_size - label_height - size as f32 * 0.1
                        && py < start_y + disk_size - size as f32 * 0.1;

                    (in_body && !in_notch) || in_label
                }
                _ => {
                    // Default circle
                    let distance = ((px - center_x).powi(2) + (py - center_y).powi(2)).sqrt();
                    distance < size as f32 * 0.3
                }
            };

            if should_draw {
                pixels[idx] = color.r(); // R
                pixels[idx + 1] = color.g(); // G
                pixels[idx + 2] = color.b(); // B
                pixels[idx + 3] = color.a(); // A
            } else {
                pixels[idx + 3] = 0; // Transparent
            }
        }
    }

    pixels
}
