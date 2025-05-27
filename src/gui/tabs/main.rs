use crate::app::state::AppState;
use crate::gui::state::UiState;
use crate::gui::theme::{ModernTheme, StyleConstants, glass_card, tech_section, neon_progress_bar, neon_button, status_indicator, step_indicator, responsive_container, animated_glass_card, get_device_type, DeviceType};
use egui::{Color32, Rounding, Stroke, Vec2, Align, Layout, RichText};
use std::sync::{Arc, Mutex};

/// Render stunning high-tech main tab with glassmorphism and neon effects
pub fn render_main_tab(ui: &mut egui::Ui, ui_state: &mut UiState, app_state: Arc<Mutex<AppState>>) {
    // Futuristic header with gradient background
    render_tech_header(ui, ui_state, app_state.clone());
    
    ui.add_space(StyleConstants::SPACING_XL);
    
    // Main content with responsive container
    responsive_container(ui, |ui| {
        // Hero workflow section with advanced styling
        render_workflow_hero(ui, ui_state, app_state.clone());
        
        let screen_width = ui.available_width();
        ui.add_space(StyleConstants::responsive_spacing(screen_width, StyleConstants::SPACING_XXL));
        
        // Advanced title selection if available
        if !ui_state.titles.is_empty() {
            render_advanced_title_selection(ui, ui_state);
        }
    });
}

/// Render high-tech header with status and progress
fn render_tech_header(ui: &mut egui::Ui, ui_state: &mut UiState, app_state: Arc<Mutex<AppState>>) {
    // Gradient background panel
    let header_rect = ui.allocate_space(Vec2::new(ui.available_width(), StyleConstants::HEADER_HEIGHT)).1;
    
    // Draw gradient background
    ui.painter().rect_filled(
        header_rect,
        Rounding::same(StyleConstants::ROUNDING_LG),
        ModernTheme::GLASS_ELEVATED,
    );
    
    // Subtle border with glow
    ui.painter().rect_stroke(
        header_rect,
        Rounding::same(StyleConstants::ROUNDING_LG),
        Stroke::new(1.5, ModernTheme::BORDER_BRIGHT),
    );
    
    // Header content
    ui.allocate_ui_at_rect(header_rect.shrink(StyleConstants::SPACING_LG), |ui| {
        ui.horizontal(|ui| {
            // Status section with neon indicators
            ui.vertical(|ui| {
                if let Ok(state) = app_state.try_lock() {
                    let (status_text, color, glow) = match &state.status {
                        crate::app::state::AppStatus::Idle => ("System Ready", ModernTheme::NEON_CYAN, false),
                        crate::app::state::AppStatus::Scanning => ("Scanning Media", ModernTheme::NEON_BLUE, true),
                        crate::app::state::AppStatus::ScanComplete(count) => {
                            ui_state.status_message = format!("Found {} titles", count);
                            ("Scan Complete", ModernTheme::NEON_GREEN, true)
                        },
                        crate::app::state::AppStatus::Ripping { completed, total } => {
                            ui_state.status_message = format!("Processing {} of {}", completed, total);
                            ("Processing Media", ModernTheme::NEON_PURPLE, true)
                        },
                        crate::app::state::AppStatus::RipComplete => ("Processing Complete", ModernTheme::NEON_GREEN, true),
                        crate::app::state::AppStatus::Uploading { progress: _ } => ("Uploading", ModernTheme::NEON_PINK, true),
                        crate::app::state::AppStatus::UploadComplete => ("Upload Complete", ModernTheme::NEON_GREEN, true),
                        crate::app::state::AppStatus::Completed => ("All Operations Complete", ModernTheme::NEON_GREEN, true),
                        crate::app::state::AppStatus::Error(_) => ("System Error", ModernTheme::ERROR, true),
                        _ => ("Standby", ModernTheme::TEXT_MUTED, false),
                    };
                    
                    status_indicator(ui, status_text, color, glow);
                    
                    if !ui_state.status_message.is_empty() {
                        ui.add_space(StyleConstants::SPACING_XS);
                        ui.colored_label(ModernTheme::TEXT_SECONDARY, &ui_state.status_message);
                    }
                    
                    // Advanced progress visualization
                    if let crate::app::state::AppStatus::Uploading { progress } = &state.status {
                        ui.add_space(StyleConstants::SPACING_SM);
                        neon_progress_bar(ui, *progress, ModernTheme::NEON_PINK, 6.0, Some(&format!("{:.0}%", progress * 100.0)));
                    }
                    
                    if let crate::app::state::AppStatus::Ripping { completed, total } = &state.status {
                        ui.add_space(StyleConstants::SPACING_SM);
                        let progress = *completed as f32 / *total as f32;
                        neon_progress_bar(ui, progress, ModernTheme::NEON_PURPLE, 6.0, Some(&format!("{}/{}", completed, total)));
                    }
                }
            });
            
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // System stats or additional info
                ui.vertical(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.colored_label(ModernTheme::TEXT_BRIGHT, 
                            RichText::new("DVD RIPPER")
                                .size(20.0)
                                .strong()
                        );
                    });
                    
                    if !ui_state.titles.is_empty() {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.colored_label(ModernTheme::NEON_CYAN, 
                                format!("{} titles • {} selected", 
                                    ui_state.titles.len(), 
                                    ui_state.selected_title_count()
                                )
                            );
                        });
                    }
                });
            });
        });
    });
}

