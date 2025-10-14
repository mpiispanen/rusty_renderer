//! Rusty Renderer - A multi-backend 3D renderer in Rust
//!
//! This is the main entry point for the Rusty Renderer application.

use anyhow::Result;
use rusty_renderer::{app, config};

fn main() -> Result<()> {
    // Parse command-line arguments
    let config = config::Config::parse_args();

    // Initialize logging with the configured level
    env_logger::Builder::from_default_env()
        .filter_level(config.log_level)
        .init();

    log::info!("Rusty Renderer v{}", env!("CARGO_PKG_VERSION"));

    // Validate configuration
    config.validate()?;

    // Run the application
    app::App::run(config)?;

    log::info!("Shutdown complete");
    Ok(())
}
