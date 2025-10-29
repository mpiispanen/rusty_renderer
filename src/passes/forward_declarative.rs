//! Declarative forward rendering pass
//!
//! This is a refactored version of the forward pass that uses the
//! DeclarativePass trait for cleaner resource management and automatic
//! pipeline configuration.

use crate::backends::Buffer;
use crate::camera::CameraUniforms;
use crate::lighting::LightingUniforms;
use crate::render_graph::{
    DeclarativePass, PassBuilder, PassExecutionContext,
    PassKind, PassPreparationContext, PipelineBuilder, PipelineStage, ResourceId, ShaderRegistry,
};
use crate::scene::Transform;
use std::sync::Arc;

/// Declarative forward rendering pass with lighting
///
/// This pass renders 3D geometry using:
/// - Camera uniforms (MVP matrices)
/// - Lighting uniforms (lights + ambient)
/// - Vertex data (position, normal, uv, color)
///
/// # Resources
/// - **Output**: Color attachment (declared in constructor)
/// - **Input**: Vertex buffer, Camera buffer, Lighting buffer, optional Material and Texture
///
/// # Example
/// ```no_run
/// use rusty_renderer::passes::ForwardDeclarativePass;
/// use rusty_renderer::render_graph::RenderGraph;
/// # fn example(color_output: rusty_renderer::render_graph::ResourceId) {
/// # let vertex_buffer = todo!();
/// # let camera_buffer = todo!();
/// # let lighting_buffer = todo!();
/// # let transform = rusty_renderer::scene::Transform::default();
/// let forward_pass = ForwardDeclarativePass::new(
///     color_output,
///     vertex_buffer,
///     camera_buffer,
///     lighting_buffer,
///     None, // material
///     None, // texture
///     transform,
///     36, // vertex count
/// );
/// # }
/// ```
pub struct ForwardDeclarativePass {
    /// Color output resource
    color_output: ResourceId,
    
    /// Vertex buffer containing geometry
    vertex_buffer: Arc<Box<dyn Buffer>>,
    
    /// Camera uniform buffer (shared)
    camera_buffer: Arc<Box<dyn Buffer>>,
    
    /// Lighting uniform buffer (shared)
    lighting_buffer: Arc<Box<dyn Buffer>>,
    
    /// Optional material uniform buffer
    material_buffer: Option<Arc<Box<dyn Buffer>>>,
    
    /// Optional texture
    texture: Option<Arc<Box<dyn crate::backends::Texture>>>,
    
    /// Object transform (position, rotation, scale)
    transform: Transform,
    
    /// Number of vertices to draw
    vertex_count: u32,
}

impl ForwardDeclarativePass {
    /// Create a new declarative forward rendering pass
    ///
    /// # Arguments
    /// * `color_output` - The color attachment resource to render to
    /// * `vertex_buffer` - The vertex buffer containing geometry data
    /// * `camera_buffer` - Shared buffer containing camera uniforms
    /// * `lighting_buffer` - Shared buffer containing lighting uniforms
    /// * `material_buffer` - Optional material uniform buffer
    /// * `texture` - Optional texture for sampling
    /// * `transform` - Object transform (position, rotation, scale)
    /// * `vertex_count` - Number of vertices to draw
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        color_output: ResourceId,
        vertex_buffer: Box<dyn Buffer>,
        camera_buffer: Arc<Box<dyn Buffer>>,
        lighting_buffer: Arc<Box<dyn Buffer>>,
        material_buffer: Option<Arc<Box<dyn Buffer>>>,
        texture: Option<Arc<Box<dyn crate::backends::Texture>>>,
        transform: Transform,
        vertex_count: u32,
    ) -> Self {
        Self {
            color_output,
            vertex_buffer: Arc::new(vertex_buffer),
            camera_buffer,
            lighting_buffer,
            material_buffer,
            texture,
            transform,
            vertex_count,
        }
    }
}

impl DeclarativePass for ForwardDeclarativePass {
    fn name(&self) -> &str {
        "forward_declarative"
    }

    fn kind(&self) -> PassKind {
        PassKind::Graphics
    }

    fn declare_dependencies(&self, builder: &mut PassBuilder) {
        // Declare color output attachment
        builder.write(
            self.color_output,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
        );

        // Note: In the future, vertex/uniform buffers would be declared here
        // as resources managed by the graph. For now, they're passed directly
        // to maintain compatibility with the existing architecture.
    }