/// Render the main workflow with stunning visual effects
fn render_workflow_hero(ui: &mut egui::Ui, ui_state: &mut UiState, app_state: Arc<Mutex<AppState>>) {
    // Calculate step states
    let step1_complete = !ui_state.input_path.is_empty();
    let step2_complete = !ui_state.output_path.is_empty();
    let step3_complete = !ui_state.titles.is_empty();
    let can_rip = step3_complete && ui_state.selected_title_count() > 0;
    
    // Workflow steps with advanced animations
    render_workflow_step(ui, ui_state, 1, "Select Media Source", step1_complete, true);
    let screen_width = ui.available_width();
    let step_spacing = StyleConstants::responsive_spacing(screen_width, StyleConstants::SPACING_XL);
    ui.add_space(step_spacing);
    
    render_workflow_step(ui, ui_state, 2, "Choose Output Location", step2_complete, step1_complete);
    ui.add_space(step_spacing);
    
    render_workflow_step(ui, ui_state, 3, "Scan & Configure", step3_complete, step2_complete);
    ui.add_space(step_spacing);
    
    render_workflow_step(ui, ui_state, 4, "Process Media", false, can_rip);
    
    // Action center
    ui.add_space(StyleConstants::responsive_spacing(screen_width, StyleConstants::SPACING_XXL));
    render_action_center(ui, ui_state, app_state, can_rip);
}

/// Render individual workflow step with enhanced glassmorphism
fn render_workflow_step(ui: &mut egui::Ui, ui_state: &mut UiState, step: u8, title: &str, completed: bool, enabled: bool) {
    let glow = completed || (enabled && !completed);
    let screen_width = ui.available_width();
    let device_type = get_device_type(screen_width);
    
    animated_glass_card(ui, egui::Id::new(format!("step_{}", step)), glow, |ui| {
        ui.horizontal(|ui| {
            // Step indicator with advanced styling
            step_indicator(ui, step, completed, enabled && !completed, 32.0);
            
            let responsive_spacing = StyleConstants::responsive_spacing(screen_width, StyleConstants::SPACING_LG);
            ui.add_space(responsive_spacing);
            
            ui.vertical(|ui| {
                // Step title with enhanced typography
                let title_color = if completed {
                    ModernTheme::NEON_GREEN
                } else if enabled {
                    ModernTheme::TEXT_BRIGHT
                } else {
                    ModernTheme::TEXT_MUTED
                };
                
                let title_size = match device_type {
                    DeviceType::Mobile => 14.0,
                    DeviceType::Tablet => 15.0,
                    _ => 16.0,
                };
                
                ui.colored_label(title_color, 
                    RichText::new(title)
                        .size(title_size)
                        .strong()
                );
                
                ui.add_space(StyleConstants::responsive_spacing(screen_width, StyleConstants::SPACING_SM));
                
                // Step content based on step number
                match step {
                    1 => render_source_selection(ui, ui_state, enabled, device_type),
                    2 => render_output_selection(ui, ui_state, enabled, device_type),
                    3 => render_scan_controls(ui, ui_state, enabled, device_type),
                    4 => render_process_controls(ui, ui_state, enabled, device_type),
                    _ => {}
                }
            });
        });
    });
}

