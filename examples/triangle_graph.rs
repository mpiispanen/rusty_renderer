//! Triangle rendering using the render graph system
//!
//! This example demonstrates how to use the render graph to render a simple triangle.
//! It serves as both a validation of the render graph system and an example of how to use it.

use rusty_renderer::render_graph::{
    AccessType, Extent3D, Format, ImageLayout, ImageUsageFlags, PassCallback, PassExecutionContext,
    PassKind, PipelineStage, RenderGraph, RenderPass, ResourceAccess, ResourceDescriptor,
    SampleCount,
};

/// Triangle pass callback - executes the actual rendering
struct TrianglePass;

impl PassCallback for TrianglePass {
    fn execute(&self, _context: &mut dyn PassExecutionContext) {
        // In a full implementation, this would:
        // 1. Bind the graphics pipeline
        // 2. Set viewport/scissor
        // 3. Draw the triangle (3 vertices)

        // For now, this is a placeholder showing the API
        println!("Executing triangle pass");
    }
}

/// Build a render graph for triangle rendering
fn build_triangle_graph(width: u32, height: u32) -> anyhow::Result<RenderGraph> {
    let mut graph = RenderGraph::new();

    // Create the color buffer resource (swapchain image in real implementation)
    let color_desc = ResourceDescriptor::Image {
        format: Format::Bgra8Unorm, // Most common swapchain format
        extent: Extent3D::new_2d(width, height),
        usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
        samples: SampleCount::One,
    };

    let color_buffer = graph.create_resource("swapchain_image", color_desc);

    // Create the triangle render pass
    let mut triangle_pass =
        RenderPass::new(graph.next_pass_id(), "triangle_pass", PassKind::Graphics);

    // Output: write to color buffer
    triangle_pass.add_output(ResourceAccess::new(
        color_buffer,
        AccessType::Write,
        PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
        Some(ImageLayout::ColorAttachment),
    ));

    // Set execution callback
    let triangle_pass = triangle_pass.with_callback(Box::new(TrianglePass));

    // Add pass to graph
    graph.add_pass(triangle_pass);

    Ok(graph)
}

fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== Render Graph Triangle Example ===\n");

    // Build the render graph
    println!("Building render graph...");
    let mut graph = build_triangle_graph(800, 600)?;

    // Compile the graph
    println!("Compiling render graph...");
    let compiled = graph.compile()?;

    println!("\n=== Compilation Results ===");
    println!("Execution order:");
    for (idx, pass_id) in compiled.execution_order.iter().enumerate() {
        if let Some(pass) = graph.get_pass(*pass_id) {
            println!("  {}: {} ({})", idx, pass.name, pass.id);
        }
    }

    println!("\nBarriers ({} total):", compiled.barriers.len());
    for barrier in &compiled.barriers {
        println!("  {:?} -> {:?}", barrier.src_pass, barrier.dst_pass);
        if !barrier.image_barriers.is_empty() {
            for img_barrier in &barrier.image_barriers {
                println!(
                    "    Image transition: {:?} -> {:?}",
                    img_barrier.old_layout, img_barrier.new_layout
                );
            }
        }
        if barrier.memory_barrier.is_some() {
            println!("    Memory barrier");
        }
    }

    println!("\nResource producers:");
    for (resource_id, pass_id) in &compiled.producers {
        if let Some(resource) = graph.get_resource(*resource_id) {
            if let Some(pass) = graph.get_pass(*pass_id) {
                println!("  {} produced by {}", resource.name, pass.name);
            }
        }
    }

    // In a real implementation, we would now execute the graph
    println!("\n=== Execution (simulated) ===");
    for pass_id in &compiled.execution_order {
        if let Some(pass) = graph.get_pass(*pass_id) {
            println!("Executing: {}", pass.name);
            if let Some(callback) = &pass.callback {
                // In real implementation, create proper context with backend access
                struct DummyContext;
                impl PassExecutionContext for DummyContext {
                    fn as_any(&self) -> &dyn std::any::Any {
                        self
                    }
                    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                        self
                    }
                    fn bind_vertex_buffer(
                        &mut self,
                        _binding: u32,
                        _buffer_ptr: *const std::ffi::c_void,
                        _offset: u64,
                    ) -> anyhow::Result<()> {
                        Ok(())
                    }
                    fn bind_index_buffer(
                        &mut self,
                        _buffer_ptr: *const std::ffi::c_void,
                        _offset: u64,
                        _index_type: rusty_renderer::render_graph::IndexType,
                    ) -> anyhow::Result<()> {
                        Ok(())
                    }
                    fn draw(
                        &mut self,
                        _vertex_count: u32,
                        _instance_count: u32,
                        _first_vertex: u32,
                        _first_instance: u32,
                    ) -> anyhow::Result<()> {
                        Ok(())
                    }
                    fn draw_indexed(
                        &mut self,
                        _index_count: u32,
                        _instance_count: u32,
                        _first_index: u32,
                        _vertex_offset: i32,
                        _first_instance: u32,
                    ) -> anyhow::Result<()> {
                        Ok(())
                    }
                }
                let mut ctx = DummyContext;
                callback.execute(&mut ctx);
            }
        }
    }

    println!("\n=== Success ===");
    println!("Render graph compiled and validated successfully!");
    println!("In a full implementation, this would render a triangle to the screen.");

    Ok(())
}
