mod app;
mod cli;
mod config;
mod dvd;
mod error;
#[cfg(feature = "gui")]
mod gui;
mod upload;
mod utils;

use tokio::runtime::Runtime;
use crate::error::Result;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Create a runtime for our async functions
    let rt = Runtime::new()?;
    
    // Run the application
    rt.block_on(async {
        app::run().await
    })
}