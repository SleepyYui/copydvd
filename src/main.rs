use std::env;
use tracing::info;
use tracing_subscriber::EnvFilter;

use error::Result;

mod app;
mod cli;
mod config;
mod dvd;
mod error;
mod gui;
mod handbrake_auto_fix;
mod handbrake_manager;
mod updater;
mod upload;
mod utils;

#[cfg(test)]
mod test_config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with better console output
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("info,copy_dvd=debug"))
                .unwrap(),
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("=== Copy DVD Application Starting ===");
    info!(
        "Platform: {} {}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    info!(
        "Working directory: {:?}",
        std::env::current_dir().unwrap_or_default()
    );

    // Set environment variables to work around font rendering issues on macOS
    #[cfg(target_os = "macos")]
    {
        info!("Applying macOS-specific workarounds for font and rendering issues");
        // Force OpenGL renderer to avoid Metal issues
        env::set_var("ICED_BACKEND", "gl");
        // Disable text multithreading which can cause font issues
        env::set_var("ICED_TEXT_MULTITHREADING", "false");
        // Set font hinting to avoid glyph rasterizer issues
        env::set_var("FREETYPE_PROPERTIES", "truetype:interpreter-version=35");
        // Enable backtrace for better debugging
        env::set_var("RUST_BACKTRACE", "1");
    }

    // Set environment variables to work around font rendering issues
    if env::var("COPYDVD_FONT_FIX").is_ok() {
        info!("Font fix environment variable detected, applying additional workarounds");
        env::set_var("ICED_BACKEND", "gl");
        env::set_var("RUST_BACKTRACE", "1");
    }

    // Run the application using the runner
    app::runner::run().await
}