/// Render source selection with responsive modern UI
fn render_source_selection(ui: &mut egui::Ui, ui_state: &mut UiState, enabled: bool, device_type: DeviceType) {
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            ui.set_height(StyleConstants::BUTTON_HEIGHT_MD);
            
            // Responsive text input with glassmorphism
            let input_width = match device_type {
                DeviceType::Mobile => ui.available_width() - 80.0,
                DeviceType::Tablet => (ui.available_width() * 0.7).min(400.0),
                _ => 350.0,
            };
            
            let text_edit = egui::TextEdit::singleline(&mut ui_state.input_path)
                .hint_text("Select DVD drive or folder path...")
                .desired_width(input_width);
            ui.add(text_edit);
            
            ui.add_space(StyleConstants::SPACING_MD);
            
            // Neon browse button
            if ui.add_sized(
                Vec2::new(100.0, StyleConstants::BUTTON_HEIGHT_MD),
                neon_button("Browse", ModernTheme::NEON_BLUE)
            ).clicked() {
                browse_for_input(ui_state);
            }
        });
        
        if !ui_state.input_path.is_empty() {
            ui.add_space(StyleConstants::SPACING_SM);
            ui.horizontal(|ui| {
                // Success checkmark
                ui.painter().circle_filled(
                    ui.next_widget_position() + Vec2::new(6.0, 8.0),
                    4.0,
                    ModernTheme::NEON_GREEN
                );
                ui.add_space(16.0);
                ui.colored_label(ModernTheme::NEON_GREEN, "Source configured");
            });
        }
    });
}

/// Render output selection with enhanced styling
fn render_output_selection(ui: &mut egui::Ui, ui_state: &mut UiState, enabled: bool, device_type: DeviceType) {
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            ui.set_height(StyleConstants::BUTTON_HEIGHT_MD);
            
            let input_width = match device_type {
                DeviceType::Mobile => ui.available_width() - 80.0,
                DeviceType::Tablet => (ui.available_width() * 0.7).min(400.0),
                _ => 350.0,
            };
            
            let text_edit = egui::TextEdit::singleline(&mut ui_state.output_path)
                .hint_text("Choose where to save files...")
                .desired_width(input_width);
            ui.add(text_edit);
            
            ui.add_space(StyleConstants::SPACING_MD);
            
            if ui.add_sized(
                Vec2::new(100.0, StyleConstants::BUTTON_HEIGHT_MD),
                neon_button("Browse", ModernTheme::NEON_CYAN)
            ).clicked() {
                browse_for_output(ui_state);
            }
        });
        
        if !ui_state.output_path.is_empty() {
            ui.add_space(StyleConstants::SPACING_SM);
            ui.horizontal(|ui| {
                ui.painter().circle_filled(
                    ui.next_widget_position() + Vec2::new(6.0, 8.0),
                    4.0,
                    ModernTheme::NEON_GREEN
                );
                ui.add_space(16.0);
                ui.colored_label(ModernTheme::NEON_GREEN, "Output configured");
            });
        }
    });
}

