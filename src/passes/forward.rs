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
    /// * `material_buffer` - Optional material uniform buffer
    /// * `texture` - Optional texture for sampling
    /// * `transform` - Object transform (position, rotation, scale)
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
        material_buffer: Option<Arc<Box<dyn Buffer>>>,
        texture: Option<Arc<Box<dyn crate::backends::Texture>>>,
        transform: crate::scene::Transform,
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

        // Set up callback with all buffers and transform
        pass = pass.with_callback(Box::new(ForwardPassCallback {
            vertex_buffer: vertex_buffer_arc,
            camera_buffer,
            lighting_buffer,
            material_buffer,
            texture,
            transform,
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
    material_buffer: Option<Arc<Box<dyn Buffer>>>,
    texture: Option<Arc<Box<dyn crate::backends::Texture>>>,
    transform: crate::scene::Transform,
    vertex_count: u32,
}

impl PassCallback for ForwardPassCallback {
    fn execute(&self, context: &mut dyn PassExecutionContext) {
        log::info!("Executing forward rendering pass with {} vertices", self.vertex_count);

        // Push model and normal matrices as push constants
        let model_matrix = self.transform.matrix();
        let normal_matrix = self.transform.normal_matrix();
        
        log::info!("Model matrix: [{:.2}, {:.2}, {:.2}, {:.2}]", 
            model_matrix[0][0], model_matrix[0][1], model_matrix[0][2], model_matrix[0][3]);
        log::info!("Normal matrix: [{:.2}, {:.2}, {:.2}, {:.2}]",
            normal_matrix[0][0], normal_matrix[0][1], normal_matrix[0][2], normal_matrix[0][3]);
        
        // Combine both matrices into a single byte array (128 bytes total)
        let mut push_data = Vec::with_capacity(128);
        
        // Add model matrix (64 bytes)
        for row in &model_matrix {
            for &val in row {
                push_data.extend_from_slice(&val.to_ne_bytes());
            }
        }
        
        // Add normal matrix (64 bytes)
        for row in &normal_matrix {
            for &val in row {
                push_data.extend_from_slice(&val.to_ne_bytes());
            }
        }
        
        // Push constants to vertex shader (stage flag 0x1 = VERTEX)
        if let Err(e) = context.push_constants(0x1, 0, &push_data) {
            log::error!("Failed to push constants: {e}");
            return;
        }
        log::info!("Push constants uploaded (model + normal matrices)");

        // Bind camera uniforms (set 0, binding 0)
        let camera_ptr = self.camera_buffer.as_ref().as_ref() as *const dyn Buffer
            as *const std::ffi::c_void;
        let camera_size = std::mem::size_of::<CameraUniforms>() as u64;
        
        if let Err(e) = context.bind_uniform_buffer(0, 0, camera_ptr, 0, camera_size) {
            log::error!("Failed to bind camera uniforms: {e}");
            return;
        }
        log::info!("Camera uniforms bound");

        // Bind lighting uniforms (set 0, binding 1)
        let lighting_ptr = self.lighting_buffer.as_ref().as_ref() as *const dyn Buffer
            as *const std::ffi::c_void;
        let lighting_size = std::mem::size_of::<LightingUniforms>() as u64;
        
        if let Err(e) = context.bind_uniform_buffer(0, 1, lighting_ptr, 0, lighting_size) {
            log::error!("Failed to bind lighting uniforms: {e}");
            return;
        }
        log::info!("Lighting uniforms bound");

        // Bind material uniforms (set 0, binding 3) if available
        if let Some(ref material_buffer) = self.material_buffer {
            let material_ptr = material_buffer.as_ref().as_ref() as *const dyn Buffer
                as *const std::ffi::c_void;
            let material_size = 32u64; // GpuMaterial size
            
            if let Err(e) = context.bind_uniform_buffer(0, 3, material_ptr, 0, material_size) {
                log::error!("Failed to bind material uniforms: {e}");
                return;
            }
            log::info!("Material uniforms bound");
        } else {
            log::info!("No material - using default");
        }

        // Bind texture (set 0, binding 2) if available
        if let Some(ref texture) = self.texture {
            let texture_ptr = texture.as_ref().as_ref() as *const dyn crate::backends::Texture
                as *const std::ffi::c_void;
            
            if let Err(e) = context.bind_texture(0, 2, texture_ptr) {
                log::error!("Failed to bind texture: {e}");
                return;
            }
            log::info!("Texture bound");
        } else {
            log::info!("No texture - using base color only");
        }

        // Bind vertex buffer
        let vertex_ptr = self.vertex_buffer.as_ref().as_ref() as *const dyn Buffer
            as *const std::ffi::c_void;
        
        if let Err(e) = context.bind_vertex_buffer(0, vertex_ptr, 0) {
            log::error!("Failed to bind vertex buffer: {e}");
            return;
        }
        log::info!("Vertex buffer bound");

        // Draw vertices
        if let Err(e) = context.draw(self.vertex_count, 1, 0, 0) {
            log::error!("Failed to draw: {e}");
            return;
        }

        log::info!("Forward pass completed successfully");
    }
}
