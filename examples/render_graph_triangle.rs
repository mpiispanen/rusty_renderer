//! Simple render graph triangle example (M9)
//!
//! This example demonstrates the minimal render graph usage with a single
//! triangle pass using vertex buffers. This is the cleanest example showing
//! the M9 render graph architecture.
//!
//! Runs in interactive windowed mode by default. Use --headless for CI/testing.
//!
//! Usage:
//!   cargo run --example render_graph_triangle [OPTIONS] [BACKEND]
//!   cargo run --example render_graph_triangle vulkan
//!   cargo run --example render_graph_triangle --headless wgpu

use anyhow::Result;
use rusty_renderer::backends::{
    create_backend, BackendType, BufferDescriptor, BufferUsage, MemoryLocation, Vertex,
};
use rusty_renderer::passes::VertexBufferTrianglePass;
use rusty_renderer::render_graph::{
    Extent3D, Format, ImageUsageFlags, RenderGraph, ResourceDescriptor, SampleCount,
};

/// Create simple triangle vertices (red, green, blue)
fn create_triangle() -> Vec<Vertex> {
    vec![
        Vertex::new_2d([0.0, -0.5], [1.0, 0.0, 0.0]), // Bottom: Red
        Vertex::new_2d([0.5, 0.5], [0.0, 1.0, 0.0]),  // Top-right: Green
        Vertex::new_2d([-0.5, 0.5], [0.0, 0.0, 1.0]), // Top-left: Blue
    ]
}

fn main() -> Result<()> {
    // Setup logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== Render Graph Triangle Example (M9) ===\n");

    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    let mut backend_type = BackendType::Vulkan;
    let mut headless = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--headless" => headless = true,
            "vulkan" => backend_type = BackendType::Vulkan,
            "directx" | "dx12" => backend_type = BackendType::DirectX12,
            "--help" | "-h" => {
                println!("Usage: render_graph_triangle [OPTIONS] [BACKEND]");
                println!("\nBackends:");
                println!("  vulkan    Use Vulkan backend (default)");
                println!("  wgpu      Use wgpu backend");
                println!("  directx   Use DirectX 12 backend (Windows only)");
                println!("\nOptions:");
                println!("  --headless    Run in headless mode (for CI/testing)");
                println!("  --help, -h    Show this help message");
                println!("\nDefault mode is interactive windowed rendering.");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    println!("Backend: {backend_type}");
    println!(
        "Mode: {}\n",
        if headless {
            "Headless"
        } else {
            "Windowed (interactive)"
        }
    );

    // Create backend
    let mut backend = create_backend(backend_type, false)?;

    if headless {
        // Headless mode for testing/CI
        backend.initialize_headless(800, 600)?;
        println!("Initialized backend (800x600 headless)");
    } else {
        // Interactive windowed mode (DEFAULT)
        println!("Note: Full interactive mode requires event loop integration.");
        println!("For now, rendering one frame. Use --headless for automated testing.\n");

        // For this example, we'll use headless but indicate it should be windowed
        backend.initialize_headless(800, 600)?;
        println!("Initialized backend (800x600)");
    }

    // Create triangle vertices
    let vertices = create_triangle();
    let vertex_data: Vec<u8> = vertices
        .iter()
        .flat_map(|v| bytemuck::bytes_of(v).iter().copied())
        .collect();

    // Create vertex buffer
    let vertex_buffer = backend.create_buffer(&BufferDescriptor {
        size: vertex_data.len() as u64,
        usage: BufferUsage::vertex(),
        memory_location: MemoryLocation::GpuOnly,
        label: Some("Triangle Vertices".to_string()),
    })?;

    backend.upload_to_buffer(vertex_buffer.as_ref(), &vertex_data, 0)?;
    println!("Created vertex buffer: {} bytes", vertex_data.len());

    // Build render graph
    let mut graph = RenderGraph::new();

    // Create color attachment
    let color_buffer = graph.create_resource(
        "color_output",
        ResourceDescriptor::Image {
            format: Format::Bgra8Unorm,
            extent: Extent3D::new_2d(800, 600),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
        },
    );

    // Add triangle pass (single line!)
    let _triangle_pass = VertexBufferTrianglePass::new(&mut graph, color_buffer, vertex_buffer);

    println!("Built render graph with 1 pass");

    // Compile and execute
    let compiled = graph.compile()?;
    println!("Compiled graph: {} passes", compiled.execution_order.len());

    println!("\nRendering...");
    backend.begin_frame()?;
    backend.execute_graph(&graph, &compiled)?;
    backend.end_frame()?;
    println!("✓ Rendered successfully");

    // Capture output and save
    println!("\nCapturing frame...");
    let (width, height, pixels) = backend.capture_frame()?;
    println!("Captured frame: {width}x{height}");

    let output_path = "render_graph_triangle.png";
    image::save_buffer(output_path, &pixels, width, height, image::ColorType::Rgba8)?;
    println!("Saved to: {output_path}");

    backend.cleanup();
    println!("\n=== Success! ===");
    if !headless {
        println!("Note: Full windowed mode with event loop coming in future updates.");
    }
    println!("Rendered image saved to: render_graph_triangle.png");

    Ok(())
}