/// Render scan controls with responsive advanced styling
fn render_scan_controls(ui: &mut egui::Ui, ui_state: &mut UiState, enabled: bool, device_type: DeviceType) {
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            if ui.add_sized(
                Vec2::new(140.0, StyleConstants::BUTTON_HEIGHT_LG),
                neon_button("Scan Media", ModernTheme::NEON_PURPLE)
            ).clicked() {
                scan_dvd(ui_state);
            }
            
            if !ui_state.titles.is_empty() {
                ui.add_space(StyleConstants::SPACING_LG);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.painter().circle_filled(
                            ui.next_widget_position() + Vec2::new(6.0, 8.0),
                            4.0,
                            ModernTheme::NEON_GREEN
                        );
                        ui.add_space(16.0);
                        ui.colored_label(ModernTheme::NEON_GREEN, 
                            format!("Found {} titles", ui_state.titles.len())
                        );
                    });
                    
                    ui.colored_label(ModernTheme::TEXT_SECONDARY, 
                        format!("{} selected for processing", ui_state.selected_title_count())
                    );
                });
            }
        });
    });
}

/// Render process controls with responsive stunning effects
fn render_process_controls(ui: &mut egui::Ui, ui_state: &mut UiState, enabled: bool, device_type: DeviceType) {
    ui.add_enabled_ui(enabled, |ui| {
        // Processing options with modern toggles
        ui.horizontal(|ui| {
            ui.checkbox(&mut ui_state.main_feature_only, "Main feature only");
            ui.add_space(StyleConstants::SPACING_LG);
            ui.checkbox(&mut ui_state.chapter_split, "Split chapters");
            ui.add_space(StyleConstants::SPACING_LG);
            ui.checkbox(&mut ui_state.upload_to_server, "Upload to server");
        });
        
        ui.add_space(StyleConstants::SPACING_MD);
        
        // Main action button with glow effect
        if ui.add_sized(
            Vec2::new(180.0, StyleConstants::BUTTON_HEIGHT_XL),
            neon_button("Start Processing", ModernTheme::NEON_GREEN)
        ).clicked() {
            start_ripping(ui_state);
        }
    });
}

/// Render action center with advanced controls
fn render_action_center(ui: &mut egui::Ui, ui_state: &mut UiState, _app_state: Arc<Mutex<AppState>>, can_process: bool) {
    glass_card(ui, true, |ui| {
        tech_section(ui, "Processing Center", Some(ModernTheme::NEON_CYAN), |ui| {
            ui.horizontal(|ui| {
                // Quick actions
                if ui.add_sized(
                    Vec2::new(120.0, StyleConstants::BUTTON_HEIGHT_MD),
                    neon_button("Quick Scan", ModernTheme::NEON_BLUE)
                ).clicked() {
                    scan_dvd(ui_state);
                }
                
                ui.add_space(StyleConstants::SPACING_MD);
                
                if ui.add_sized(
                    Vec2::new(120.0, StyleConstants::BUTTON_HEIGHT_MD),
                    neon_button("Refresh", ModernTheme::NEON_CYAN)
                ).clicked() {
                    refresh_titles(ui_state);
                }
                
                ui.add_space(StyleConstants::SPACING_MD);
                
                if ui.add_sized(
                    Vec2::new(120.0, StyleConstants::BUTTON_HEIGHT_MD),
                    neon_button("Clear", ModernTheme::WARNING)
                ).clicked() {
                    clear_selection(ui_state);
                }
            });
        });
    });
}

/// Browse for DVD source
fn browse_for_source(ui_state: &mut UiState) {
    // TODO: Implement DVD source browsing
}

/// Browse for output directory
fn browse_for_output(ui_state: &mut UiState) {
    // TODO: Implement output directory browsing
}

/// Scan DVD for titles
fn scan_dvd(ui_state: &mut UiState) {
    // TODO: Implement DVD scanning
}

/// Start ripping process
fn start_ripping(ui_state: &mut UiState) {
    // TODO: Implement ripping start
}

/// Refresh title list
fn refresh_titles(ui_state: &mut UiState) {
    // TODO: Implement title refresh
}

/// Clear title selection
fn clear_selection(ui_state: &mut UiState) {
    ui_state.titles.clear();
}

