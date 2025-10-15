use rusty_renderer::{RenderConfig, RenderBackend};
use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    log::info!("Starting Rusty Renderer Triangle Example");
    
    // Create configuration
    let config = RenderConfig {
        backend: RenderBackend::Vulkan,
        width: 800,
        height: 600,
        vsync: true,
        debug: true,
        log_level: log::LevelFilter::Info,
    };
    
    // Create app
    let mut app = rusty_renderer::app::App::new(config)?;
    
    // Create event loop and run
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    
    event_loop.run_app(&mut app)?;
    
    Ok(())
}
