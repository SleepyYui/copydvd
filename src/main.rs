use std::process;
use tracing::{info, error};

mod app;
mod dvd;
mod config;
mod error;
mod gui;
mod cli;
mod utils;
mod upload;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("DVD Ripper starting up");
    
    // Log system information
    utils::log_system_info();

    // Run the application
    match app::run().await {
        Ok(_) => info!("DVD Ripper completed successfully"),
        Err(e) => {
            error!("DVD Ripper failed: {e}");
            process::exit(1);
        }
    }
}