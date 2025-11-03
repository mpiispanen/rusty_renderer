//! Forward rendering pass using render graph
//!
//! This pass implements forward rendering with lighting, following the same
//! pattern as VertexBufferTrianglePass. Resources are passed directly.

use crate::backends::{Buffer, Texture};
use crate::camera::CameraUniforms;
use crate::lighting::LightingUniforms;
use crate::render_graph::{
    AccessType, ImageLayout, PassCallback, PassExecutionContext, PassId, PassKind,
    PassPreparationContext, PipelineBuilder, PipelineStage, RenderGraph, RenderPass,
    ResourceAccess, ResourceId, ShaderRegistry,
};
use crate::scene::Transform;
use anyhow::Result;
use std::sync::Arc;

/// Forward rendering pass for render graph
///
/// This pass renders 3D geometry with lighting. It follows the vertex buffer triangle
/// pattern: resources are created externally and passed in.
///
/// # Example
/// ```no_run
/// use rusty_renderer::passes::ForwardRenderPass;
/// use rusty_renderer::render_graph::RenderGraph;
/// use std::sync::Arc;
/// # fn example(
/// #     graph: &mut RenderGraph,
/// #     color_output: rusty_renderer::render_graph::ResourceId,
/// #     vertex_buffer: Box<dyn rusty_renderer::backends::Buffer>,
/// #     camera_buffer: Arc<Box<dyn rusty_renderer::backends::Buffer>>,
/// #     lighting_buffer: Arc<Box<dyn rusty_renderer::backends::Buffer>>,
/// # ) {
/// use rusty_renderer::scene::Transform;
/// let pass = ForwardRenderPass::builder()
///     .color_output(color_output)
///     .vertex_buffer(vertex_buffer)
///     .camera_buffer(camera_buffer)
///     .lighting_buffer(lighting_buffer)
///     .vertex_count(36)
///     .transform(Transform::default())
///     .build(graph)
///     .unwrap();
/// # }
/// ```
pub struct ForwardRenderPass {
    pass_id: PassId,
}

impl ForwardRenderPass {
    /// Get the pass ID
    pub fn pass_id(&self) -> PassId {
        self.pass_id
    }

    /// Create a builder for the pass
    pub fn builder() -> ForwardRenderPassBuilder {
        ForwardRenderPassBuilder::new()
    }
}

/// Builder for ForwardRenderPass
pub struct ForwardRenderPassBuilder {
    color_output: Option<ResourceId>,
    depth_output: Option<ResourceId>,
    vertex_buffer: Option<Box<dyn Buffer>>,
    camera_buffer: Option<Arc<Box<dyn Buffer>>>,
    lighting_buffer: Option<Arc<Box<dyn Buffer>>>,
    material_buffer: Option<Arc<Box<dyn Buffer>>>,
    texture: Option<Arc<Box<dyn Texture>>>,
    transform: Transform,
    vertex_count: u32,
    name: String,
}

impl ForwardRenderPassBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            color_output: None,
            depth_output: None,
            vertex_buffer: None,
            camera_buffer: None,
            lighting_buffer: None,
            material_buffer: None,
            texture: None,
            transform: Transform::default(),
            vertex_count: 0,
            name: "forward_render".to_string(),
        }
    }

    /// Set color output attachment
    pub fn color_output(mut self, resource: ResourceId) -> Self {
        self.color_output = Some(resource);
        self
    }

    /// Set optional depth output attachment
    pub fn depth_output(mut self, resource: ResourceId) -> Self {
        self.depth_output = Some(resource);
        self
    }

    /// Set vertex buffer
    pub fn vertex_buffer(mut self, buffer: Box<dyn Buffer>) -> Self {
        self.vertex_buffer = Some(buffer);
        self
    }

    /// Set camera uniforms buffer
    pub fn camera_buffer(mut self, buffer: Arc<Box<dyn Buffer>>) -> Self {
        self.camera_buffer = Some(buffer);
        self
    }

    /// Set lighting uniforms buffer
    pub fn lighting_buffer(mut self, buffer: Arc<Box<dyn Buffer>>) -> Self {
        self.lighting_buffer = Some(buffer);
        self
    }

    /// Set optional material uniforms buffer
    pub fn material_buffer(mut self, buffer: Arc<Box<dyn Buffer>>) -> Self {
        self.material_buffer = Some(buffer);
        self
    }

    /// Set optional texture
    pub fn texture(mut self, texture: Arc<Box<dyn Texture>>) -> Self {
        self.texture = Some(texture);
        self
    }

    /// Set object transform
    pub fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Set vertex count
    pub fn vertex_count(mut self, count: u32) -> Self {
        self.vertex_count = count;
        self
    }

    /// Set custom pass name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Build the pass and add it to the render graph
    pub fn build(self, graph: &mut RenderGraph) -> Result<ForwardRenderPass> {
        // Validate required resources
        let color_output = self
            .color_output
            .ok_or_else(|| anyhow::anyhow!("color_output is required"))?;
        let vertex_buffer = self
            .vertex_buffer
            .ok_or_else(|| anyhow::anyhow!("vertex_buffer is required"))?;
        let camera_buffer = self
            .camera_buffer
            .ok_or_else(|| anyhow::anyhow!("camera_buffer is required"))?;
        let lighting_buffer = self
            .lighting_buffer
            .ok_or_else(|| anyhow::anyhow!("lighting_buffer is required"))?;

        if self.vertex_count == 0 {
            return Err(anyhow::anyhow!("vertex_count must be greater than 0"));
        }

        let pass_id = graph.next_pass_id();
        let mut pass = RenderPass::new(pass_id, &self.name, PassKind::Graphics);

        // Declare dependencies
        // Output: Color attachment
        pass.add_output(ResourceAccess::new(
            color_output,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            Some(ImageLayout::ColorAttachment),
        ));

        // Optional depth attachment
        if let Some(depth_output) = self.depth_output {
            pass.add_output(ResourceAccess::new(
                depth_output,
                AccessType::Write,
                PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
                Some(ImageLayout::DepthStencilAttachment),
            ));
        }

        // Set up execution callback
        let callback = ForwardRenderPassCallback {
            vertex_buffer: Arc::new(vertex_buffer),
            camera_buffer,
            lighting_buffer,
            _material_buffer: self.material_buffer,
            _texture: self.texture,
            transform: self.transform,
            vertex_count: self.vertex_count,
        };

        pass = pass.with_callback(Box::new(callback));
        graph.add_pass(pass);

        Ok(ForwardRenderPass { pass_id })
    }
}

