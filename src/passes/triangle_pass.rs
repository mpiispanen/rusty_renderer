//! Triangle rendering pass
//!
//! A simple pass that renders a colored triangle. The triangle vertices
//! are hardcoded in the vertex shader.

#[cfg(test)]
use crate::render_graph::ExtentMode;
use crate::render_graph::{
    AccessType, ImageLayout, PassCallback, PassExecutionContext, PassId, PassKind, PipelineStage,
    RenderGraph, RenderPass, ResourceAccess, ResourceId,
};
use anyhow::Result;

/// Triangle rendering pass
///
/// This pass renders a single colored triangle to a color attachment.
/// The triangle geometry is hardcoded in the vertex shader, so no
/// vertex buffers are needed.
///
/// # Resources
/// - **Output**: Color attachment (swapchain image or offscreen buffer)
///
/// # Example
/// ```no_run
/// use rusty_renderer::passes::TrianglePass;
/// use rusty_renderer::render_graph::RenderGraph;
/// # fn example(mut graph: RenderGraph, color_buffer: rusty_renderer::render_graph::ResourceId) {
/// let triangle_pass = TrianglePass::new(&mut graph, color_buffer);
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
    ///
    /// # Returns
    /// A TrianglePass instance that can be used to configure the pass further
    pub fn new(graph: &mut RenderGraph, color_output: ResourceId) -> Self {
        let pass_id = graph.next_pass_id();

        let mut pass = RenderPass::new(pass_id, "triangle_pass", PassKind::Graphics);

        // Configure output: write to color buffer
        pass.add_output(ResourceAccess::new(
            color_output,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            Some(ImageLayout::ColorAttachment),
        ));

        // Set up callback
        pass = pass.with_callback(Box::new(TrianglePassCallback));

        graph.add_pass(pass);

        Self { pass_id }
    }

    /// Get the pass ID
    pub fn pass_id(&self) -> PassId {
        self.pass_id
    }
}

/// Callback for triangle pass execution
struct TrianglePassCallback;

impl PassCallback for TrianglePassCallback {
    fn declare_pipeline(
        &self,
        builder: &mut crate::render_graph::PipelineBuilder,
        registry: &crate::render_graph::ShaderRegistry,
    ) {
        // Load triangle shaders
        let vs = registry
            .get_handle("triangle.vert")
            .expect("Failed to load triangle vertex shader");
        let fs = registry
            .get_handle("triangle.frag")
            .expect("Failed to load triangle fragment shader");

        builder
            .vertex_shader(vs)
            .fragment_shader(fs)
            .depth_test(false)
            .depth_write(false)
            .cull_mode(crate::render_graph::CullMode::None);
    }

    fn execute(&self, _context: &mut dyn PassExecutionContext) {
        // The actual drawing is currently handled by the backend's execute_graph
        // In a full implementation, this would:
        // 1. Downcast context to backend-specific type (e.g., VulkanPassContext)
        // 2. Bind pipeline
        // 3. Set viewport/scissor
        // 4. Issue draw call: cmd_draw(3, 1, 0, 0)

        log::trace!("Triangle pass callback executed");
    }
}

/// Builder for triangle pass with more configuration options
pub struct TrianglePassBuilder {
    color_output: ResourceId,
    clear_color: Option<[f32; 4]>,
    name: String,
}

impl TrianglePassBuilder {
    /// Create a new triangle pass builder
    pub fn new(color_output: ResourceId) -> Self {
        Self {
            color_output,
            clear_color: None,
            name: "triangle_pass".to_string(),
        }
    }

    /// Set the clear color for the pass
    pub fn with_clear_color(mut self, color: [f32; 4]) -> Self {
        self.clear_color = Some(color);
        self
    }

    /// Set a custom name for the pass
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Build the pass and add it to the render graph
    pub fn build(self, graph: &mut RenderGraph) -> Result<TrianglePass> {
        let pass_id = graph.next_pass_id();

        let mut pass = RenderPass::new(pass_id, &self.name, PassKind::Graphics);

        pass.add_output(ResourceAccess::new(
            self.color_output,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            Some(ImageLayout::ColorAttachment),
        ));

        pass = pass.with_callback(Box::new(TrianglePassCallback));

        graph.add_pass(pass);

        Ok(TrianglePass { pass_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_graph::{Extent3D, Format, ImageUsageFlags, ResourceDescriptor, SampleCount};

    #[test]
    fn test_triangle_pass_creation() {
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

        // Create triangle pass
        let pass = TrianglePass::new(&mut graph, color_buffer);

        // Verify pass was created
        assert_eq!(pass.pass_id().0, 0);
    }

    #[test]
    fn test_triangle_pass_builder() {
        let mut graph = RenderGraph::new();

        let color_desc = ResourceDescriptor::Image {
            format: Format::Bgra8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(800, 600)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let color_buffer = graph.create_resource("color_buffer", color_desc);

        // Create triangle pass with builder
        let pass = TrianglePassBuilder::new(color_buffer)
            .with_clear_color([0.1, 0.2, 0.3, 1.0])
            .with_name("custom_triangle")
            .build(&mut graph)
            .unwrap();

        assert_eq!(pass.pass_id().0, 0);
    }

    #[test]
    fn test_triangle_pass_compiles() {
        use crate::render_graph::{ShaderDescriptor, ShaderStage};

        let mut graph = RenderGraph::new();

        // Register shaders first
        graph.register_shader(
            "triangle.vert",
            ShaderDescriptor::from_file("shaders/hlsl/triangle.hlsl", ShaderStage::Vertex)
                .with_entry_point("VSMain"),
        );
        graph.register_shader(
            "triangle.frag",
            ShaderDescriptor::from_file("shaders/hlsl/triangle.hlsl", ShaderStage::Fragment)
                .with_entry_point("PSMain"),
        );

        let color_desc = ResourceDescriptor::Image {
            format: Format::Bgra8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(800, 600)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let color_buffer = graph.create_resource("color_buffer", color_desc);

        let _pass = TrianglePass::new(&mut graph, color_buffer);

        // Graph should compile successfully
        let compiled = graph.compile().unwrap();
        assert_eq!(compiled.execution_order.len(), 1);
        // Should have pipeline description for triangle pass
        assert_eq!(compiled.pipeline_descriptions.len(), 1);
    }
}
