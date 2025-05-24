use std::env;
use tracing::info;
use tracing_subscriber::EnvFilter;

use error::Result;

mod app;
mod cli;
mod config;
mod dvd;
mod error;
#[cfg(feature = "gui")]
mod gui;
mod upload;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    
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
    if env::var("DVDRIPPER_FONT_FIX").is_ok() {
        info!("Font fix environment variable detected, applying additional workarounds");
        env::set_var("ICED_BACKEND", "gl");
        env::set_var("RUST_BACKTRACE", "1");
    }

    // Run the application using the runner
    app::runner::run().await
}