impl Default for ForwardRenderPassBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution callback for the forward rendering pass
struct ForwardRenderPassCallback {
    vertex_buffer: Arc<Box<dyn Buffer>>,
    camera_buffer: Arc<Box<dyn Buffer>>,
    lighting_buffer: Arc<Box<dyn Buffer>>,
    _material_buffer: Option<Arc<Box<dyn Buffer>>>,
    _texture: Option<Arc<Box<dyn Texture>>>,
    transform: Transform,
    vertex_count: u32,
}

impl PassCallback for ForwardRenderPassCallback {
    fn declare_pipeline(&self, builder: &mut PipelineBuilder, registry: &ShaderRegistry) {
        use crate::render_graph::{
            CullMode, InputRate, VertexAttribute, VertexBinding, VertexFormat, VertexLayout,
        };

        // Get shaders from registry
        let vs_handle = registry
            .get_handle("forward.vert")
            .expect("forward.vert not found in shader registry");
        let fs_handle = registry
            .get_handle("forward.frag")
            .expect("forward.frag not found in shader registry");

        // Define vertex layout to match the Vertex struct:
        // struct Vertex {
        //     position: [f32; 3],  // offset 0, 12 bytes
        //     normal: [f32; 3],    // offset 12, 12 bytes
        //     uv: [f32; 2],        // offset 24, 8 bytes
        //     color: [f32; 4],     // offset 32, 16 bytes
        // }                        // total: 48 bytes
        let vertex_layout = VertexLayout {
            attributes: vec![
                VertexAttribute {
                    location: 0,
                    format: VertexFormat::Float32x3,
                    offset: 0, // position
                },
                VertexAttribute {
                    location: 1,
                    format: VertexFormat::Float32x3,
                    offset: 12, // normal
                },
                VertexAttribute {
                    location: 2,
                    format: VertexFormat::Float32x2,
                    offset: 24, // uv
                },
                VertexAttribute {
                    location: 3,
                    format: VertexFormat::Float32x4,
                    offset: 32, // color
                },
            ],
            bindings: vec![VertexBinding {
                binding: 0,
                stride: 48, // total size of Vertex
                input_rate: InputRate::Vertex,
            }],
        };

        builder
            .vertex_shader(vs_handle)
            .fragment_shader(fs_handle)
            .vertex_layout(vertex_layout)
            .depth_test(true)
            .depth_write(true)
            .cull_mode(CullMode::Back);
    }

    fn prepare(&self, _context: &mut dyn PassPreparationContext) {
        log::debug!("Preparing forward render pass");
        // Resource preparation is handled by backend
    }

    fn execute(&self, context: &mut dyn PassExecutionContext) {
        log::debug!(
            "Executing forward render pass with {} vertices",
            self.vertex_count
        );

        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open("rusty_renderer_debug.log")
        {
            use std::io::Write;
            let _ = writeln!(f, "ForwardPass::execute - START");
        }

        // Push model and normal matrices as push constants
        let model_matrix = self.transform.matrix();
        let normal_matrix = self.transform.normal_matrix();

        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open("rusty_renderer_debug.log")
        {
            use std::io::Write;
            let _ = writeln!(f, "ForwardPass::execute - matrices computed");
        }

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

        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open("rusty_renderer_debug.log")
        {
            use std::io::Write;
            let _ = writeln!(
                f,
                "ForwardPass::execute - push data prepared, {} bytes",
                push_data.len()
            );
        }

        // Push constants to vertex shader (stage flag 0x1 = VERTEX)
        if let Err(e) = context.push_constants(0x1, 0, &push_data) {
            log::error!("Failed to push constants: {e}");
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .append(true)
                .open("rusty_renderer_debug.log")
            {
                use std::io::Write;
                let _ = writeln!(f, "ForwardPass::execute - ERROR pushing constants: {e}");
            }
            return;
        }

        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open("rusty_renderer_debug.log")
        {
            use std::io::Write;
            let _ = writeln!(f, "ForwardPass::execute - push constants done");
        }

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
        // TODO: Re-enable when shader supports materials and textures
        // if let Some(ref material_buffer) = self.material_buffer {
        //     let material_ptr =
        //         material_buffer.as_ref().as_ref() as *const dyn Buffer as *const std::ffi::c_void;
        //     let material_size = 32u64; // GpuMaterial size
        //
        //     if let Err(e) = context.bind_uniform_buffer(0, 3, material_ptr, 0, material_size) {
        //         log::error!("Failed to bind material uniforms: {e}");
        //     }
        // }
        //
        // // Bind texture (set 0, binding 2) if available
        // if let Some(ref texture) = self.texture {
        //     let texture_ptr =
        //         texture.as_ref().as_ref() as *const dyn Texture as *const std::ffi::c_void;
        //
        //     if let Err(e) = context.bind_texture(0, 2, texture_ptr) {
        //         log::error!("Failed to bind texture: {e}");
        //     }
        // }

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

        log::debug!("Forward render pass executed successfully");
    }
}
