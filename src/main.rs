//! Rusty Renderer - A multi-backend 3D renderer in Rust
//!
//! This is the main entry point for the Rusty Renderer application.

use anyhow::Result;

// Module declarations
mod app;
mod backends;
mod profiling;
mod render_graph;
mod scene;
mod shaders;
mod ui;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Rusty Renderer v{}", env!("CARGO_PKG_VERSION"));

    // Run the application
    app::App::run()?;

    log::info!("Shutdown complete");
    Ok(())
}
