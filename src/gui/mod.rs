use crate::app::state::AppState;
use crate::config::Config;
use crate::gui::theme::{apply_modern_theme, ModernTheme, StyleConstants};
use crate::gui::components::{render_tab_bar, tab_content_area, error_display};
use crate::gui::state::{UiState, Tab};
use crate::gui::tabs::*;
use crate::gui::utils::{auto_check_for_updates, manual_check_for_updates};
use std::sync::{Arc, Mutex};
use egui::{Color32, Rounding, Stroke, Vec2, RichText, Align2, FontId};

pub mod theme;
pub mod components;
pub mod tabs;
pub mod utils;
pub mod state;

/// Main entry point for the stunning modern GUI application
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_min_inner_size([900.0, 650.0])
            .with_icon(load_icon())
            .with_decorations(true)
            .with_transparent(false),
        ..Default::default()
    };

    eframe::run_native(
        "DVD Ripper - High-Tech Edition",
        options,
        Box::new(|cc| {
            apply_modern_theme(&cc.egui_ctx);
            Ok(Box::new(DvdRipperApp::new(cc)))
        }),
    )
}

fn load_icon() -> Arc<egui::IconData> {
    Arc::new(egui::IconData {
        rgba: vec![255; 32 * 32 * 4],
        width: 32,
        height: 32,
    })
}

/// Main application with stunning high-tech interface
struct DvdRipperApp {
    app_state: Arc<Mutex<AppState>>,
    ui_state: UiState,
    config: Arc<Mutex<Config>>,
    first_frame: bool,
    animation_time: f32,
}

