use std::path::PathBuf;
use thiserror::Error;

/// Custom error types for the Copy DVD application
#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("DVD not found at path: {0}")]
    DvdNotFound(PathBuf),

    #[error("HandBrake CLI not found. Please install HandBrake.")]
    #[allow(dead_code)]
    HandBrakeNotFound,

    #[error("Failed to parse DVD structure: {0}")]
    #[allow(dead_code)]
    DvdParseFailed(String),

    #[error("Failed to execute command: {0}")]
    #[allow(dead_code)]
    CommandFailed(String),

    #[error("Mount failed: {0}")]
    #[allow(dead_code)]
    MountFailed(String),

    #[error("No valid titles found on DVD")]
    #[allow(dead_code)]
    NoTitlesFound,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Output file already exists: {0}")]
    #[allow(dead_code)]
    OutputExists(PathBuf),

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("GUI error: {0}")]
    #[allow(dead_code)]
    GuiError(String),

    #[error("User error: {0}")]
    #[allow(dead_code)]
    UserError(String),

    #[error("Unknown error: {0}")]
    #[allow(dead_code)]
    Unknown(String),

    #[error("HandBrake error: {0}")]
    HandbrakeError(String),

    #[error("DVD operation error: {0}")]
    DvdError(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
