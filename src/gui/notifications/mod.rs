use notify_rust::{Notification, Timeout};

pub struct OSNotifications;

impl OSNotifications {
    pub fn success(title: &str, message: &str) {
        tracing::info!(
            "Attempting to show success notification: {} - {}",
            title,
            message
        );
        match Notification::new()
            .summary(title)
            .body(message)
            .icon("resources/icon.svg")
            .timeout(Timeout::Milliseconds(5000))
            .show()
        {
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
        match Notification::new()
            .summary(title)
            .body(message)
            .icon("resources/icon.svg")
            .timeout(Timeout::Milliseconds(8000))
            .show()
        {
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
        match Notification::new()
            .summary(title)
            .body(message)
            .icon("resources/icon.svg")
            .timeout(Timeout::Milliseconds(6000))
            .show()
        {
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
        match Notification::new()
            .summary(title)
            .body(message)
            .icon("resources/icon.svg")
            .timeout(Timeout::Milliseconds(4000))
            .show()
        {
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
    OSNotifications::success("Copy DVD", message);
}

pub fn notify_error(message: &str) {
    OSNotifications::error("Copy DVD - Error", message);
}

#[allow(dead_code)]
pub fn notify_warning(message: &str) {
    OSNotifications::warning("Copy DVD - Warning", message);
}

pub fn notify_info(message: &str) {
    OSNotifications::info("Copy DVD", message);
}
