use crate::error::{AppError, Result};
use std::path::Path;
use std::process::Command;
use tracing::{info, warn};

/// Automatic macOS security fixes for HandBrake
#[allow(dead_code)]
pub struct MacOSAutoFix;

impl MacOSAutoFix {
    /// Attempt all automatic fixes for macOS HandBrake security issues
    #[allow(dead_code)]
    pub fn attempt_all_fixes(binary_path: &Path) -> Result<bool> {
        info!("Attempting automatic macOS security fixes for HandBrake...");

        let mut fixes_applied = false;

        // 1. Remove quarantine attributes
        if Self::remove_quarantine_attributes(binary_path)? {
            info!("Successfully removed quarantine attributes");
            fixes_applied = true;
        }

        // 2. Set executable permissions
        if Self::set_executable_permissions(binary_path)? {
            info!("Set executable permissions");
            fixes_applied = true;
        }

        // 3. Open Security preferences to help user
        if Self::open_security_preferences()? {
            info!("Opened Security preferences");
            fixes_applied = true;
        }

        Ok(fixes_applied)
    }

    /// Remove quarantine attributes from the binary
    #[allow(dead_code)]
    fn remove_quarantine_attributes(binary_path: &Path) -> Result<bool> {
        let quarantine_attrs = [
            "com.apple.quarantine",
            "com.apple.metadata:kMDItemWhereFroms",
            "com.apple.metadata:kMDItemDownloadedDate",
        ];

        let mut any_removed = false;

        for attr in &quarantine_attrs {
            match Command::new("xattr")
                .args(["-d", attr])
                .arg(binary_path)
                .output()
            {
                Ok(result) => {
                    if result.status.success() {
                        info!("Removed quarantine attribute: {}", attr);
                        any_removed = true;
                    }
                }
                Err(e) => {
                    warn!("Failed to remove {}: {}", attr, e);
                }
            }
        }

        Ok(any_removed)
    }

    /// Set executable permissions on the binary
    #[allow(dead_code)]
    fn set_executable_permissions(binary_path: &Path) -> Result<bool> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let metadata = std::fs::metadata(binary_path)
                .map_err(|e| AppError::HandbrakeError(format!("Failed to get metadata: {}", e)))?;

            let mut permissions = metadata.permissions();
            let current_mode = permissions.mode();
            let new_mode = current_mode | 0o755;

            if current_mode != new_mode {
                permissions.set_mode(new_mode);
                std::fs::set_permissions(binary_path, permissions).map_err(|e| {
                    AppError::HandbrakeError(format!("Failed to set permissions: {}", e))
                })?;
                info!("Set executable permissions (mode: {:o})", new_mode);
                Ok(true)
            } else {
                Ok(false)
            }
        }

        #[cfg(not(unix))]
        {
            // On Windows, executable permissions are not needed
            info!("Skipping executable permissions on Windows");
            Ok(false)
        }
    }

    /// Open Security & Privacy preferences
    #[allow(dead_code)]
    fn open_security_preferences() -> Result<bool> {
        let methods = [
            (
                "open",
                vec!["/System/Library/PreferencePanes/Security.prefPane"],
            ),
            ("open", vec!["-a", "System Preferences"]),
            ("open", vec!["-a", "System Settings"]),
        ];

        for (cmd, args) in &methods {
            if Command::new(cmd).args(args).spawn().is_ok() {
                info!("Opened Security preferences using: {} {:?}", cmd, args);
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if the binary has quarantine attributes
    #[allow(dead_code)]
    pub fn has_quarantine_attribute(binary_path: &Path) -> bool {
        Command::new("xattr")
            .args(["-l"])
            .arg(binary_path)
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).contains("com.apple.quarantine"))
            .unwrap_or(false)
    }
}
