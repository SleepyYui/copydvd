use crate::gui::icons::svg_icon;
use crate::gui::state::ui_state::UiState;
use crate::updater::{Release, UpdateProgress, UpdateStatus, Updater};
use eframe::egui::{self, *};
use std::sync::mpsc;
use std::thread;
use tokio::runtime::Handle;

#[derive(Debug, Default)]
pub struct UpdatesTab {
    updater: Option<Updater>,
    pub update_status: Option<UpdateStatus>,
    release_history: Vec<Release>,
    checking_for_updates: bool,
    pub download_progress: Option<UpdateProgress>,
    selected_release: Option<usize>,
    show_release_details: bool,
    auto_check_enabled: bool,
    pub last_check_time: Option<std::time::SystemTime>,
    error_message: Option<String>,
    update_receiver: Option<mpsc::Receiver<UpdateCheckResult>>,
    download_receiver: Option<mpsc::Receiver<UpdateProgress>>,
}

#[derive(Debug)]
enum UpdateCheckResult {
    Status(UpdateStatus),
    Releases(Vec<Release>),
    Error(String),
}

impl UpdatesTab {
    pub fn new() -> Self {
        Self {
            auto_check_enabled: true,
            ..Default::default()
        }
    }

    pub fn show(&mut self, ctx: &Context, _ui_state: &mut UiState) {
        // Check for async results
        self.check_async_results();

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    self.show_header(ui);
                    ui.separator();
                    self.show_current_version(ui);
                    ui.separator();
                    self.show_update_check_section(ui);
                    ui.separator();
                    self.show_release_history(ui);
                });
        });

        // Show popup windows
        if self.show_release_details {
            self.show_release_details_popup(ctx);
        }

        // Auto-check for updates if enabled and it's been a while
        if self.auto_check_enabled && self.should_auto_check() {
            self.check_for_updates_async();
        }
    }

    fn show_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            svg_icon(ui, "refresh", 24.0, Color32::from_gray(120));
            ui.heading("Updates & Release Notes");
        });

        ui.add_space(8.0);
        ui.label("Keep Copy DVD up to date and view release information");
    }

    fn show_current_version(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Current Version:");
            ui.strong(env!("CARGO_PKG_VERSION"));

            if let Some(last_check) = self.last_check_time {
                if let Ok(duration) = last_check.elapsed() {
                    let hours = duration.as_secs() / 3600;
                    if hours < 1 {
                        ui.label("(checked recently)");
                    } else if hours < 24 {
                        ui.label(format!("(checked {} hours ago)", hours));
                    } else {
                        ui.label(format!("(checked {} days ago)", hours / 24));
                    }
                }
            }
        });

        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.auto_check_enabled,
                "Automatically check for updates",
            );
            svg_icon(ui, "refresh", 16.0, Color32::from_gray(120));
            ui.label("")
                .on_hover_text("Enable to automatically check for updates when the app starts");
        });
    }

    fn show_update_check_section(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.checking_for_updates, Button::new("Check for Updates"))
                .clicked()
            {
                self.check_for_updates_async();
            }

            if self.checking_for_updates {
                ui.spinner();
                ui.label("Checking for updates...");
            }
        });

        // Show update status
        if let Some(status) = self.update_status.clone() {
            ui.add_space(8.0);
            match status {
                UpdateStatus::UpToDate => {
                    ui.horizontal(|ui| {
                        svg_icon(ui, "check", 16.0, Color32::from_rgb(0, 150, 0));
                        ui.label("You're running the latest version!");
                    });
                }
                UpdateStatus::UpdateAvailable {
                    current,
                    latest,
                    changelog,
                } => {
                    self.show_update_available(ui, &current, &latest, &changelog);
                }
                UpdateStatus::Error(error) => {
                    ui.horizontal(|ui| {
                        svg_icon(ui, "cross", 16.0, Color32::RED);
                        ui.colored_label(Color32::RED, format!("Error: {}", error));
                    });
                }
            }
        }

        // Show download progress
        if let Some(progress) = &self.download_progress {
            ui.add_space(8.0);
            match progress {
                UpdateProgress::Checking => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Preparing update...");
                    });
                }
                UpdateProgress::Downloading { progress } => {
                    ui.horizontal(|ui| {
                        svg_icon(ui, "arrow-right", 16.0, Color32::from_rgb(0, 100, 200));
                        ui.label("Downloading update:");
                        ui.add(ProgressBar::new(*progress).desired_width(200.0));
                        ui.label(format!("{:.1}%", progress * 100.0));
                    });
                }
                UpdateProgress::Installing => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        svg_icon(ui, "wrench", 16.0, Color32::from_rgb(255, 165, 0));
                        ui.label("Installing update... (this may take a moment)");
                    });
                }
                UpdateProgress::Complete => {
                    ui.horizontal(|ui| {
                        svg_icon(ui, "check", 16.0, Color32::GREEN);
                        ui.colored_label(Color32::GREEN, "Update installed successfully!");
                        ui.label("Restart the application to use the new version.");
                    });
                }
                UpdateProgress::Failed(error) => {
                    ui.horizontal(|ui| {
                        svg_icon(ui, "cross", 16.0, Color32::RED);
                        ui.colored_label(Color32::RED, format!("Update failed: {}", error));
                    });
                }
            }
        }

        if let Some(error) = &self.error_message {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                svg_icon(ui, "warning", 16.0, Color32::from_rgb(255, 165, 0));
                ui.colored_label(Color32::RED, error);
            });
        }
    }

    fn show_update_available(
        &mut self,
        ui: &mut Ui,
        current: &str,
        latest: &Release,
        changelog: &str,
    ) {
        ui.horizontal(|ui| {
            svg_icon(ui, "lightbulb", 16.0, Color32::from_rgb(255, 215, 0));
            ui.strong("New version available!");
        });

        ui.horizontal(|ui| {
            ui.label(format!("Current: {}", current));
            svg_icon(ui, "arrow-right", 12.0, Color32::from_gray(120));
            ui.colored_label(Color32::GREEN, format!("Latest: {}", latest.tag_name));
        });

        ui.horizontal(|ui| {
            if ui.button("View Release Notes").clicked() {
                // Show detailed release notes for this version
                self.show_changelog_popup(latest, changelog);
            }

            if ui
                .add(Button::new("Download & Install").fill(Color32::GREEN))
                .clicked()
            {
                self.start_update(latest.clone());
            }
        });

        // Show brief summary
        if !latest.body.is_empty() {
            ui.add_space(4.0);
            let summary = self.extract_summary(&latest.body);
            ui.label(format!("What's new: {}", summary));
        }
    }

    fn show_release_history(&mut self, ui: &mut Ui) {
        ui.heading("Release History");
        ui.add_space(8.0);

        if self.release_history.is_empty() {
            if ui.button("Load Release History").clicked() {
                self.load_release_history_async();
            }
        } else {
            self.show_release_list(ui);
        }
    }

    fn show_release_list(&mut self, ui: &mut Ui) {
        let current_version = env!("CARGO_PKG_VERSION");

        for (index, release) in self.release_history.iter().enumerate() {
            let is_current = release.tag_name.trim_start_matches('v') == current_version;
            let is_newer = self.is_version_newer(&release.tag_name, current_version);

            ui.horizontal(|ui| {
                // Version badge
                if is_current {
                    ui.colored_label(Color32::GREEN, "CURRENT");
                } else if is_newer {
                    ui.colored_label(Color32::BLUE, "NEWER");
                } else {
                    ui.colored_label(Color32::GRAY, "OLDER");
                }

                // Release info
                ui.strong(&release.name);
                ui.label(format!("({})", release.tag_name));

                // Published date
                if let Ok(date) = chrono::DateTime::parse_from_rfc3339(&release.published_at) {
                    ui.label(format!("- {}", date.format("%Y-%m-%d")));
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.small_button("View Details").clicked() {
                        self.selected_release = Some(index);
                        self.show_release_details = true;
                    }
                });
            });

            // Show brief description
            if !release.body.is_empty() {
                let summary = self.extract_summary(&release.body);
                ui.indent(
                    format!("release_summary_{}_{}", release.tag_name, index),
                    |ui| {
                        ui.label(summary);
                    },
                );
            }

            ui.add_space(4.0);
        }
    }

    fn show_release_details_popup(&mut self, ctx: &Context) {
        if let Some(index) = self.selected_release {
            if let Some(release) = self.release_history.get(index).cloned() {
                let window_title = format!("Release Notes - {}", release.name);
                let version_newer =
                    self.is_version_newer(&release.tag_name, env!("CARGO_PKG_VERSION"));

                Window::new(window_title)
                    .default_size([600.0, 500.0])
                    .resizable(true)
                    .show(ctx, |ui| {
                        ScrollArea::vertical().show(ui, |ui| {
                            self.show_detailed_release_notes(ui, &release);
                        });

                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Close").clicked() {
                                self.show_release_details = false;
                                self.selected_release = None;
                            }

                            if version_newer
                                && ui
                                    .add(Button::new("Update to This Version").fill(Color32::GREEN))
                                    .clicked()
                            {
                                self.start_update(release.clone());
                                self.show_release_details = false;
                                self.selected_release = None;
                            }
                        });
                    });
            }
        }
    }

    fn show_detailed_release_notes(&self, ui: &mut Ui, release: &Release) {
        ui.heading(&release.name);
        ui.label(format!("Version: {}", release.tag_name));

        if let Ok(date) = chrono::DateTime::parse_from_rfc3339(&release.published_at) {
            ui.label(format!(
                "Released: {}",
                date.format("%B %d, %Y at %H:%M UTC")
            ));
        }

        ui.add_space(16.0);

        // Parse and display markdown content
        self.show_markdown_content(ui, &release.body);

        // Show assets if any
        if !release.assets.is_empty() {
            ui.add_space(16.0);
            ui.heading("Downloads");
            for asset in &release.assets {
                ui.horizontal(|ui| {
                    svg_icon(ui, "info", 16.0, Color32::from_gray(120));
                    if ui.link(&asset.name).clicked() {
                        if let Err(e) = open::that(&asset.download_url) {
                            eprintln!("Failed to open download link: {}", e);
                        }
                    }
                    ui.label(format!("({} MB)", asset.size / 1024 / 1024));
                });
            }
        }
    }

    fn show_markdown_content(&self, ui: &mut Ui, content: &str) {
        // Simple markdown parsing for release notes
        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("## ") {
                ui.add_space(8.0);
                ui.heading(line.trim_start_matches("## "));
            } else if line.starts_with("### ") {
                ui.add_space(4.0);
                ui.strong(line.trim_start_matches("### "));
            } else if line.starts_with("- ") {
                ui.horizontal(|ui| {
                    ui.label("  •");
                    ui.label(line.trim_start_matches("- "));
                });
            } else if line.starts_with("* ") {
                ui.horizontal(|ui| {
                    ui.label("  •");
                    ui.label(line.trim_start_matches("* "));
                });
            } else if !line.is_empty() {
                ui.label(line);
            } else {
                ui.add_space(4.0);
            }
        }
    }

    fn show_changelog_popup(&self, _release: &Release, _changelog: &str) {
        // This would open a detailed changelog window
        // Implementation similar to show_release_details_popup
    }

    fn extract_summary(&self, body: &str) -> String {
        // Extract first meaningful line or first bullet point
        for line in body.lines() {
            let line = line.trim();
            if line.starts_with("- ") || line.starts_with("* ") {
                let summary = line.trim_start_matches("- ").trim_start_matches("* ");
                if summary.len() > 10 {
                    return if summary.len() > 80 {
                        format!("{}...", &summary[..77])
                    } else {
                        summary.to_string()
                    };
                }
            } else if !line.is_empty() && !line.starts_with("#") && line.len() > 10 {
                return if line.len() > 80 {
                    format!("{}...", &line[..77])
                } else {
                    line.to_string()
                };
            }
        }
        "No description available".to_string()
    }

    fn is_version_newer(&self, version1: &str, version2: &str) -> bool {
        let v1 = version1.trim_start_matches('v');
        let v2 = version2.trim_start_matches('v');

        match (semver::Version::parse(v1), semver::Version::parse(v2)) {
            (Ok(ver1), Ok(ver2)) => ver1 > ver2,
            _ => false,
        }
    }

    fn should_auto_check(&self) -> bool {
        if let Some(last_check) = self.last_check_time {
            if let Ok(duration) = last_check.elapsed() {
                duration.as_secs() > 3600 // Check every hour
            } else {
                true
            }
        } else {
            true // Never checked before
        }
    }

    fn check_async_results(&mut self) {
        // Check for update check results
        if let Some(receiver) = &self.update_receiver {
            while let Ok(result) = receiver.try_recv() {
                match result {
                    UpdateCheckResult::Status(status) => {
                        self.update_status = Some(status);
                        self.checking_for_updates = false;
                    }
                    UpdateCheckResult::Releases(releases) => {
                        self.release_history = releases;
                    }
                    UpdateCheckResult::Error(error) => {
                        self.error_message = Some(error);
                        self.checking_for_updates = false;
                    }
                }
            }
        }

        // Check for download progress
        if let Some(receiver) = &self.download_receiver {
            while let Ok(progress) = receiver.try_recv() {
                self.download_progress = Some(progress);
            }
        }
    }

    fn check_for_updates_async(&mut self) {
        if self.checking_for_updates {
            return;
        }

        self.checking_for_updates = true;
        self.update_status = None;
        self.error_message = None;

        if self.updater.is_none() {
            self.updater = Some(Updater::new());
        }

        // Set up channel for receiving results
        let (sender, receiver) = mpsc::channel();
        self.update_receiver = Some(receiver);

        let updater = Updater::new();

        // Spawn async task
        thread::spawn(move || {
            if let Ok(handle) = Handle::try_current() {
                handle.spawn(async move {
                    let status = updater.check_for_updates().await;
                    let _ = sender.send(UpdateCheckResult::Status(status));
                });
            } else {
                // Fallback to blocking operation
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let status = updater.check_for_updates().await;
                    let _ = sender.send(UpdateCheckResult::Status(status));
                });
            }
        });

        self.last_check_time = Some(std::time::SystemTime::now());
    }

    fn load_release_history_async(&mut self) {
        if self.updater.is_none() {
            self.updater = Some(Updater::new());
        }

        // Set up channel for receiving results
        let (sender, receiver) = mpsc::channel();
        self.update_receiver = Some(receiver);

        let updater = Updater::new();

        thread::spawn(move || {
            if let Ok(handle) = Handle::try_current() {
                handle.spawn(async move {
                    match updater.get_release_history().await {
                        Ok(releases) => {
                            let _ = sender.send(UpdateCheckResult::Releases(releases));
                        }
                        Err(e) => {
                            let _ = sender.send(UpdateCheckResult::Error(format!(
                                "Failed to load releases: {}",
                                e
                            )));
                        }
                    }
                });
            } else {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    match updater.get_release_history().await {
                        Ok(releases) => {
                            let _ = sender.send(UpdateCheckResult::Releases(releases));
                        }
                        Err(e) => {
                            let _ = sender.send(UpdateCheckResult::Error(format!(
                                "Failed to load releases: {}",
                                e
                            )));
                        }
                    }
                });
            }
        });
    }

    fn start_update(&mut self, release: Release) {
        if self.updater.is_none() {
            self.updater = Some(Updater::new());
        }

        self.download_progress = Some(UpdateProgress::Checking);

        // Set up progress channel
        let (progress_sender, progress_receiver) = mpsc::channel();
        self.download_receiver = Some(progress_receiver);

        let updater = Updater::new();

        thread::spawn(move || {
            if let Ok(handle) = Handle::try_current() {
                handle.spawn(async move {
                    let progress_callback = move |progress: UpdateProgress| {
                        let _ = progress_sender.send(progress);
                    };

                    let _ = updater
                        .download_and_install_update(&release, progress_callback)
                        .await;
                });
            } else {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let progress_callback = move |progress: UpdateProgress| {
                        let _ = progress_sender.send(progress);
                    };

                    let _ = updater
                        .download_and_install_update(&release, progress_callback)
                        .await;
                });
            }
        });
    }
}
