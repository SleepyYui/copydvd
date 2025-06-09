use crate::gui::icons::svg_icon;
use eframe::egui::*;

#[derive(Default)]
pub struct ExitConfirmationDialog {
    pub show: bool,
    pub confirmed: bool,
}

impl ExitConfirmationDialog {
    pub fn show_dialog(
        &mut self,
        ctx: &Context,
        has_active_tasks: bool,
        pending_updates: bool,
        has_handbrake_operations: bool,
        has_update_operations: bool,
    ) -> bool {
        if !self.show {
            return false;
        }

        let mut should_close = false;

        Window::new("Confirm Exit")
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    if has_active_tasks {
                        ui.horizontal(|ui| {
                            svg_icon(ui, "warning", 20.0, Color32::from_rgb(255, 165, 0));
                            ui.heading("DVD Operations in Progress");
                        });

                        ui.add_space(8.0);
                        ui.label("DVD ripping or upload tasks are currently running.");
                        ui.label("Closing now will interrupt these operations and may result in incomplete files.");

                        ui.add_space(16.0);
                        ui.columns(2, |columns| {
                            columns[0].with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                if ui.add_sized([120.0, 32.0], Button::new("Keep Running").fill(Color32::from_rgb(50, 150, 50))).clicked() {
                                    self.show = false;
                                    should_close = false;
                                }
                            });columns[1].with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                if ui.add_sized([120.0, 32.0], Button::new("Force Exit").fill(Color32::from_rgb(200, 50, 50))).clicked() {
                                    self.confirmed = true;
                                    self.show = false;
                                    should_close = true;
                                }
                            });
                        });
                    } else if has_handbrake_operations {
                        ui.horizontal(|ui| {
                            svg_icon(ui, "wrench", 20.0, Color32::from_rgb(255, 165, 0));
                            ui.heading("HandBrake Operations in Progress");
                        });

                        ui.add_space(8.0);
                        ui.label("HandBrake is currently being downloaded, installed, or verified.");
                        ui.label("Closing now will interrupt this process.");

                        ui.add_space(16.0);
                        ui.columns(2, |columns| {
                            columns[0].with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                if ui.add_sized([120.0, 32.0], Button::new("Wait").fill(Color32::from_rgb(50, 150, 50))).clicked() {
                                    self.show = false;
                                    should_close = false;
                                }
                            });
                            columns[1].with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                if ui.add_sized([120.0, 32.0], Button::new("Force Exit").fill(Color32::from_rgb(200, 50, 50))).clicked() {
                                    self.confirmed = true;
                                    self.show = false;
                                    should_close = true;
                                }
                            });
                        });
                    } else if has_update_operations {
                        ui.horizontal(|ui| {
                            svg_icon(ui, "refresh", 20.0, Color32::from_rgb(0, 150, 200));
                            ui.heading("App Update in Progress");
                        });

                        ui.add_space(8.0);
                        ui.label("An application update is currently being downloaded or installed.");
                        ui.label("Closing now will interrupt the update process.");

                        ui.add_space(16.0);
                        ui.columns(2, |columns| {
                            columns[0].with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                if ui.add_sized([120.0, 32.0], Button::new("Wait").fill(Color32::from_rgb(50, 150, 50))).clicked() {
                                    self.show = false;
                                    should_close = false;
                                }
                            });
                            columns[1].with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                if ui.add_sized([120.0, 32.0], Button::new("Force Exit").fill(Color32::from_rgb(200, 50, 50))).clicked() {
                                    self.confirmed = true;
                                    self.show = false;
                                    should_close = true;
                                }
                            });
                        });
                    } else if pending_updates {
                        ui.horizontal(|ui| {
                            svg_icon(ui, "refresh", 20.0, Color32::from_rgb(0, 150, 200));
                            ui.heading("Update Ready");
                        });

                        ui.add_space(8.0);
                        ui.label("An update has been downloaded and is ready to install.");
                        ui.label("Would you like to restart now to complete the update?");

                        ui.add_space(16.0);
                        ui.columns(2, |columns| {
                            columns[0].with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                if ui.add_sized([120.0, 32.0], Button::new("Exit Without Update").fill(Color32::from_rgb(150, 150, 150))).clicked() {
                                    self.confirmed = true;
                                    self.show = false;
                                    should_close = true;
                                }
                            });
                            columns[1].with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                if ui.add_sized([120.0, 32.0], Button::new("Restart & Update").fill(Color32::from_rgb(50, 150, 50))).clicked() {
                                    // This would trigger the restart and update process
                                    self.restart_and_update();
                                    self.show = false;
                                    should_close = true;
                                }
                            });
                        });
                    } else {
                        ui.horizontal(|ui| {
                            svg_icon(ui, "info", 20.0, Color32::from_rgb(100, 150, 200));
                            ui.heading("Exit Copy DVD?");
                        });

                        ui.add_space(8.0);
                        ui.label("Are you sure you want to exit the application?");

                        ui.add_space(16.0);
                        ui.columns(2, |columns| {
                            columns[0].with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                if ui.add_sized([120.0, 32.0], Button::new("Cancel").fill(Color32::from_rgb(50, 150, 50))).clicked() {
                                    self.show = false;
                                    should_close = false;
                                }
                            });
                            columns[1].with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                                if ui.add_sized([120.0, 32.0], Button::new("Exit").fill(Color32::from_rgb(150, 150, 150))).clicked() {
                                    self.confirmed = true;
                                    self.show = false;
                                    should_close = true;
                                }
                            });
                        });
                    }

                    ui.add_space(10.0);
                });
            });

        should_close
    }

    pub fn request_exit(&mut self) {
        self.show = true;
        self.confirmed = false;
    }

    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.show = false;
        self.confirmed = false;
    }

    #[cfg(target_os = "windows")]
    fn restart_and_update(&self) {
        // This would integrate with the updater to restart and apply updates
        tokio::spawn(async {
            if let Ok(updater) =
                tokio::task::spawn_blocking(|| crate::updater::Updater::new()).await
            {
                let _ = updater.execute_pending_update_and_restart().await;
            }
        });
    }

    #[cfg(not(target_os = "windows"))]
    fn restart_and_update(&self) {
        // On macOS/Linux, just exit - the user will manually restart
        std::process::exit(0);
    }
}

pub fn has_active_ripping_tasks(app_state: &crate::app::state::AppState) -> bool {
    // Check if any DVD ripping tasks are currently running
    matches!(
        app_state.status,
        crate::app::state::AppStatus::Scanning
            | crate::app::state::AppStatus::Ripping { .. }
            | crate::app::state::AppStatus::Uploading { .. }
    )
}

pub fn has_pending_updates() -> bool {
    // Check if there's a pending update script on Windows
    #[cfg(target_os = "windows")]
    {
        if let Ok(current_exe) = std::env::current_exe() {
            let update_script_path = current_exe.with_file_name("update_copydvd.bat");
            return update_script_path.exists();
        }
    }

    false
}
