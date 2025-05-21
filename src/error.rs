use thiserror::Error;
use std::path::PathBuf;

/// Custom error types for the DVD ripper application
#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("DVD not found at path: {0}")]
    DvdNotFound(PathBuf),

    #[error("HandBrake CLI not found. Please install HandBrakeCLI.")]
    HandBrakeNotFound,

    #[error("Failed to parse DVD structure: {0}")]
    DvdParseFailed(String),

    #[error("Failed to execute command: {0}")]
    CommandFailed(String),

    #[error("Mount failed: {0}")]
    MountFailed(String),

    #[error("No titles found on DVD")]
    NoTitlesFound,

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Output file already exists: {0}")]
    OutputExists(PathBuf),

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("GUI error: {0}")]
    GuiError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, AppError>;