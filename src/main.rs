//! Rusty Renderer - A multi-backend 3D renderer in Rust
//!
//! This is the main entry point for the Rusty Renderer application.

use anyhow::Result;
use rusty_renderer::application::ApplicationRunner;

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

    // Create and run the application
    eprintln!("Parsing arguments...");
    if let Some(ref mut f) = log_file {
        let _ = writeln!(f, "Parsing arguments");
        let _ = f.flush();
    }
    let runner = match ApplicationRunner::from_args() {
        Ok(r) => {
            eprintln!("Arguments parsed successfully");
            if let Some(ref mut f) = log_file {
                let _ = writeln!(f, "Arguments parsed successfully");
                let _ = f.flush();
            }
            r
        }
        Err(e) => {
            eprintln!("Error creating application: {e}");
            if let Some(ref mut f) = log_file {
                let _ = writeln!(f, "Error creating application: {e}");
                let _ = f.flush();
            }
            return Err(e);
        }
    };

    eprintln!("Running application...");
    if let Some(ref mut f) = log_file {
        let _ = writeln!(f, "Running application");
        let _ = f.flush();
    }
    if let Err(e) = runner.run() {
        eprintln!("Error running application: {e}");
        if let Some(ref mut f) = log_file {
            let _ = writeln!(f, "Error running application: {e}");
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
