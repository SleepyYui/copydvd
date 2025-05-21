use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{info, warn};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::error::{AppError, Result};
use crate::config::ServerConfig;

/// Represents an upload task
#[derive(Debug, Clone)]
pub struct UploadTask {
    pub local_path: PathBuf,
    pub remote_path: String,
    pub server_config: ServerConfig,
}

/// Upload a file to a remote server
pub async fn upload_file(task: &UploadTask) -> Result<()> {
    info!("Uploading file: {} to {}:{}", 
          task.local_path.display(), 
          task.server_config.host,
          task.remote_path);
    
    // Ensure the file exists
    if !task.local_path.exists() {
        return Err(AppError::UploadFailed(format!(
            "File not found: {}", task.local_path.display()
        )));
    }
    
    // Check which upload method to use based on OS and configuration
    match determine_upload_method(&task.server_config) {
        UploadMethod::Rsync => upload_with_rsync(task).await,
        UploadMethod::Scp => upload_with_scp(task).await,
        UploadMethod::Sftp => upload_with_sftp(task).await,
    }
}

/// Upload methods
enum UploadMethod {
    Rsync,
    Scp,
    Sftp,
}

/// Determine the best upload method based on the OS and available tools
fn determine_upload_method(server_config: &ServerConfig) -> UploadMethod {
    // Default to rsync if available, otherwise fall back to scp
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // Check if rsync is available
        if std::process::Command::new("rsync").arg("--version").output().is_ok() {
            return UploadMethod::Rsync;
        }
    }
    
    // Check if scp is available as a fallback
    if std::process::Command::new("scp").arg("-V").output().is_ok() {
        return UploadMethod::Scp;
    }
    
    // Default to SFTP (we'll implement this using a library)
    UploadMethod::Sftp
}

/// Upload a file using rsync
async fn upload_with_rsync(task: &UploadTask) -> Result<()> {
    let password = task.server_config.password.as_deref().unwrap_or("");
    let username = &task.server_config.username;
    let host = &task.server_config.host;
    
    let remote_path = format!("{}@{}:{}", username, host, task.remote_path);
    
    // Use sshpass if password is provided
    let mut command = if !password.is_empty() {
        let mut cmd = Command::new("sshpass");
        cmd.arg("-p")
            .arg(password)
            .arg("rsync")
            .arg("-avz")
            .arg("--progress");
        cmd
    } else {
        let mut cmd = Command::new("rsync");
        cmd.arg("-avz")
            .arg("--progress");
        cmd
    };
    
    // Add the local and remote paths
    command.arg(&task.local_path)
           .arg(remote_path);
    
    // Execute the command
    let output = command.output().await
        .map_err(|e| AppError::UploadFailed(format!("Failed to execute rsync: {}", e)))?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::UploadFailed(format!("rsync failed: {}", error)));
    }
    
    info!("File uploaded successfully with rsync");
    Ok(())
}

/// Upload a file using scp
async fn upload_with_scp(task: &UploadTask) -> Result<()> {
    let password = task.server_config.password.as_deref().unwrap_or("");
    let username = &task.server_config.username;
    let host = &task.server_config.host;
    
    let remote_path = format!("{}@{}:{}", username, host, task.remote_path);
    
    // Use sshpass if password is provided
    let mut command = if !password.is_empty() {
        let mut cmd = Command::new("sshpass");
        cmd.arg("-p")
            .arg(password)
            .arg("scp");
        cmd
    } else {
        let mut cmd = Command::new("scp");
        cmd
    };
    
    // Add the local and remote paths
    command.arg(&task.local_path)
           .arg(remote_path);
    
    // Execute the command
    let output = command.output().await
        .map_err(|e| AppError::UploadFailed(format!("Failed to execute scp: {}", e)))?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::UploadFailed(format!("scp failed: {}", error)));
    }
    
    info!("File uploaded successfully with scp");
    Ok(())
}

/// Upload a file using sftp (not implemented yet)
async fn upload_with_sftp(task: &UploadTask) -> Result<()> {
    // This would typically use a Rust SFTP library
    warn!("SFTP upload not yet implemented, falling back to shell command");
    
    // For now, fall back to using scp as a temporary measure
    upload_with_scp(task).await
}

/// Create an upload task for a ripped DVD file
pub fn create_upload_task(
    local_path: impl AsRef<Path>,
    server_config: &ServerConfig,
    movie_name: &str
) -> UploadTask {
    let local_path = local_path.as_ref().to_path_buf();
    
    // Get the filename from the local path
    let filename = local_path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "movie.mp4".to_string());
    
    // Construct remote path
    let remote_path = format!("{}/{}/{}", 
                             server_config.path.trim_end_matches('/'),
                             movie_name,
                             filename);
    
    UploadTask {
        local_path,
        remote_path,
        server_config: server_config.clone(),
    }
}