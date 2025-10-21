//! Forward rendering pass with lighting (M10 Phase 3)
//!
//! A rendering pass that uses camera and lighting uniforms to render
//! 3D geometry with Blinn-Phong lighting.

use crate::backends::Buffer;
use crate::camera::CameraUniforms;
use crate::lighting::LightingUniforms;
use crate::render_graph::{
    AccessType, ImageLayout, PassCallback, PassExecutionContext, PassId, PassKind, PipelineStage,
    RenderGraph, RenderPass, ResourceAccess, ResourceId,
};
use std::sync::Arc;

/// Forward rendering pass with lighting
///
/// This pass renders 3D geometry using:
/// - Camera uniforms (MVP matrices)
/// - Lighting uniforms (lights + ambient)
/// - Vertex data (position, normal, uv, color)
///
/// # Resources
/// - **Output**: Color attachment
/// - **Input**: Vertex buffer, Camera uniform buffer, Lighting uniform buffer
pub struct ForwardPass {
    pass_id: PassId,
}

impl ForwardPass {
    /// Create a new forward rendering pass
    ///
    /// # Arguments
    /// * `graph` - The render graph to add the pass to
    /// * `color_output` - The color attachment resource to render to
    /// * `vertex_buffer` - The vertex buffer containing geometry data
    /// * `camera_buffer` - Shared buffer containing camera uniforms
    /// * `lighting_buffer` - Shared buffer containing lighting uniforms
    /// * `vertex_count` - Number of vertices to draw
    ///
    /// # Returns
    /// A ForwardPass instance
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        graph: &mut RenderGraph,
        color_output: ResourceId,
        vertex_buffer: Box<dyn Buffer>,
        camera_buffer: Arc<Box<dyn Buffer>>,
        lighting_buffer: Arc<Box<dyn Buffer>>,
        vertex_count: u32,
    ) -> Self {
        let pass_id = graph.next_pass_id();

        let mut pass = RenderPass::new(pass_id, "forward_rendering", PassKind::Graphics);

        // Configure output: write to color buffer
        pass.add_output(ResourceAccess::new(
            color_output,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            Some(ImageLayout::ColorAttachment),
        ));

        // Wrap vertex buffer in Arc, use provided Arc for uniforms
        let vertex_buffer_arc = Arc::new(vertex_buffer);

        // Set up callback with all buffers
        pass = pass.with_callback(Box::new(ForwardPassCallback {
            vertex_buffer: vertex_buffer_arc,
            camera_buffer,
            lighting_buffer,
            vertex_count,
        }));

        graph.add_pass(pass);

        Self { pass_id }
    }

    /// Get the pass ID
    pub fn pass_id(&self) -> PassId {
        self.pass_id
    }
}

/// Callback for forward rendering pass execution
struct ForwardPassCallback {
    vertex_buffer: Arc<Box<dyn Buffer>>,
    camera_buffer: Arc<Box<dyn Buffer>>,
    lighting_buffer: Arc<Box<dyn Buffer>>,
    vertex_count: u32,
}

impl PassCallback for ForwardPassCallback {
    fn execute(&self, context: &mut dyn PassExecutionContext) {
        log::debug!("Executing forward rendering pass");

        // Bind camera uniforms (set 0, binding 0)
        let camera_ptr = self.camera_buffer.as_ref().as_ref() as *const dyn Buffer
            as *const std::ffi::c_void;
        let camera_size = std::mem::size_of::<CameraUniforms>() as u64;
        
        if let Err(e) = context.bind_uniform_buffer(0, 0, camera_ptr, 0, camera_size) {
            log::error!("Failed to bind camera uniforms: {e}");
            return;
        }
        log::debug!("Camera uniforms bound");

        // Bind lighting uniforms (set 0, binding 1)
        let lighting_ptr = self.lighting_buffer.as_ref().as_ref() as *const dyn Buffer
            as *const std::ffi::c_void;
        let lighting_size = std::mem::size_of::<LightingUniforms>() as u64;
        
        if let Err(e) = context.bind_uniform_buffer(0, 1, lighting_ptr, 0, lighting_size) {
            log::error!("Failed to bind lighting uniforms: {e}");
            return;
        }
        log::debug!("Lighting uniforms bound");

        // Bind vertex buffer
        let vertex_ptr = self.vertex_buffer.as_ref().as_ref() as *const dyn Buffer
            as *const std::ffi::c_void;
        
        if let Err(e) = context.bind_vertex_buffer(0, vertex_ptr, 0) {
            log::error!("Failed to bind vertex buffer: {e}");
            return;
        }
        log::debug!("Vertex buffer bound");

        // Draw vertices
        if let Err(e) = context.draw(self.vertex_count, 1, 0, 0) {
            log::error!("Failed to draw: {e}");
            return;
        }

        log::debug!("Forward pass drawn {} vertices successfully", self.vertex_count);
    }
}
