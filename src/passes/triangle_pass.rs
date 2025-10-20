//! Triangle rendering pass
//!
//! A simple pass that renders a colored triangle using a vertex buffer.

use crate::backends::{Buffer, BufferDescriptor, BufferUsage, MemoryLocation, Vertex};
use crate::render_graph::{
    AccessType, ImageLayout, PassCallback, PassExecutionContext, PassId, PassKind, PipelineStage,
    RenderGraph, RenderPass, ResourceAccess, ResourceId,
};
use anyhow::Result;

/// Triangle rendering pass
///
/// This pass renders a single colored triangle to a color attachment.
/// The triangle vertices are stored in a GPU vertex buffer.
///
/// # Resources
/// - **Output**: Color attachment (swapchain image or offscreen buffer)
/// - **Vertex Buffer**: 3 vertices (RGB triangle)
///
/// # Example
/// ```no_run
/// use rusty_renderer::passes::TrianglePass;
/// use rusty_renderer::render_graph::RenderGraph;
/// # fn example(mut graph: RenderGraph, color_buffer: rusty_renderer::render_graph::ResourceId, backend: &mut dyn rusty_renderer::backends::GraphicsBackend) {
/// let triangle_pass = TrianglePass::new(&mut graph, color_buffer, backend).unwrap();
/// # }
/// ```
pub struct TrianglePass {
    pass_id: PassId,
}

impl TrianglePass {
    /// Create a new triangle rendering pass
    ///
    /// # Arguments
    /// * `graph` - The render graph to add the pass to
    /// * `color_output` - The color attachment resource to render to
    /// * `backend` - The graphics backend to create the vertex buffer
    ///
    /// # Returns
    /// A TrianglePass instance
    pub fn new(
        graph: &mut RenderGraph,
        color_output: ResourceId,
        backend: &mut dyn crate::backends::GraphicsBackend,
    ) -> Result<Self> {
        let pass_id = graph.next_pass_id();

        // Create triangle vertices
        let vertices = [
            Vertex::new_2d([0.0, -0.5], [1.0, 0.0, 0.0]), // Bottom - Red
            Vertex::new_2d([0.5, 0.5], [0.0, 1.0, 0.0]),  // Top Right - Green
            Vertex::new_2d([-0.5, 0.5], [0.0, 0.0, 1.0]), // Top Left - Blue
        ];

        // Create vertex buffer
        let vertex_buffer_size = (vertices.len() * Vertex::size()) as u64;
        let vertex_desc = BufferDescriptor {
            size: vertex_buffer_size,
            usage: BufferUsage::vertex(),
            memory_location: MemoryLocation::GpuOnly,
            label: Some("Triangle Vertex Buffer".to_string()),
        };

        let vertex_buffer = backend.create_buffer(&vertex_desc)?;

        // Upload vertex data
        let vertex_data = vertices
            .iter()
            .flat_map(bytemuck::bytes_of)
            .copied()
            .collect::<Vec<u8>>();

        backend.upload_to_buffer(vertex_buffer.as_ref(), &vertex_data, 0)?;

        log::debug!("Created triangle vertex buffer: {vertex_buffer_size} bytes");

        // Convert to raw pointer for callback
        let vertex_buffer_ptr =
            vertex_buffer.as_ref() as *const dyn Buffer as *const std::ffi::c_void;

        let mut pass = RenderPass::new(pass_id, "triangle_pass", PassKind::Graphics);

        // Configure output: write to color buffer
        pass.add_output(ResourceAccess::new(
            color_output,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            Some(ImageLayout::ColorAttachment),
        ));

        // Set up callback with vertex buffer
        pass = pass.with_callback(Box::new(TrianglePassCallback { vertex_buffer_ptr }));

        graph.add_pass(pass);

        // Keep vertex buffer alive (TODO: proper resource management)
        std::mem::forget(vertex_buffer);

        Ok(Self { pass_id })
    }

    /// Get the pass ID
    pub fn pass_id(&self) -> PassId {
        self.pass_id
    }
}

/// Callback for triangle pass execution
struct TrianglePassCallback {
    vertex_buffer_ptr: *const std::ffi::c_void,
}

// Safety: The vertex buffer pointer is only used during rendering within a single thread
unsafe impl Send for TrianglePassCallback {}
unsafe impl Sync for TrianglePassCallback {}

impl PassCallback for TrianglePassCallback {
    fn execute(&self, context: &mut dyn PassExecutionContext) {
        // Bind vertex buffer
        if let Err(e) = context.bind_vertex_buffer(0, self.vertex_buffer_ptr, 0) {
            log::error!("Failed to bind vertex buffer: {e}");
            return;
        }

        // Draw 3 vertices (triangle) with 1 instance
        if let Err(e) = context.draw(3, 1, 0, 0) {
            log::error!("Failed to draw triangle: {e}");
            return;
        }

        log::trace!("Triangle pass executed with vertex buffer");
    }
}

#[cfg(test)]
mod tests {
    // Note: Tests disabled because TrianglePass now requires a backend to create vertex buffers
    // These should be integration tests with a real backend

    #[test]
    #[ignore]
    fn test_triangle_pass_creation() {
        // TODO: Implement with mock backend or as integration test
    }

    #[test]
    #[ignore]
    fn test_triangle_pass_builder() {
        // TODO: Implement with mock backend or as integration test
    }

    #[test]
    #[ignore]
    fn test_triangle_pass_compiles() {
        // TODO: Implement with mock backend or as integration test
    }
}
