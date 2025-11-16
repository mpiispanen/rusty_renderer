//! Rusty Renderer - A multi-backend 3D renderer in Rust
//!
//! This is the main entry point for the Rusty Renderer application.

use anyhow::Result;
use rusty_renderer::app::App;
use rusty_renderer::config::Config;

fn setup_logging(log_level: log::LevelFilter) -> Result<()> {
    // Configure console output
    let console_dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stderr());

    // Configure file output
    let file_dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log_level)
        .chain(fern::log_file("rusty_renderer.log")?);

    // Combine both dispatches
    fern::Dispatch::new()
        .chain(console_dispatch)
        .chain(file_dispatch)
        .apply()
        .map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;

    Ok(())
}

fn main() -> Result<()> {
    // Set up panic hook for better error messages
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC: {panic_info}");
        log::error!("PANIC: {panic_info}");
    }));

    // Parse config first to get log level
    let config = Config::parse_args();

    // Initialize logging with proper dual output (stdout + file)
    setup_logging(config.log_level)?;

    log::info!("Rusty Renderer v{}", env!("CARGO_PKG_VERSION"));
    log::info!(
        "Backend: {:?}, Scene: {}, Debug: {}",
        config.backend,
        config.scene,
        config.debug
    );

    if let Err(e) = config.validate() {
        log::error!("Invalid configuration: {e}");
        return Err(e);
    }

    if let Err(e) = App::run(config) {
        log::error!("Error running application: {e:?}");
        return Err(e);
    }

    log::info!("Shutdown complete");
    Ok(())
}
