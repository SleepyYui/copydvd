use crate::error::{AppError, Result};
use std::path::Path;

// This module could contain more specific DVD detection logic,
// for example, verifying if a given path is indeed a DVD structure.
// The general auto_detect_dvd is currently in app/startup.rs

#[allow(dead_code)]
pub fn verify_dvd_path(path: &Path) -> Result<()> {
    if path.join("VIDEO_TS").is_dir() {
        Ok(())
    } else {
        Err(AppError::DvdNotFound(path.into()))
    }
}
