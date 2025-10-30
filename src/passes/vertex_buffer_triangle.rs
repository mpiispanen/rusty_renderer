//! Vertex buffer triangle rendering pass (M9)
//!
//! A triangle rendering pass that uses actual vertex buffers instead of
//! hardcoded vertices in the shader. This demonstrates proper vertex buffer
//! usage through the render graph system.

use crate::backends::Buffer;
#[cfg(test)]
use crate::render_graph::ExtentMode;
use crate::render_graph::{
    AccessType, ImageLayout, IndexType, PassCallback, PassExecutionContext, PassId, PassKind,
    PipelineStage, RenderGraph, RenderPass, ResourceAccess, ResourceId,
};
use anyhow::Result;
use std::sync::Arc;

/// Vertex buffer triangle rendering pass
///
/// This pass renders geometry using vertex data from a GPU buffer.
/// The vertex buffer is provided during construction and is owned by the pass
/// through an Arc for shared ownership. Optionally supports indexed drawing.
///
/// # Resources
/// - **Output**: Color attachment (swapchain image or offscreen buffer)
/// - **Input**: Vertex buffer (provided at construction)
/// - **Input (optional)**: Index buffer for indexed drawing
///
/// # Example
/// ```no_run
/// use rusty_renderer::passes::VertexBufferTrianglePass;
/// use rusty_renderer::render_graph::RenderGraph;
/// # fn example(mut graph: RenderGraph, color_buffer: rusty_renderer::render_graph::ResourceId, vertex_buffer: Box<dyn rusty_renderer::backends::Buffer>) {
/// let triangle_pass = VertexBufferTrianglePass::new(&mut graph, color_buffer, vertex_buffer, None, 3);
/// # }
/// ```
pub struct VertexBufferTrianglePass {
    pass_id: PassId,
}

