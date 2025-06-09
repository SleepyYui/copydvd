use crate::gui::state::ui_state::{ToastNotification, ToastType, UiState};
use crate::updater::{Release, UpdateStatus};
use std::time::SystemTime;

#[derive(Default)]
pub struct UpdateNotificationManager {
    last_notification_time: Option<SystemTime>,
    update_available_notified: bool,
}

impl UpdateNotificationManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_and_notify(&mut self, ui_state: &mut UiState) {
        // Check for update status changes and show appropriate notifications
        if let Some(status) = ui_state.updates_tab.update_status.clone() {
            match status {
                UpdateStatus::UpdateAvailable { latest, .. } => {
                    if !self.update_available_notified {
                        self.show_update_available_notification(ui_state, &latest);
                        self.update_available_notified = true;
                        self.last_notification_time = Some(SystemTime::now());
                    }
                }
                UpdateStatus::UpToDate => {
                    // Only notify if we just checked and user might be interested
                    if let Some(last_check) = ui_state.updates_tab.last_check_time {
                        if let Some(last_notification) = self.last_notification_time {
                            // Only show if it's been less than 30 seconds since last check
                            // and we haven't notified recently
                            if let (Ok(check_duration), Ok(notification_duration)) =
                                (last_check.elapsed(), last_notification.elapsed())
                            {
                                if check_duration.as_secs() < 30
                                    && notification_duration.as_secs() > 300
                                {
                                    self.show_up_to_date_notification(ui_state);
                                    self.last_notification_time = Some(SystemTime::now());
                                }
                            }
                        } else {
                            // First time checking
                            if let Ok(duration) = last_check.elapsed() {
                                if duration.as_secs() < 30 {
                                    self.show_up_to_date_notification(ui_state);
                                    self.last_notification_time = Some(SystemTime::now());
                                }
                            }
                        }
                    }
                    // Reset the update available flag when we're up to date
                    self.update_available_notified = false;
                }
                UpdateStatus::Error(_) => {
                    // Don't spam error notifications
                    if let Some(last_notification) = self.last_notification_time {
                        if let Ok(duration) = last_notification.elapsed() {
                            if duration.as_secs() > 300 {
                                // 5 minutes
                                self.show_update_error_notification(ui_state);
                                self.last_notification_time = Some(SystemTime::now());
                            }
                        }
                    } else {
                        self.show_update_error_notification(ui_state);
                        self.last_notification_time = Some(SystemTime::now());
                    }
                }
            }
        }

        // Check for download progress and show completion notifications
        if let Some(progress) = ui_state.updates_tab.download_progress.clone() {
            match progress {
                crate::updater::UpdateProgress::Complete => {
                    self.show_update_complete_notification(ui_state);
                }
                crate::updater::UpdateProgress::Failed(error) => {
                    self.show_update_failed_notification(ui_state, &error);
                }
                _ => {}
            }
        }
    }

    fn show_update_available_notification(&self, ui_state: &mut UiState, release: &Release) {
        let message = format!(
            "Update Available: {}. Click Updates tab to install.",
            release.tag_name
        );

        ui_state.add_toast(ToastNotification::new(message, ToastType::Info));
    }

    fn show_up_to_date_notification(&self, ui_state: &mut UiState) {
        ui_state.add_toast(ToastNotification::new(
            "You're running the latest version of Copy DVD.".to_string(),
            ToastType::Success,
        ));
    }

    fn show_update_error_notification(&self, ui_state: &mut UiState) {
        ui_state.add_toast(ToastNotification::new(
            "Unable to check for updates. Please check your internet connection.".to_string(),
            ToastType::Error,
        ));
    }

    fn show_update_complete_notification(&self, ui_state: &mut UiState) {
        ui_state.add_toast(ToastNotification::new(
            "Update has been installed successfully. Restart to use the new version.".to_string(),
            ToastType::Success,
        ));
    }

    fn show_update_failed_notification(&self, ui_state: &mut UiState, error: &str) {
        let message = format!("Update installation failed: {}", error);

        ui_state.add_toast(ToastNotification::new(message, ToastType::Error));
    }

    #[allow(dead_code)]
    pub fn show_restart_prompt(&self, ui_state: &mut UiState) {
        ui_state.add_toast(ToastNotification::new(
            "Please restart Copy DVD to complete the update process.".to_string(),
            ToastType::Info,
        ));
    }

    #[allow(dead_code)]
    pub fn reset_notifications(&mut self) {
        self.update_available_notified = false;
        self.last_notification_time = None;
    }
}