/// Render advanced title selection with glassmorphism
fn render_advanced_title_selection(ui: &mut egui::Ui, ui_state: &mut UiState) {
    glass_card(ui, true, |ui| {
        tech_section(ui, "Media Titles", Some(ModernTheme::NEON_PURPLE), |ui| {
            // Header with stats
            ui.horizontal(|ui| {
                ui.colored_label(ModernTheme::TEXT_BRIGHT, 
                    RichText::new("Select titles to process")
                        .size(14.0)
                );
                
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.colored_label(ModernTheme::NEON_CYAN, 
                        format!("{} / {} selected", 
                            ui_state.selected_title_count(), 
                            ui_state.titles.len()
                        )
                    );
                });
            });
            
            ui.add_space(StyleConstants::SPACING_MD);
            
            // Title list with advanced styling
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for (i, title) in ui_state.titles.iter().enumerate() {
                        if i < ui_state.selected_titles.len() {
                            let selected = ui_state.selected_titles[i];
                            
                            // Title card with glow effect when selected
                            let frame = egui::Frame::none()
                                .fill(if selected { 
                                    ModernTheme::PRIMARY_GLOW 
                                } else { 
                                    ModernTheme::GLASS_SURFACE 
                                })
                                .stroke(if selected {
                                    Stroke::new(1.5, ModernTheme::NEON_BLUE)
                                } else {
                                    Stroke::new(1.0, ModernTheme::BORDER_SUBTLE)
                                })
                                .rounding(Rounding::same(StyleConstants::ROUNDING_MD))
                                .inner_margin(egui::Margin::symmetric(
                                    StyleConstants::SPACING_MD, 
                                    StyleConstants::SPACING_SM
                                ));
                            
                            frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Custom checkbox with neon effect
                                    let checkbox_response = ui.allocate_response(
                                        Vec2::splat(20.0), 
                                        egui::Sense::click()
                                    );
                                    
                                    if checkbox_response.clicked() {
                                        ui_state.selected_titles[i] = !ui_state.selected_titles[i];
                                    }
                                    
                                    let checkbox_rect = checkbox_response.rect;
                                    let checkbox_color = if selected {
                                        ModernTheme::NEON_BLUE
                                    } else {
                                        ModernTheme::BORDER_NORMAL
                                    };
                                    
                                    ui.painter().rect_stroke(
                                        checkbox_rect.shrink(2.0),
                                        Rounding::same(4.0),
                                        Stroke::new(2.0, checkbox_color),
                                    );
                                    
                                    if selected {
                                        ui.painter().rect_filled(
                                            checkbox_rect.shrink(6.0),
                                            Rounding::same(2.0),
                                            ModernTheme::NEON_BLUE,
                                        );
                                    }
                                    
                                    ui.add_space(StyleConstants::SPACING_MD);
                                    
                                    // Title info with enhanced typography
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.colored_label(ModernTheme::TEXT_BRIGHT, 
                                                RichText::new(format!("Title {}", title.number))
                                                    .strong()
                                            );
                                            
                                            if title.number == 1 {
                                                ui.add_space(StyleConstants::SPACING_SM);
                                                ui.colored_label(ModernTheme::NEON_GREEN, 
                                                    RichText::new("MAIN")
                                                        .small()
                                                        .strong()
                                                );
                                            }
                                            
                                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                                ui.colored_label(ModernTheme::TEXT_SECONDARY, 
                                                    format!("{:.1} min", title.duration.as_secs() as f64 / 60.0)
                                                );
                                            });
                                        });
                                        
                                        ui.colored_label(ModernTheme::TEXT_SECONDARY, 
                                            format!("{} chapters", title.chapters.len())
                                        );
                                    });
                                });
                            });
                            
                            if i < ui_state.titles.len() - 1 {
                                ui.add_space(StyleConstants::SPACING_SM);
                            }
                        }
                    }
                });
        });
    });
}

/// Handle browsing for input DVD with error handling
fn browse_for_input(ui_state: &mut UiState) {
    if let Some(path) = rfd::FileDialog::new()
        .set_title("Select DVD Drive or Media Folder")
        .pick_folder()
    {
        ui_state.input_path = path.to_string_lossy().to_string();
        ui_state.clear_error();
    }
}

