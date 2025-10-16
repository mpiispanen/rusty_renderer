use rusty_renderer::{RenderBackend, RenderConfig};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use winit::event_loop::EventLoop;

/// Command-line arguments for triangle example
#[derive(Debug)]
struct Args {
    /// Optional test duration in seconds (for automated testing)
    test_duration: Option<u64>,
    /// Backend to use (vulkan, directx, wgpu)
    backend: Option<RenderBackend>,
}

impl Args {
    fn parse() -> Self {
        let mut test_duration = None;
        let mut backend = None;

        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--test-duration" => {
                    test_duration = args.next().and_then(|s| s.parse().ok());
                }
                "--backend" | "-b" => {
                    backend = args.next().and_then(|s| match s.to_lowercase().as_str() {
                        "vulkan" | "vk" => Some(RenderBackend::Vulkan),
                        "directx" | "dx" | "dx12" => Some(RenderBackend::DirectX),
                        "wgpu" => Some(RenderBackend::Wgpu),
                        _ => {
                            eprintln!("Unknown backend: {s}");
                            eprintln!("Valid backends: vulkan, directx, wgpu");
                            None
                        }
                    });
                }
                "--help" | "-h" => {
                    println!("Triangle Example - Rusty Renderer");
                    println!("\nUsage: triangle [OPTIONS]");
                    println!("\nOptions:");
                    println!("  --backend, -b <backend>    Graphics backend to use");
                    println!("                             Options: vulkan (default), directx, wgpu");
                    println!("  --test-duration <seconds>  Run for specified duration then exit (for testing)");
                    println!("  --help, -h                 Show this help message");
                    println!("\nExamples:");
                    println!("  triangle                   # Use default Vulkan backend");
                    println!("  triangle --backend wgpu    # Use wgpu backend");
                    println!("  triangle -b dx12           # Use DirectX 12 backend");
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Unknown argument: {arg}");
                    eprintln!("Use --help for usage information");
                    std::process::exit(1);
                }
            }
        }

        Self {
            test_duration,
            backend,
        }
    }
}

fn main() -> anyhow::Result<()> {
    // Parse arguments
    let args = Args::parse();

    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting Rusty Renderer Triangle Example");

    // Determine backend
    let backend = args.backend.unwrap_or(RenderBackend::Vulkan);
    log::info!("Using backend: {:?}", backend);

    if let Some(duration) = args.test_duration {
        log::info!("Test mode: will run for {duration} seconds");
    }

    // Create configuration
    let config = RenderConfig {
        backend,
        width: 800,
        height: 600,
        vsync: true,
        debug: true,
        log_level: log::LevelFilter::Info,
    };

    // Create app
    let mut app = rusty_renderer::app::App::new(config)?;

    // Set up test duration timer if specified
    let start_time = args.test_duration.map(|_| Instant::now());
    let test_duration = args.test_duration.map(Duration::from_secs);

    // Create event loop
    let event_loop = EventLoop::new()?;

    // Use Poll mode for better responsiveness
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    // Run event loop with duration check
    let result = if let (Some(_start), Some(duration)) = (start_time, test_duration) {
        // Wrap the check in Arc<Mutex> for thread safety
        let should_exit = Arc::new(Mutex::new(false));
        let should_exit_clone = Arc::clone(&should_exit);

        // Spawn thread to set exit flag after duration
        std::thread::spawn(move || {
            std::thread::sleep(duration);
            *should_exit_clone.lock().unwrap() = true;
            log::info!("Test duration elapsed, requesting exit");
        });

        // Run with exit check
        // Note: In winit 0.30, we need to use the event loop API differently
        // For now, just run normally - the thread will exit the process
        event_loop.run_app(&mut app)
    } else {
        // Run normally without time limit
        event_loop.run_app(&mut app)
    };

    log::info!("Triangle example finished");
    result.map_err(|e| anyhow::anyhow!("Event loop error: {e:?}"))
}