    fn declare_pipeline(&self, builder: &mut PipelineBuilder, registry: &ShaderRegistry) {
        // Get shaders from registry
        // For now, we'll use compile-time includes as fallback
        // TODO: Register shaders in the registry during app setup
        
        use crate::render_graph::CullMode;
        
        // Try to get shaders from registry (using get_handle which returns Result)
        if let Ok(vs_handle) = registry.get_handle("forward.vert") {
            builder.vertex_shader(vs_handle);
        } else {
            log::warn!("forward.vert not in shader registry, pipeline will be incomplete");
        }
        
        if let Ok(fs_handle) = registry.get_handle("forward.frag") {
            builder.fragment_shader(fs_handle);
        } else {
            log::warn!("forward.frag not in shader registry, pipeline will be incomplete");
        }
        
        // Configure pipeline state
        // Note: VertexLayout is a complex struct; for now we skip it and let
        // the backend handle vertex format. In the future, we should properly
        // define the layout here.
        builder
            .depth_test(true)
            .depth_write(true)
            .cull_mode(CullMode::Back);
    }

    fn prepare(&self, context: &mut dyn PassPreparationContext) {
        log::info!("Preparing forward declarative pass");

        // Prepare camera uniforms (set 0, binding 0)
        let camera_ptr =
            self.camera_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;
        let camera_size = std::mem::size_of::<CameraUniforms>() as u64;

        if let Err(e) = context.prepare_uniform_buffer(0, 0, camera_ptr, 0, camera_size) {
            log::error!("Failed to prepare camera uniforms: {e}");
            return;
        }

        // Prepare lighting uniforms (set 0, binding 1)
        let lighting_ptr =
            self.lighting_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;
        let lighting_size = std::mem::size_of::<LightingUniforms>() as u64;

        if let Err(e) = context.prepare_uniform_buffer(0, 1, lighting_ptr, 0, lighting_size) {
            log::error!("Failed to prepare lighting uniforms: {e}");
            return;
        }

        // Prepare material uniforms (set 0, binding 3) if available
        if let Some(ref material_buffer) = self.material_buffer {
            let material_ptr =
                material_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;
            let material_size = 32u64; // GpuMaterial size

            if let Err(e) = context.prepare_uniform_buffer(0, 3, material_ptr, 0, material_size) {
                log::error!("Failed to prepare material uniforms: {e}");
                return;
            }
        }

        // Prepare texture (set 0, binding 2) if available
        if let Some(ref texture) = self.texture {
            let texture_ptr = texture.as_ref().as_ref() as *const dyn crate::backends::Texture
                as *const std::ffi::c_void;

            if let Err(e) = context.prepare_texture(0, 2, texture_ptr) {
                log::error!("Failed to prepare texture: {e}");
                return;
            }
        }

        log::info!("Forward declarative pass preparation complete");
    }

    fn execute(&self, context: &mut dyn PassExecutionContext) {
        log::info!(
            "Executing forward declarative pass with {} vertices",
            self.vertex_count
        );

        // Push model and normal matrices as push constants
        let model_matrix = self.transform.matrix();
        let normal_matrix = self.transform.normal_matrix();

        log::debug!(
            "Model matrix: [{:.2}, {:.2}, {:.2}, {:.2}]",
            model_matrix[0][0],
            model_matrix[0][1],
            model_matrix[0][2],
            model_matrix[0][3]
        );

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
        log::debug!("Push constants uploaded (model + normal matrices)");

        // Bind camera uniforms (set 0, binding 0)
        let camera_ptr =
            self.camera_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;
        let camera_size = std::mem::size_of::<CameraUniforms>() as u64;

        if let Err(e) = context.bind_uniform_buffer(0, 0, camera_ptr, 0, camera_size) {
            log::error!("Failed to bind camera uniforms: {e}");
            return;
        }

        // Bind lighting uniforms (set 0, binding 1)
        let lighting_ptr =
            self.lighting_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;
        let lighting_size = std::mem::size_of::<LightingUniforms>() as u64;

        if let Err(e) = context.bind_uniform_buffer(0, 1, lighting_ptr, 0, lighting_size) {
            log::error!("Failed to bind lighting uniforms: {e}");
            return;
        }

        // Bind material uniforms (set 0, binding 3) if available
        if let Some(ref material_buffer) = self.material_buffer {
            let material_ptr =
                material_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;
            let material_size = 32u64; // GpuMaterial size

            if let Err(e) = context.bind_uniform_buffer(0, 3, material_ptr, 0, material_size) {
                log::error!("Failed to bind material uniforms: {e}");
            }
        }

        // Bind texture (set 0, binding 2) if available
        if let Some(ref texture) = self.texture {
            let texture_ptr = texture.as_ref().as_ref() as *const dyn crate::backends::Texture
                as *const std::ffi::c_void;

            if let Err(e) = context.bind_texture(0, 2, texture_ptr) {
                log::error!("Failed to bind texture: {e}");
            }
        }

        // Bind vertex buffer
        let vertex_ptr =
            self.vertex_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;

        if let Err(e) = context.bind_vertex_buffer(0, vertex_ptr, 0) {
            log::error!("Failed to bind vertex buffer: {e}");
            return;
        }

        // Draw vertices
        if let Err(e) = context.draw(self.vertex_count, 1, 0, 0) {
            log::error!("Failed to draw: {e}");
            return;
        }

        log::info!("Forward declarative pass completed successfully");
    }
}
