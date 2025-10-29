//! Vertex buffer triangle rendering pass (M9)
//!
//! A triangle rendering pass that uses actual vertex buffers instead of
//! hardcoded vertices in the shader. This demonstrates proper vertex buffer
//! usage through the render graph system.

use crate::backends::Buffer;
#[cfg(test)]
use crate::render_graph::ExtentMode;
use crate::render_graph::{
    AccessType, ImageLayout, PassCallback, PassExecutionContext, PassId, PassKind, PipelineStage,
    RenderGraph, RenderPass, ResourceAccess, ResourceId,
};
use anyhow::Result;
use std::sync::Arc;

/// Vertex buffer triangle rendering pass
///
/// This pass renders a colored triangle using vertex data from a GPU buffer.
/// The vertex buffer is provided during construction and is owned by the pass
/// through an Arc for shared ownership.
///
/// # Resources
/// - **Output**: Color attachment (swapchain image or offscreen buffer)
/// - **Input**: Vertex buffer (provided at construction)
///
/// # Example
/// ```no_run
/// use rusty_renderer::passes::VertexBufferTrianglePass;
/// use rusty_renderer::render_graph::RenderGraph;
/// # fn example(mut graph: RenderGraph, color_buffer: rusty_renderer::render_graph::ResourceId, vertex_buffer: Box<dyn rusty_renderer::backends::Buffer>) {
/// let triangle_pass = VertexBufferTrianglePass::new(&mut graph, color_buffer, vertex_buffer);
/// # }
/// ```
pub struct VertexBufferTrianglePass {
    pass_id: PassId,
}

impl VertexBufferTrianglePass {
    /// Create a new vertex buffer triangle rendering pass
    ///
    /// # Arguments
    /// * `graph` - The render graph to add the pass to
    /// * `color_output` - The color attachment resource to render to
    /// * `vertex_buffer` - The vertex buffer containing triangle vertex data
    ///
    /// # Returns
    /// A VertexBufferTrianglePass instance
    pub fn new(
        graph: &mut RenderGraph,
        color_output: ResourceId,
        vertex_buffer: Box<dyn Buffer>,
    ) -> Self {
        let pass_id = graph.next_pass_id();

        let mut pass = RenderPass::new(pass_id, "vertex_buffer_triangle", PassKind::Graphics);

        // Configure output: write to color buffer
        pass.add_output(ResourceAccess::new(
            color_output,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            Some(ImageLayout::ColorAttachment),
        ));

        // Wrap buffer in Arc for shared ownership
        let vertex_buffer_arc = Arc::new(vertex_buffer);

        // Set up callback with vertex buffer
        pass = pass.with_callback(Box::new(VertexBufferTriangleCallback {
            vertex_buffer: vertex_buffer_arc,
        }));

        graph.add_pass(pass);

        Self { pass_id }
    }

    /// Get the pass ID
    pub fn pass_id(&self) -> PassId {
        self.pass_id
    }
}

/// Callback for vertex buffer triangle pass execution
struct VertexBufferTriangleCallback {
    vertex_buffer: Arc<Box<dyn Buffer>>,
}

impl PassCallback for VertexBufferTriangleCallback {
    fn prepare(&self, _context: &mut dyn crate::render_graph::PassPreparationContext) {
        // For now, just call the default preparation which should set up minimal bind groups
        log::info!("VertexBufferTriangleCallback::prepare called");
        // Suboptimal: We need to ensure bind groups are created even if we have no uniforms/textures
        // The wgpu backend should create empty/default bind groups
    }

