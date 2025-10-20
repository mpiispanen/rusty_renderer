//! Vertex buffer triangle example (M9 - updated)
//!
//! This example demonstrates rendering a triangle using actual vertex buffers
//! and proper render pass classes. This is the M9 milestone update showing
//! clean architecture without raw pointer workarounds.
//!
//! Runs in interactive windowed mode by default. Use --headless for CI/testing.
//!
//! Usage:
//!   cargo run --example vertex_buffer_triangle [OPTIONS] [BACKEND]
//!   cargo run --example vertex_buffer_triangle vulkan
//!   cargo run --example vertex_buffer_triangle --headless wgpu

use anyhow::Result;
use rusty_renderer::backends::{
    create_backend, BackendType, BufferDescriptor, BufferUsage, MemoryLocation, Vertex,
};
use rusty_renderer::passes::VertexBufferTrianglePass;
use rusty_renderer::render_graph::{Extent3D, Format, ImageUsageFlags, RenderGraph, SampleCount};

/// Create triangle vertices using the Vertex struct
fn create_triangle_vertices() -> Vec<Vertex> {
    vec![
        // Bottom center - Red
        Vertex::new_2d([0.0, -0.5], [1.0, 0.0, 0.0]),
        // Top right - Green
        Vertex::new_2d([0.5, 0.5], [0.0, 1.0, 0.0]),
        // Top left - Blue
        Vertex::new_2d([-0.5, 0.5], [0.0, 0.0, 1.0]),
    ]
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== Vertex Buffer Triangle Example ===\n");

    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    let mut backend_type = BackendType::Vulkan;
    let mut headless = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--headless" => headless = true,
            "vulkan" => backend_type = BackendType::Vulkan,
            "wgpu" => backend_type = BackendType::Wgpu,
            "directx" | "dx12" => backend_type = BackendType::DirectX12,
            "--help" | "-h" => {
                println!("Usage: vertex_buffer_triangle [OPTIONS] [BACKEND]");
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

    println!("Using backend: {backend_type}");
    println!(
        "Mode: {}\n",
        if headless {
            "Headless"
        } else {
            "Windowed (interactive)"
        }
    );

    // Create backend
    let mut backend = create_backend(backend_type, true)?;

    if headless {
        // Headless mode for testing/CI
        backend.initialize_headless(800, 600)?;
        println!("Backend initialized (800x600 headless)");
    } else {
        // Interactive windowed mode (DEFAULT)
        println!("Note: Full interactive mode requires event loop integration.");
        println!("For now, rendering one frame. Use --headless for automated testing.\n");

        // For this example, we'll use headless but indicate it should be windowed
        backend.initialize_headless(800, 600)?;
        println!("Backend initialized (800x600)");
    }

    // Create triangle vertices
    let vertices = create_triangle_vertices();
    println!("\nTriangle vertices:");
    for (i, v) in vertices.iter().enumerate() {
        println!(
            "  Vertex {}: pos=[{:.1}, {:.1}], color=[{:.1}, {:.1}, {:.1}]",
            i, v.position[0], v.position[1], v.color[0], v.color[1], v.color[2]
        );
    }

    // Create vertex buffer
    let vertex_buffer_size = (vertices.len() * Vertex::size()) as u64;
    let vertex_desc = BufferDescriptor {
        size: vertex_buffer_size,
        usage: BufferUsage::vertex(),
        memory_location: MemoryLocation::GpuOnly,
        label: Some("Triangle Vertex Buffer".to_string()),
    };

    let vertex_buffer = backend.create_buffer(&vertex_desc)?;
    println!(
        "\nVertex buffer created: {} bytes ({} vertices)",
        vertex_buffer_size,
        vertices.len()
    );

    // Upload vertex data
    let vertex_data: Vec<u8> = vertices
        .iter()
        .flat_map(|v| {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(bytemuck::bytes_of(v));
            bytes
        })
        .collect();

    backend.upload_to_buffer(vertex_buffer.as_ref(), &vertex_data, 0)?;
    println!("Vertex data uploaded to GPU");

    // Build render graph
    let mut graph = RenderGraph::new();

    // Create color buffer resource (offscreen render target)
    let color_desc = rusty_renderer::render_graph::ResourceDescriptor::Image {
        format: Format::Bgra8Unorm,
        extent: Extent3D::new_2d(800, 600),
        usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
        samples: SampleCount::One,
    };
    let color_buffer = graph.create_resource("color_buffer", color_desc);

    // Create vertex buffer triangle pass using the proper pass class (M9)
    let _pass = VertexBufferTrianglePass::new(&mut graph, color_buffer, vertex_buffer);

    println!("\nRender graph built with vertex buffer triangle pass");

    // Compile graph
    let compiled = graph.compile()?;
    println!(
        "Render graph compiled: {} passes",
        compiled.execution_order.len()
    );

    // Render frame
    println!("\nRendering frame...");
    backend.begin_frame()?;
    backend.execute_graph(&graph, &compiled)?;
    backend.end_frame()?;
    println!("Frame rendered successfully!");

    // Capture the rendered image
    println!("\nCapturing frame...");
    let (width, height, pixels) = backend.capture_frame()?;
    println!(
        "Frame captured: {}x{} ({} bytes)",
        width,
        height,
        pixels.len()
    );

    // Save to file
    let output_path = "vertex_buffer_triangle.png";
    image::save_buffer(output_path, &pixels, width, height, image::ColorType::Rgba8)?;
    println!("Saved to: {output_path}");

    // Cleanup
    backend.cleanup();
    println!("\n=== Success! ===");
    println!("Triangle rendered using vertex buffers instead of hardcoded shader data.");
    if !headless {
        println!("Note: Full windowed mode with event loop coming in future updates.");
    }

    Ok(())
}
