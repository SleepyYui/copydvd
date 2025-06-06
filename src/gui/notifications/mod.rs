use notify_rust::{Notification, Timeout};

pub struct OSNotifications;

// Embed the icon directly in the binary
const ICON_DATA: &[u8] = include_bytes!("../../../resources/icons/icon-64.png");

fn get_notification_with_icon(title: &str, body: &str, timeout_ms: u32) -> Notification {
    let mut notification = Notification::new();
    notification
        .summary(title)
        .body(body)
        .timeout(Timeout::Milliseconds(timeout_ms));

    // Try to create a persistent temp file for the icon
    let temp_path = std::env::temp_dir().join(format!("copydvd_icon_{}.png", std::process::id()));
    if !temp_path.exists() {
        if let Ok(mut file) = std::fs::File::create(&temp_path) {
            use std::io::Write;
            if file.write_all(ICON_DATA).is_ok() && file.sync_all().is_ok() {
                tracing::info!("Created persistent icon at: {:?}", temp_path);
                notification.icon(&temp_path.to_string_lossy());
                return notification;
            }
        }
        tracing::error!("Failed to create icon file");
    } else {
        // File exists, just use it
        notification.icon(&temp_path.to_string_lossy());
        return notification;
    }

    // Fallback: try to use a data URI (may not work on all platforms)
    use base64::{engine::general_purpose, Engine};
    let base64_icon = general_purpose::STANDARD.encode(ICON_DATA);
    let data_uri = format!("data:image/png;base64,{}", base64_icon);
    notification.icon(&data_uri);

    notification
}

impl OSNotifications {
    pub fn success(title: &str, message: &str) {
        tracing::info!(
            "Attempting to show success notification: {} - {}",
            title,
            message
        );

        match get_notification_with_icon(title, message, 5000).show() {
            Ok(handle) => {
                tracing::info!("Success notification sent successfully: {:?}", handle);
            }
            Err(e) => {
                tracing::error!("Failed to show success notification: {}", e);
            }
        }
    }

    pub fn error(title: &str, message: &str) {
        tracing::info!(
            "Attempting to show error notification: {} - {}",
            title,
            message
        );

        match get_notification_with_icon(title, message, 8000).show() {
            Ok(handle) => {
                tracing::info!("Error notification sent successfully: {:?}", handle);
            }
            Err(e) => {
                tracing::error!("Failed to show error notification: {}", e);
            }
        }
    }

    #[allow(dead_code)]
    pub fn warning(title: &str, message: &str) {
        tracing::info!(
            "Attempting to show warning notification: {} - {}",
            title,
            message
        );

        match get_notification_with_icon(title, message, 6000).show() {
            Ok(handle) => {
                tracing::info!("Warning notification sent successfully: {:?}", handle);
            }
            Err(e) => {
                tracing::error!("Failed to show warning notification: {}", e);
            }
        }
    }

    pub fn info(title: &str, message: &str) {
        tracing::info!(
            "Attempting to show info notification: {} - {}",
            title,
            message
        );

        match get_notification_with_icon(title, message, 4000).show() {
            Ok(handle) => {
                tracing::info!("Info notification sent successfully: {:?}", handle);
            }
            Err(e) => {
                tracing::error!("Failed to show info notification: {}", e);
            }
        }
    }
}

// Convenience functions
pub fn notify_success(message: &str) {
    OSNotifications::success("CopyDVD", message);
}

pub fn notify_error(message: &str) {
    OSNotifications::error("CopyDVD - Error", message);
}

#[allow(dead_code)]
pub fn notify_warning(message: &str) {
    OSNotifications::warning("CopyDVD - Warning", message);
}

pub fn notify_info(message: &str) {
    OSNotifications::info("CopyDVD", message);
}