impl DvdRipperApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = match Config::load() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load configuration: {}", e);
                Config::default()
            }
        };
        
        let config = Arc::new(Mutex::new(config));
        let app_state = Arc::new(Mutex::new(AppState::new(
            config.lock().unwrap().clone()
        )));
        
        let mut ui_state = UiState::new();
        
        if let Ok(config_ref) = config.try_lock() {
            ui_state.load_config_temp(&*config_ref);
        }
        
        Self {
            app_state,
            ui_state,
            config,
            first_frame: true,
            animation_time: 0.0,
        }
    }
    
    fn update_status(&mut self, ctx: &egui::Context) {
        if let Ok(state) = self.app_state.try_lock() {
            match &state.status {
                crate::app::state::AppStatus::Idle => {
                    self.ui_state.set_status("System Ready".to_string());
                }
                crate::app::state::AppStatus::Scanning => {
                    self.ui_state.set_status("Scanning Media...".to_string());
                }
                crate::app::state::AppStatus::ScanComplete(count) => {
                    self.ui_state.set_status(format!("Found {} titles", count));
                }
                crate::app::state::AppStatus::Ripping { completed, total } => {
                    self.ui_state.set_status(format!("Processing {} of {}", completed, total));
                }
                crate::app::state::AppStatus::RipComplete => {
                    self.ui_state.set_status("Processing Complete".to_string());
                }
                crate::app::state::AppStatus::Uploading { progress } => {
                    self.ui_state.set_status(format!("Uploading... {:.0}%", progress * 100.0));
                }
                crate::app::state::AppStatus::UploadComplete => {
                    self.ui_state.set_status("Upload Complete".to_string());
                }
                crate::app::state::AppStatus::Completed => {
                    self.ui_state.set_status("All Operations Complete".to_string());
                }
                crate::app::state::AppStatus::Error(err) => {
                    self.ui_state.set_error(err.clone());
                }
                _ => {}
            }
            
            if let Some(dvd_arc) = &state.dvd {
                if let Ok(dvd) = dvd_arc.try_lock() {
                    if self.ui_state.titles.len() != dvd.titles.len() {
                        self.ui_state.update_titles(dvd.titles.clone());
                    }
                }
            }
        }
        
        ctx.request_repaint_after(std::time::Duration::from_millis(16));
    }
    
    fn handle_first_frame(&mut self) {
        if self.first_frame {
            self.first_frame = false;
            
            let ui_state_clone = self.ui_state.clone();
            tokio::spawn(async move {
                let mut ui_state = ui_state_clone;
                auto_check_for_updates(&mut ui_state).await;
            });
        }
    }
    
    fn save_all_configs(&mut self) {
        if let Ok(mut config) = self.config.try_lock() {
            self.update_config_from_ui(&mut config);
            
            if let Err(e) = config.save() {
                self.ui_state.set_error(format!("Failed to save configuration: {}", e));
            }
        }
    }
    
    fn update_config_from_ui(&self, config: &mut Config) {
        let temp = &self.ui_state.config_temp;
        
        if !temp.handbrake_path.is_empty() {
            config.handbrake_path = Some(temp.handbrake_path.clone().into());
        } else {
            config.handbrake_path = None;
        }
        
        config.encode_algo = temp.encode_algo.clone();
        config.eject_after_rip = temp.eject_after_rip;
        
        if let Ok(thread_count) = temp.thread_count.parse::<usize>() {
            config.thread_count = thread_count;
        }
        
        if !self.ui_state.output_path.is_empty() {
            config.output_dir = self.ui_state.output_path.clone().into();
        }
        
        config.handbrake_management.auto_download = temp.auto_download;
        config.handbrake_management.prefer_system = temp.prefer_system;
        config.handbrake_management.verify_on_startup = temp.verify_on_startup;
        
        if let Ok(cache_size) = temp.max_cache_size_mb.parse::<u64>() {
            config.handbrake_management.max_cache_size_mb = cache_size;
        }
        
        if !temp.server_host.is_empty() && !temp.server_username.is_empty() {
            config.server = Some(crate::config::ServerConfig {
                host: temp.server_host.clone(),
                username: temp.server_username.clone(),
                password: if temp.server_password.is_empty() {
                    None
                } else {
                    Some(temp.server_password.clone())
                },
                path: temp.server_path.clone(),
            });
        } else {
            config.server = None;
        }
    }

    /// Render stunning glassmorphism header with advanced effects
    fn render_futuristic_header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let header_height = StyleConstants::HEADER_HEIGHT + 20.0;
        let header_rect = ui.allocate_space(Vec2::new(ui.available_width(), header_height)).1;
        
        // Animated gradient background
        let gradient_offset = (self.animation_time * 0.5).sin() * 0.1 + 0.5;
        let bg_color = Color32::from_rgba_premultiplied(
            (12.0 + gradient_offset * 8.0) as u8,
            (15.0 + gradient_offset * 10.0) as u8,
            (23.0 + gradient_offset * 15.0) as u8,
            240
        );
        
        // Main header background with glassmorphism
        ui.painter().rect_filled(
            header_rect,
            Rounding::same(StyleConstants::ROUNDING_XL),
            bg_color,
        );
        
        // Animated border with neon glow
        let glow_intensity = (self.animation_time * 2.0).sin() * 0.3 + 0.7;
        let border_color = Color32::from_rgba_premultiplied(
            (0.0 + glow_intensity * 100.0) as u8,
            (150.0 * glow_intensity) as u8,
            255,
            (120.0 * glow_intensity) as u8
        );
        
        ui.painter().rect_stroke(
            header_rect,
            Rounding::same(StyleConstants::ROUNDING_XL),
            Stroke::new(2.0, border_color),
        );
        
        // Subtle inner glow
        ui.painter().rect_stroke(
            header_rect.shrink(1.0),
            Rounding::same(StyleConstants::ROUNDING_XL - 1.0),
            Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 255, 255, 20)),
        );
        
        // Header content
        ui.allocate_ui_at_rect(header_rect.shrink(StyleConstants::SPACING_XL), |ui| {
            ui.horizontal(|ui| {
                // Left side - Logo and title
                ui.vertical(|ui| {
                    // Main title with neon effect
                    ui.painter().text(
                        ui.next_widget_position(),
                        Align2::LEFT_TOP,
                        "DVD RIPPER",
                        FontId::proportional(28.0),
                        ModernTheme::TEXT_BRIGHT,
                    );
                    
                    // Subtitle with glow
                    ui.add_space(StyleConstants::SPACING_SM);
                    ui.painter().text(
                        ui.next_widget_position(),
                        Align2::LEFT_TOP,
                        "High-Tech Media Processing Suite",
                        FontId::proportional(14.0),
                        ModernTheme::NEON_CYAN,
                    );
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Right side - Status and system info
                    ui.vertical(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Top), |ui| {
                            // System status indicator
                            if let Ok(state) = self.app_state.try_lock() {
                                let (status_text, color, pulse) = match &state.status {
                                    crate::app::state::AppStatus::Idle => ("STANDBY", ModernTheme::NEON_CYAN, false),
                                    crate::app::state::AppStatus::Scanning => ("SCANNING", ModernTheme::NEON_BLUE, true),
                                    crate::app::state::AppStatus::Ripping { .. } => ("PROCESSING", ModernTheme::NEON_PURPLE, true),
                                    crate::app::state::AppStatus::Error(_) => ("ERROR", ModernTheme::ERROR, true),
                                    _ => ("ACTIVE", ModernTheme::NEON_GREEN, false),
                                };
                                
                                // Status dot with pulse animation
                                let dot_alpha = if pulse {
                                    ((self.animation_time * 4.0).sin() * 0.3 + 0.7) as u8
                                } else {
                                    255
                                };
                                
                                let status_color = Color32::from_rgba_premultiplied(
                                    color.r(),
                                    color.g(),
                                    color.b(),
                                    dot_alpha
                                );
                                
                                ui.horizontal(|ui| {
                                    ui.painter().circle_filled(
                                        ui.next_widget_position() + Vec2::new(6.0, 8.0),
                                        6.0,
                                        status_color
                                    );
                                    ui.add_space(16.0);
                                    ui.colored_label(status_color, 
                                        RichText::new(status_text)
                                            .size(12.0)
                                            .strong()
                                    );
                                });
                            }
                        });
                        
                        ui.add_space(StyleConstants::SPACING_SM);
                        
                        // Additional status info
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Top), |ui| {
                            if !self.ui_state.titles.is_empty() {
                                ui.colored_label(ModernTheme::TEXT_SECONDARY, 
                                    format!("{} titles • {} selected", 
                                        self.ui_state.titles.len(), 
                                        self.ui_state.selected_title_count()
                                    )
                                );
                            } else {
                                ui.colored_label(ModernTheme::TEXT_MUTED, "Ready for media input");
                            }
                        });
                    });
                });
            });
        });
        
        // Floating particles effect (subtle)
        let particles = [
            (0.2, 0.3, 3.0),
            (0.7, 0.2, 2.0),
            (0.9, 0.8, 2.5),
            (0.1, 0.7, 1.5),
        ];
        
        for (x_ratio, y_ratio, speed) in particles {
            let particle_x = header_rect.min.x + header_rect.width() * x_ratio;
            let particle_y = header_rect.min.y + header_rect.height() * y_ratio + 
                (self.animation_time * speed).sin() * 5.0;
            
            let alpha = ((self.animation_time * speed + x_ratio * 10.0).sin() * 0.3 + 0.4) as u8;
            ui.painter().circle_filled(
                egui::pos2(particle_x, particle_y),
                1.5,
                Color32::from_rgba_premultiplied(100, 200, 255, alpha * 40 / 255)
            );
        }
    }
}

