//! Rusty Renderer - A multi-backend 3D renderer in Rust
//!
//! This is the main entry point for the Rusty Renderer application.

use anyhow::Result;
use rusty_renderer::app::App;
use rusty_renderer::config::Config;

fn main() -> Result<()> {
    // Write to file for debugging under Wine
    use std::io::Write;
    let mut log_file = std::fs::File::create("rusty_renderer_debug.log").ok();
    if let Some(ref mut f) = log_file {
        let _ = writeln!(f, "Starting Rusty Renderer v{}", env!("CARGO_PKG_VERSION"));
        let _ = f.flush();
    }

    // Set up panic hook for better error messages under Wine
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC: {panic_info}");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open("rusty_renderer_debug.log")
        {
            let _ = writeln!(f, "PANIC: {panic_info}");
        }
    }));

    eprintln!("Starting Rusty Renderer v{}", env!("CARGO_PKG_VERSION"));

    // Initialize logging
    if let Some(ref mut f) = log_file {
        let _ = writeln!(f, "Initializing logging");
        let _ = f.flush();
    }
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    eprintln!("Logging initialized");
    log::info!("Rusty Renderer v{}", env!("CARGO_PKG_VERSION"));

    let config = Config::parse_args();
    if let Err(e) = config.validate() {
        eprintln!("Invalid configuration: {e}");
        if let Some(ref mut f) = log_file {
            let _ = writeln!(f, "Invalid configuration: {e}");
            let _ = f.flush();
        }
        return Err(e);
    }

    eprintln!("Configuration parsed successfully");
    if let Some(ref mut f) = log_file {
        let _ = writeln!(f, "Configuration parsed successfully");
        let _ = f.flush();
    }

    eprintln!("Running renderer...");
    if let Some(ref mut f) = log_file {
        let _ = writeln!(f, "Running renderer");
        let _ = f.flush();
    }

    if let Err(e) = App::run(config) {
        eprintln!("Error running application: {e:?}");
        if let Some(ref mut f) = log_file {
            let _ = writeln!(f, "Error running application: {e:?}");
            let _ = f.flush();
        }
        return Err(e);
    }

    eprintln!("Shutdown complete");
    log::info!("Shutdown complete");
    if let Some(ref mut f) = log_file {
        let _ = writeln!(f, "Shutdown complete");
    }
    Ok(())
}