    fn execute(&self, context: &mut dyn PassExecutionContext) {
        log::debug!("Executing vertex buffer triangle pass");

        // Get raw pointer from the buffer for backend API
        // Arc<Box<dyn Buffer>> -> &dyn Buffer -> *const dyn Buffer -> *const c_void
        let buffer_ptr =
            self.vertex_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;

        // Bind the vertex buffer
        if let Err(e) = context.bind_vertex_buffer(0, buffer_ptr, 0) {
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

/// Builder for vertex buffer triangle pass with more configuration options
pub struct VertexBufferTrianglePassBuilder {
    color_output: ResourceId,
    vertex_buffer: Option<Box<dyn Buffer>>,
    vertex_count: u32,
    name: String,
}

impl VertexBufferTrianglePassBuilder {
    /// Create a new vertex buffer triangle pass builder
    pub fn new(color_output: ResourceId) -> Self {
        Self {
            color_output,
            vertex_buffer: None,
            vertex_count: 3,
            name: "vertex_buffer_triangle".to_string(),
        }
    }

    /// Set the vertex buffer
    pub fn with_vertex_buffer(mut self, buffer: Box<dyn Buffer>) -> Self {
        self.vertex_buffer = Some(buffer);
        self
    }

    /// Set the number of vertices to draw
    pub fn with_vertex_count(mut self, count: u32) -> Self {
        self.vertex_count = count;
        self
    }

    /// Set a custom name for the pass
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Build the pass and add it to the render graph
    pub fn build(self, graph: &mut RenderGraph) -> Result<VertexBufferTrianglePass> {
        let vertex_buffer = self
            .vertex_buffer
            .ok_or_else(|| anyhow::anyhow!("Vertex buffer not set"))?;

        let pass_id = graph.next_pass_id();

        let mut pass = RenderPass::new(pass_id, &self.name, PassKind::Graphics);

        pass.add_output(ResourceAccess::new(
            self.color_output,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            Some(ImageLayout::ColorAttachment),
        ));

        // Wrap buffer in Arc for shared ownership
        let vertex_buffer_arc = Arc::new(vertex_buffer);

        pass = pass.with_callback(Box::new(ConfigurableVertexBufferCallback {
            vertex_buffer: vertex_buffer_arc,
            vertex_count: self.vertex_count,
        }));

        graph.add_pass(pass);

        Ok(VertexBufferTrianglePass { pass_id })
    }
}

/// Configurable callback for vertex buffer rendering
struct ConfigurableVertexBufferCallback {
    vertex_buffer: Arc<Box<dyn Buffer>>,
    vertex_count: u32,
}

impl PassCallback for ConfigurableVertexBufferCallback {
    fn execute(&self, context: &mut dyn PassExecutionContext) {
        log::debug!(
            "Executing vertex buffer pass with {} vertices",
            self.vertex_count
        );

        let buffer_ptr =
            self.vertex_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;

        if let Err(e) = context.bind_vertex_buffer(0, buffer_ptr, 0) {
            log::error!("Failed to bind vertex buffer: {e}");
            return;
        }

        if let Err(e) = context.draw(self.vertex_count, 1, 0, 0) {
            log::error!("Failed to draw: {e}");
            return;
        }

        log::debug!("Vertex buffer rendered successfully");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::{BufferUsage, MemoryLocation};
    use crate::render_graph::{Extent3D, Format, ImageUsageFlags, ResourceDescriptor, SampleCount};

    // Helper to create a mock buffer for testing
    struct MockBuffer {
        size: u64,
    }

    impl Buffer for MockBuffer {
        fn size(&self) -> u64 {
            self.size
        }

        fn usage(&self) -> BufferUsage {
            BufferUsage::vertex()
        }

        fn memory_location(&self) -> MemoryLocation {
            MemoryLocation::GpuOnly
        }

        fn map(&mut self) -> Result<&mut [u8]> {
            anyhow::bail!("MockBuffer does not support mapping")
        }

        fn unmap(&mut self) {
            // No-op for mock
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_vertex_buffer_triangle_pass_creation() {
        let mut graph = RenderGraph::new();

        // Create a color buffer
        let color_desc = ResourceDescriptor::Image {
            format: Format::Bgra8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(800, 600)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let color_buffer = graph.create_resource("color_buffer", color_desc);

        // Create a mock vertex buffer
        let vertex_buffer = Box::new(MockBuffer { size: 144 });

        // Create pass
        let pass = VertexBufferTrianglePass::new(&mut graph, color_buffer, vertex_buffer);

        // Verify pass was created
        assert_eq!(pass.pass_id().0, 0);
    }

    #[test]
    fn test_vertex_buffer_triangle_pass_builder() {
        let mut graph = RenderGraph::new();

        let color_desc = ResourceDescriptor::Image {
            format: Format::Bgra8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(800, 600)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let color_buffer = graph.create_resource("color_buffer", color_desc);

        let vertex_buffer = Box::new(MockBuffer { size: 144 });

        // Create pass with builder
        let pass = VertexBufferTrianglePassBuilder::new(color_buffer)
            .with_vertex_buffer(vertex_buffer)
            .with_vertex_count(3)
            .with_name("custom_vb_triangle")
            .build(&mut graph)
            .unwrap();

        assert_eq!(pass.pass_id().0, 0);
    }

    #[test]
    fn test_vertex_buffer_triangle_pass_compiles() {
        let mut graph = RenderGraph::new();

        let color_desc = ResourceDescriptor::Image {
            format: Format::Bgra8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(800, 600)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let color_buffer = graph.create_resource("color_buffer", color_desc);

        let vertex_buffer = Box::new(MockBuffer { size: 144 });

        let _pass = VertexBufferTrianglePass::new(&mut graph, color_buffer, vertex_buffer);

        // Graph should compile successfully
        let compiled = graph.compile().unwrap();
        assert_eq!(compiled.execution_order.len(), 1);
    }
}
