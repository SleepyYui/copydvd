use crate::error::{AppError, Result};
use std::path::PathBuf;

// This module could contain more specific DVD detection logic,
// for example, verifying if a given path is indeed a DVD structure.
// The general auto_detect_dvd is currently in app/startup.rs

pub fn verify_dvd_path(path: &PathBuf) -> Result<()> {
    if path.join("VIDEO_TS").is_dir() {
        Ok(())
    } else {
        Err(AppError::DvdNotFound(path.clone()))
    }
}
