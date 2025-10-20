//! Rusty Renderer - A multi-backend 3D renderer in Rust
//!
//! This is the main entry point for the Rusty Renderer application.

use anyhow::Result;
use rusty_renderer::application::ApplicationRunner;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Rusty Renderer v{}", env!("CARGO_PKG_VERSION"));

    // Create and run the application
    let runner = ApplicationRunner::from_args()?;
    runner.run()?;

    log::info!("Shutdown complete");
    Ok(())
}
