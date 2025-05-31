use notify_rust::{Notification, Timeout};

pub struct OSNotifications;

impl OSNotifications {
    pub fn success(title: &str, message: &str) {
        let _ = Notification::new()
            .summary(title)
            .body(message)
            .timeout(Timeout::Milliseconds(5000))
            .show();
    }
    
    pub fn error(title: &str, message: &str) {
        let _ = Notification::new()
            .summary(title)
            .body(message)
            .timeout(Timeout::Milliseconds(8000))
            .show();
    }
    
    pub fn warning(title: &str, message: &str) {
        let _ = Notification::new()
            .summary(title)
            .body(message)
            .timeout(Timeout::Milliseconds(6000))
            .show();
    }
    
    pub fn info(title: &str, message: &str) {
        let _ = Notification::new()
            .summary(title)
            .body(message)
            .timeout(Timeout::Milliseconds(4000))
            .show();
    }
}

// Convenience functions
pub fn notify_success(message: &str) {
    OSNotifications::success("Copy DVD", message);
}

pub fn notify_error(message: &str) {
    OSNotifications::error("Copy DVD - Error", message);
}

pub fn notify_warning(message: &str) {
    OSNotifications::warning("Copy DVD - Warning", message);
}

pub fn notify_info(message: &str) {
    OSNotifications::info("Copy DVD", message);
}