impl eframe::App for DvdRipperApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update animation time
        self.animation_time += ctx.input(|i| i.unstable_dt);
        
        self.handle_first_frame();
        self.update_status(ctx);
        
        // Main application window with stunning effects
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(ModernTheme::BACKGROUND_MAIN))
            .show(ctx, |ui| {
                // Stunning glassmorphism header
                self.render_futuristic_header(ui, ctx);
                
                ui.add_space(StyleConstants::SPACING_XL);
                
                // Enhanced tab navigation
                render_tab_bar(ui, &mut self.ui_state.active_tab);
                
                // Main content with enhanced styling
                tab_content_area(ui, |ui| {
                    error_display(ui, &mut self.ui_state.error_message);
                    
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
                        Tab::About => {
                            render_about_tab(ui, &mut self.ui_state);
                        }
                    }
                });
            });
        
        // Enhanced keyboard shortcuts
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            if self.ui_state.active_tab == Tab::Main {
                // TODO: Trigger DVD scan
            }
        }
        
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.save_all_configs();
        }
        
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::U)) {
            let ui_state_clone = self.ui_state.clone();
            tokio::spawn(async move {
                let mut ui_state = ui_state_clone;
                manual_check_for_updates(&mut ui_state).await;
            });
        }
        
        // Tab switching shortcuts (Ctrl+1-5)
        for (i, tab) in Tab::all().iter().enumerate() {
            if ctx.input(|input| {
                input.modifiers.ctrl && input.key_pressed(egui::Key::from_name(&(i + 1).to_string()).unwrap_or(egui::Key::Num1))
            }) {
                self.ui_state.active_tab = *tab;
            }
        }
    }
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_all_configs();
    }
}