impl VertexBufferTrianglePass {
    /// Create a new vertex buffer rendering pass
    ///
    /// # Arguments
    /// * `graph` - The render graph to add the pass to
    /// * `color_output` - The color attachment resource to render to
    /// * `vertex_buffer` - The vertex buffer containing vertex data
    /// * `index_buffer` - Optional index buffer for indexed drawing
    /// * `draw_count` - Number of vertices/indices to draw
    ///
    /// # Returns
    /// A VertexBufferTrianglePass instance
    pub fn new(
        graph: &mut RenderGraph,
        color_output: ResourceId,
        vertex_buffer: Box<dyn Buffer>,
        index_buffer: Option<Box<dyn Buffer>>,
        draw_count: u32,
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

        // Wrap buffers in Arc for shared ownership
        let vertex_buffer_arc = Arc::new(vertex_buffer);
        let index_buffer_arc = index_buffer.map(Arc::new);

        // Set up callback with vertex buffer
        pass = pass.with_callback(Box::new(VertexBufferTriangleCallback {
            vertex_buffer: vertex_buffer_arc,
            index_buffer: index_buffer_arc,
            draw_count,
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
    index_buffer: Option<Arc<Box<dyn Buffer>>>,
    draw_count: u32,
}

impl PassCallback for VertexBufferTriangleCallback {
    fn declare_pipeline(
        &self,
        builder: &mut crate::render_graph::PipelineBuilder,
        registry: &crate::render_graph::ShaderRegistry,
    ) {
        use crate::render_graph::{
            InputRate, VertexAttribute, VertexBinding, VertexFormat, VertexLayout,
        };

        // Load simple vertex shaders that read from vertex buffers
        let vs = registry
            .get_handle("simple_vertex.vert")
            .expect("Failed to load simple vertex shader");
        let fs = registry
            .get_handle("simple_vertex.frag")
            .expect("Failed to load simple fragment shader");

        // Define vertex layout (position + color)
        // Vertex format: position (3 floats) + color (3 floats) = 24 bytes
        let mut layout = VertexLayout::new();
        layout.add_binding(VertexBinding {
            binding: 0,
            stride: 24, // 6 floats * 4 bytes each
            input_rate: InputRate::Vertex,
        });
        layout.add_attribute(VertexAttribute {
            location: 0,
            format: VertexFormat::Float32x3,
            offset: 0, // position
        });
        layout.add_attribute(VertexAttribute {
            location: 1,
            format: VertexFormat::Float32x3,
            offset: 12, // color (after 3 floats)
        });

        // Configure pipeline
        builder
            .vertex_shader(vs)
            .fragment_shader(fs)
            .vertex_layout(layout)
            .depth_test(false)
            .depth_write(false)
            .cull_mode(crate::render_graph::CullMode::Back);
    }

    fn prepare(&self, _context: &mut dyn crate::render_graph::PassPreparationContext) {
        // For now, just call the default preparation which should set up minimal bind groups
        log::info!("VertexBufferTriangleCallback::prepare called");
        // Suboptimal: We need to ensure bind groups are created even if we have no uniforms/textures
        // The wgpu backend should create empty/default bind groups
    }

    fn execute(&self, context: &mut dyn PassExecutionContext) {
        log::debug!("Executing vertex buffer pass");

        // Create MVP matrix (Model-View-Projection)
        // Simple perspective projection looking at a cube from an angle
        let aspect = 800.0 / 600.0;
        let fov = 60.0f32.to_radians();
        let near = 0.1;
        let far = 100.0;

        // Perspective projection matrix
        let f = 1.0 / (fov / 2.0).tan();
        let projection = [
            f / aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            f,
            0.0,
            0.0,
            0.0,
            0.0,
            (far + near) / (near - far),
            -1.0,
            0.0,
            0.0,
            (2.0 * far * near) / (near - far),
            0.0,
        ];

        // View matrix (camera at [2, 2, 3] looking at origin)
        let eye = [2.0f32, 2.0, 3.0];
        let center = [0.0f32, 0.0, 0.0];
        let up = [0.0f32, 1.0, 0.0];

        // Simple lookAt calculation
        let f_x = center[0] - eye[0];
        let f_y = center[1] - eye[1];
        let f_z = center[2] - eye[2];
        let f_len = (f_x * f_x + f_y * f_y + f_z * f_z).sqrt();
        let f = [f_x / f_len, f_y / f_len, f_z / f_len];

        let s_x = f[1] * up[2] - f[2] * up[1];
        let s_y = f[2] * up[0] - f[0] * up[2];
        let s_z = f[0] * up[1] - f[1] * up[0];
        let s_len = (s_x * s_x + s_y * s_y + s_z * s_z).sqrt();
        let s = [s_x / s_len, s_y / s_len, s_z / s_len];

        let u_x = s[1] * f[2] - s[2] * f[1];
        let u_y = s[2] * f[0] - s[0] * f[2];
        let u_z = s[0] * f[1] - s[1] * f[0];

        let view = [
            s[0],
            u_x,
            -f[0],
            0.0,
            s[1],
            u_y,
            -f[1],
            0.0,
            s[2],
            u_z,
            -f[2],
            0.0,
            -(s[0] * eye[0] + s[1] * eye[1] + s[2] * eye[2]),
            -(u_x * eye[0] + u_y * eye[1] + u_z * eye[2]),
            f[0] * eye[0] + f[1] * eye[1] + f[2] * eye[2],
            1.0,
        ];

        // MVP = Projection * View (no model transform for now)
        // Matrix multiply: projection * view
        let mut mvp = [0.0f32; 16];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    mvp[i * 4 + j] += projection[i * 4 + k] * view[k * 4 + j];
                }
            }
        }

        // Push the MVP matrix as push constants (vertex shader stage)
        let mvp_bytes = unsafe { std::slice::from_raw_parts(mvp.as_ptr() as *const u8, 64) };
        // VERTEX_SHADER = 0x01
        if let Err(e) = context.push_constants(0x01, 0, mvp_bytes) {
            log::error!("Failed to push MVP matrix: {e}");
            return;
        }

        // Get raw pointer from the buffer for backend API
        // Arc<Box<dyn Buffer>> -> &dyn Buffer -> *const dyn Buffer -> *const c_void
        let buffer_ptr =
            self.vertex_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;

        // Bind the vertex buffer
        if let Err(e) = context.bind_vertex_buffer(0, buffer_ptr, 0) {
            log::error!("Failed to bind vertex buffer: {e}");
            return;
        }

        // If we have an index buffer, bind it and use indexed drawing
        if let Some(ref index_buffer) = self.index_buffer {
            let index_ptr =
                index_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;

            // We use 32-bit indices (u32)
            if let Err(e) = context.bind_index_buffer(index_ptr, 0, IndexType::U32) {
                log::error!("Failed to bind index buffer: {e}");
                return;
            }

            // Draw using indices
            if let Err(e) = context.draw_indexed(self.draw_count, 1, 0, 0, 0) {
                log::error!("Failed to draw indexed: {e}");
                return;
            }

            log::debug!(
                "Vertex buffer drawn successfully with {} indices",
                self.draw_count
            );
        } else {
            // Draw without indices
            if let Err(e) = context.draw(self.draw_count, 1, 0, 0) {
                log::error!("Failed to draw: {e}");
                return;
            }

            log::debug!(
                "Vertex buffer drawn successfully with {} vertices",
                self.draw_count
            );
        }
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
        let pass = VertexBufferTrianglePass::new(&mut graph, color_buffer, vertex_buffer, None, 3);

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
        use crate::render_graph::{ShaderDescriptor, ShaderStage};

        let mut graph = RenderGraph::new();

        // Register shaders
        graph.register_shader(
            "simple_vertex.vert",
            ShaderDescriptor::from_file("shaders/hlsl/simple_vertex.hlsl", ShaderStage::Vertex)
                .with_entry_point("VSMain"),
        );
        graph.register_shader(
            "simple_vertex.frag",
            ShaderDescriptor::from_file("shaders/hlsl/simple_vertex.hlsl", ShaderStage::Fragment)
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

        let vertex_buffer = Box::new(MockBuffer { size: 144 });

        let _pass = VertexBufferTrianglePass::new(&mut graph, color_buffer, vertex_buffer, None, 3);

        // Graph should compile successfully
        let compiled = graph.compile().unwrap();
        assert_eq!(compiled.execution_order.len(), 1);
        // Should have pipeline description
        assert_eq!(compiled.pipeline_descriptions.len(), 1);
    }
}
