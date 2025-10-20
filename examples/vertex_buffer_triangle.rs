//! Vertex buffer triangle example
//!
//! This example demonstrates rendering a triangle using actual vertex buffers
//! instead of hardcoded vertices in the shader. This is the M8.2 milestone
//! demonstrating proper vertex/index buffer rendering.

use anyhow::Result;
use rusty_renderer::backends::{
    create_backend, BackendType, BufferDescriptor, BufferUsage, MemoryLocation, Vertex,
};
use rusty_renderer::render_graph::{
    AccessType, Extent3D, Format, ImageLayout, ImageUsageFlags, PassCallback, PassExecutionContext,
    PassKind, PipelineStage, RenderGraph, RenderPass, ResourceAccess, SampleCount,
};

/// Triangle pass that uses a vertex buffer
struct VertexBufferTrianglePass {
    vertex_buffer_ptr: *const std::ffi::c_void,
}

// Safety: The vertex buffer pointer is only used during rendering within a single thread
unsafe impl Send for VertexBufferTrianglePass {}
unsafe impl Sync for VertexBufferTrianglePass {}

impl PassCallback for VertexBufferTrianglePass {
    fn execute(&self, context: &mut dyn PassExecutionContext) {
        log::debug!("Executing vertex buffer triangle pass");

        // Bind the vertex buffer
        if let Err(e) = context.bind_vertex_buffer(0, self.vertex_buffer_ptr, 0) {
            log::error!("Failed to bind vertex buffer: {e}");
            return;
        }

        // Draw 3 vertices (triangle), 1 instance
        if let Err(e) = context.draw(3, 1, 0, 0) {
            log::error!("Failed to draw triangle: {e}");
            return;
        }

        log::debug!("Vertex buffer triangle drawn successfully");
    }
}

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

    // Parse backend from command line (default to Vulkan)
    let backend_type = std::env::args()
        .nth(1)
        .and_then(|s| match s.to_lowercase().as_str() {
            "vulkan" => Some(BackendType::Vulkan),
            "wgpu" => Some(BackendType::Wgpu),
            "directx" | "dx12" => Some(BackendType::DirectX12),
            _ => None,
        })
        .unwrap_or(BackendType::Vulkan);

    println!("Using backend: {backend_type}");

    // Create backend
    let mut backend = create_backend(backend_type, true)?;
    println!("Backend created");

    // Initialize in headless mode for screenshot capture
    backend.initialize_headless(800, 600)?;
    println!("Backend initialized (headless 800x600)");

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

    // Create triangle pass
    let pass_id = graph.next_pass_id();
    let mut pass = RenderPass::new(pass_id, "vertex_buffer_triangle", PassKind::Graphics);

    // Output: write to color buffer
    pass.add_output(ResourceAccess::new(
        color_buffer,
        AccessType::Write,
        PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
        Some(ImageLayout::ColorAttachment),
    ));

    // Get raw pointer to buffer (safe because buffer lives longer than pass execution)
    let buffer_ptr = vertex_buffer.as_ref() as *const _ as *const std::ffi::c_void;

    // Set callback with vertex buffer
    let callback = Box::new(VertexBufferTrianglePass {
        vertex_buffer_ptr: buffer_ptr,
    });
    pass = pass.with_callback(callback);

    graph.add_pass(pass);

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

    Ok(())
}
