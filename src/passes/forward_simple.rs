//! Simplified forward rendering pass with full render graph integration
//!
//! This pass demonstrates the target architecture where ALL resources are
//! managed by the render graph. No buffers or textures are passed directly.

use crate::camera::CameraController;
use crate::lighting::Lighting;
use crate::render_graph::IndexType;
use crate::render_graph::{
    AccessType, ImageLayout, PassCallback, PassExecutionContext, PassId, PassKind,
    PassPreparationContext, PipelineBuilder, PipelineStage, RenderGraph, RenderPass,
    ResourceAccess, ResourceId, ShaderRegistry,
};
use crate::scene::{GeometryData, Scene, SceneObject, Transform};
use anyhow::Result;

/// Simplified forward rendering pass
///
/// This pass renders 3D geometry using resources managed entirely by the render graph:
/// - Color and depth attachments (Images)
/// - Camera uniforms (Buffer)
/// - Lighting uniforms (Buffer)
/// - Vertex data (Buffer)
/// - Optional material uniforms (Buffer)
/// - Optional albedo texture (Image)
///
/// # Example
/// ```no_run
/// use rusty_renderer::passes::ForwardSimplePass;
/// use rusty_renderer::render_graph::{RenderGraph, ResourceDescriptor};
/// # fn example(graph: &mut RenderGraph) {
/// # use rusty_renderer::scene::Transform;
/// # let color_output = graph.create_resource("color", todo!());
/// # let depth_output = graph.create_resource("depth", todo!());
/// # let vertex_buffer = graph.create_resource("vertices", todo!());
/// # let camera_buffer = graph.create_resource("camera", todo!());
/// # let lighting_buffer = graph.create_resource("lighting", todo!());
///
/// let pass = ForwardSimplePass::builder()
///     .color_output(color_output)
///     .depth_output(depth_output)
///     .vertex_buffer(vertex_buffer)
///     .camera_buffer(camera_buffer)
///     .lighting_buffer(lighting_buffer)
///     .transform(Transform::default())
///     .vertex_count(36)
///     .build(graph)
///     .unwrap();
/// # }
/// ```
#[allow(dead_code)] // Fields will be used when full execution is implemented
pub struct ForwardSimplePass {
    pass_id: PassId,
    transform: Transform,
    vertex_count: u32,
}

impl ForwardSimplePass {
    /// Register shaders required by this pass
    ///
    /// Should be called before creating any ForwardSimplePass instances
    pub fn register_shaders(graph: &mut RenderGraph) {
        use crate::render_graph::{ShaderDescriptor, ShaderStage};

        // Use pre-compiled SPIR-V shaders generated from unified HLSL source
        graph.register_shader(
            "forward_simple.vert",
            ShaderDescriptor::from_compiled("shaders/forward_simple.vert.spv", ShaderStage::Vertex)
                .with_entry_point("VSMain"),
        );
        graph.register_shader(
            "forward_simple.frag",
            ShaderDescriptor::from_compiled(
                "shaders/forward_simple.frag.spv",
                ShaderStage::Fragment,
            )
            .with_entry_point("PSMain"),
        );
    }

    /// Get the pass ID
    pub fn pass_id(&self) -> PassId {
        self.pass_id
    }

    /// Create a builder for the forward pass
    pub fn builder() -> ForwardSimplePassBuilder {
        ForwardSimplePassBuilder::new()
    }

    /// Prepare render graph resources from a scene for the forward pass.
    ///
    /// Returns a descriptor with all resource identifiers and metadata that the pass needs.
    pub fn prepare_scene_resources(
        scene: &Scene,
        graph: &mut RenderGraph,
        width: u32,
        height: u32,
    ) -> Result<ForwardSimpleSceneResources> {
        use crate::render_graph::BufferUsageFlags;

        // Expand inline geometry into vertex list (currently supports single mesh scenes).
        let mut base_vertices = Vec::new();
        let mut indices = Vec::new();
        let mut vertex_offset = 0u32;

        for obj in &scene.objects {
            match obj {
                SceneObject::Mesh { geometry, .. } => match geometry {
                    GeometryData::Inline {
                        vertices,
                        indices: mesh_indices,
                    } => {
                        // Push vertex data
                        base_vertices.extend_from_slice(vertices);

                        // Push indices with proper offsets (generate sequential if missing)
                        if let Some(idx) = mesh_indices {
                            indices.extend(idx.iter().map(|i| i + vertex_offset));
                        } else {
                            indices.extend((0..vertices.len() as u32).map(|i| i + vertex_offset));
                        }
                        vertex_offset += vertices.len() as u32;
                    }
                    GeometryData::File { .. } => {
                        log::warn!("ForwardSimplePass: external geometry files not yet supported");
                    }
                },
                SceneObject::GltfModel { .. } => {
                    log::warn!("ForwardSimplePass: glTF models not yet supported in render graph");
                }
            }
        }

        #[repr(C)]
        #[derive(Clone, Copy)]
        struct GpuVertex {
            position: [f32; 3],
            normal: [f32; 3],
            uv: [f32; 2],
            color: [f32; 4],
        }
        unsafe impl bytemuck::Pod for GpuVertex {}
        unsafe impl bytemuck::Zeroable for GpuVertex {}

        let gpu_vertices: Vec<GpuVertex> = base_vertices
            .iter()
            .map(|v| GpuVertex {
                position: v.position,
                normal: v.normal,
                uv: v.uv,
                color: [v.color[0], v.color[1], v.color[2], 1.0],
            })
            .collect();
        let vertex_buffer = graph.declare_buffer_with_data(
            "forward_simple_vertices",
            bytemuck::cast_slice(&gpu_vertices).to_vec(),
            BufferUsageFlags::new(BufferUsageFlags::VERTEX),
        );

        if indices.is_empty() {
            log::warn!("ForwardSimplePass: no indices found; generated sequential indices");
            indices.extend(0..gpu_vertices.len() as u32);
        }

        let index_buffer = graph.declare_buffer_with_data(
            "forward_simple_indices",
            bytemuck::cast_slice(&indices).to_vec(),
            BufferUsageFlags::new(BufferUsageFlags::INDEX),
        );

        // Camera uniforms
        let camera_ctrl = CameraController::from_scene_camera(&scene.camera, width, height);
        let camera_uniforms = camera_ctrl.uniforms();
        let camera_buffer = graph.declare_buffer_with_data(
            "forward_simple_camera",
            bytemuck::bytes_of(&camera_uniforms).to_vec(),
            BufferUsageFlags::new(BufferUsageFlags::UNIFORM),
        );

        // Lighting uniforms
        let scene_lighting = scene.lighting.as_ref().cloned().unwrap_or_default();
        let lighting = Lighting::new(&scene_lighting);
        let lighting_buffer = graph.declare_buffer_with_data(
            "forward_simple_lighting",
            lighting.buffer_data().to_vec(),
            BufferUsageFlags::new(BufferUsageFlags::UNIFORM),
        );

        // Current implementation uses transform of the first mesh if available.
        let transform = scene
            .objects
            .first()
            .and_then(|obj| match obj {
                SceneObject::Mesh { transform, .. } => Some(*transform),
                _ => None,
            })
            .unwrap_or_default();

        Ok(ForwardSimpleSceneResources {
            vertex_buffer,
            index_buffer,
            vertex_count: gpu_vertices.len() as u32,
            index_count: indices.len() as u32,
            camera_buffer,
            lighting_buffer,
            transform,
        })
    }
}

/// Prepared resources for the ForwardSimplePass created from a scene.
pub struct ForwardSimpleSceneResources {
    pub vertex_buffer: ResourceId,
    pub index_buffer: ResourceId,
    pub vertex_count: u32,
    pub index_count: u32,
    pub camera_buffer: ResourceId,
    pub lighting_buffer: ResourceId,
    pub transform: Transform,
}

/// Builder for ForwardSimplePass
pub struct ForwardSimplePassBuilder {
    color_output: Option<ResourceId>,
    depth_output: Option<ResourceId>,
    vertex_buffer: Option<ResourceId>,
    index_buffer: Option<ResourceId>,
    camera_buffer: Option<ResourceId>,
    lighting_buffer: Option<ResourceId>,
    material_buffer: Option<ResourceId>,
    albedo_texture: Option<ResourceId>,
    shadow_map: Option<ResourceId>,
    shadow_uniforms: Option<ResourceId>,
    transform: Transform,
    vertex_count: u32,
    index_count: u32,
    name: String,
}

impl ForwardSimplePassBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            color_output: None,
            depth_output: None,
            vertex_buffer: None,
            index_buffer: None,
            camera_buffer: None,
            lighting_buffer: None,
            material_buffer: None,
            albedo_texture: None,
            shadow_map: None,
            shadow_uniforms: None,
            transform: Transform::default(),
            vertex_count: 0,
            index_count: 0,
            name: "forward_simple".to_string(),
        }
    }

    /// Set color output attachment
    pub fn color_output(mut self, resource: ResourceId) -> Self {
        self.color_output = Some(resource);
        self
    }

    /// Set depth output attachment
    pub fn depth_output(mut self, resource: ResourceId) -> Self {
        self.depth_output = Some(resource);
        self
    }

    /// Set vertex buffer
    pub fn vertex_buffer(mut self, resource: ResourceId) -> Self {
        self.vertex_buffer = Some(resource);
        self
    }

    /// Set index buffer
    pub fn index_buffer(mut self, resource: ResourceId) -> Self {
        self.index_buffer = Some(resource);
        self
    }

    /// Set camera uniforms buffer
    pub fn camera_buffer(mut self, resource: ResourceId) -> Self {
        self.camera_buffer = Some(resource);
        self
    }

    /// Set lighting uniforms buffer
    pub fn lighting_buffer(mut self, resource: ResourceId) -> Self {
        self.lighting_buffer = Some(resource);
        self
    }

    /// Set optional material uniforms buffer
    pub fn material_buffer(mut self, resource: ResourceId) -> Self {
        self.material_buffer = Some(resource);
        self
    }

    /// Set optional albedo texture
    pub fn albedo_texture(mut self, resource: ResourceId) -> Self {
        self.albedo_texture = Some(resource);
        self
    }

    /// Set shadow map texture
    pub fn shadow_map(mut self, resource: ResourceId) -> Self {
        self.shadow_map = Some(resource);
        self
    }

    /// Set shadow uniforms buffer
    pub fn shadow_uniforms(mut self, resource: ResourceId) -> Self {
        self.shadow_uniforms = Some(resource);
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

    /// Set index count
    pub fn index_count(mut self, count: u32) -> Self {
        self.index_count = count;
        self
    }

    /// Set custom pass name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Build the pass and add it to the render graph
    ///
    /// # Note
    /// Call `ForwardSimplePass::register_shaders(graph)` before building instances
    pub fn build(self, graph: &mut RenderGraph) -> Result<ForwardSimplePass> {
        // Register shaders before building the pass
        ForwardSimplePass::register_shaders(graph);

        // Validate required resources
        let color_output = self
            .color_output
            .ok_or_else(|| anyhow::anyhow!("color_output is required"))?;
        let depth_output = self
            .depth_output
            .ok_or_else(|| anyhow::anyhow!("depth_output is required"))?;
        let vertex_buffer = self
            .vertex_buffer
            .ok_or_else(|| anyhow::anyhow!("vertex_buffer is required"))?;
        let index_buffer = self
            .index_buffer
            .ok_or_else(|| anyhow::anyhow!("index_buffer is required"))?;
        let camera_buffer = self
            .camera_buffer
            .ok_or_else(|| anyhow::anyhow!("camera_buffer is required"))?;
        let lighting_buffer = self
            .lighting_buffer
            .ok_or_else(|| anyhow::anyhow!("lighting_buffer is required"))?;

        if self.vertex_count == 0 {
            return Err(anyhow::anyhow!("vertex_count must be greater than 0"));
        }
        if self.index_count == 0 {
            return Err(anyhow::anyhow!("index_count must be greater than 0"));
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

        // Output: Depth attachment
        pass.add_output(ResourceAccess::new(
            depth_output,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT), // TODO: Should be EARLY_FRAGMENT_TESTS
            Some(ImageLayout::DepthStencilAttachment),
        ));

        // Input: Vertex buffer
        pass.add_input(ResourceAccess::new(
            vertex_buffer,
            AccessType::Read,
            PipelineStage::new(PipelineStage::VERTEX_INPUT),
            None,
        ));

        // Input: Index buffer
        pass.add_input(ResourceAccess::new(
            index_buffer,
            AccessType::Read,
            PipelineStage::new(PipelineStage::VERTEX_INPUT),
            None,
        ));

        // Input: Camera uniforms
        pass.add_input(ResourceAccess::new(
            camera_buffer,
            AccessType::Read,
            PipelineStage::new(PipelineStage::VERTEX_SHADER),
            None,
        ));

        // Input: Lighting uniforms
        pass.add_input(ResourceAccess::new(
            lighting_buffer,
            AccessType::Read,
            PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
            None,
        ));

        // Optional: Material uniforms
        if let Some(material_buffer) = self.material_buffer {
            pass.add_input(ResourceAccess::new(
                material_buffer,
                AccessType::Read,
                PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
                None,
            ));
        }

        // Optional: Albedo texture
        if let Some(albedo_texture) = self.albedo_texture {
            pass.add_input(ResourceAccess::new(
                albedo_texture,
                AccessType::Read,
                PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
                Some(ImageLayout::ShaderReadOnly),
            ));
        }

        // Optional: Shadow map
        let shadow_map = self.shadow_map;
        if let Some(shadow_map_id) = shadow_map {
            pass.add_input(ResourceAccess::new(
                shadow_map_id,
                AccessType::Read,
                PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
                Some(ImageLayout::ShaderReadOnly),
            ));
        }

        // Optional: Shadow uniforms
        let shadow_uniforms = self.shadow_uniforms;
        if let Some(shadow_uniforms_id) = shadow_uniforms {
            pass.add_input(ResourceAccess::new(
                shadow_uniforms_id,
                AccessType::Read,
                PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
                None,
            ));
        }

        // Set up execution callback
        let callback = ForwardSimplePassCallback {
            transform: self.transform,
            vertex_count: self.vertex_count,
            index_count: self.index_count,
            vertex_buffer,
            index_buffer,
            camera_buffer,
            lighting_buffer,
            material_buffer: self.material_buffer,
            albedo_texture: self.albedo_texture,
            shadow_map,
            shadow_uniforms,
        };

        pass = pass.with_callback(Box::new(callback));

        graph.add_pass(pass);

        Ok(ForwardSimplePass {
            pass_id,
            transform: self.transform,
            vertex_count: self.vertex_count,
        })
    }
}

impl Default for ForwardSimplePassBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution callback for the forward pass
#[allow(dead_code)] // Fields will be used when full execution is implemented
struct ForwardSimplePassCallback {
    transform: Transform,
    vertex_count: u32,
    index_count: u32,
    vertex_buffer: ResourceId,
    index_buffer: ResourceId,
    camera_buffer: ResourceId,
    lighting_buffer: ResourceId,
    material_buffer: Option<ResourceId>,
    albedo_texture: Option<ResourceId>,
    shadow_map: Option<ResourceId>,
    shadow_uniforms: Option<ResourceId>,
}

impl PassCallback for ForwardSimplePassCallback {
    fn declare_pipeline(&self, builder: &mut PipelineBuilder, registry: &ShaderRegistry) {
        use crate::render_graph::{
            CullMode, InputRate, VertexAttribute, VertexBinding, VertexFormat, VertexLayout,
        };

        // Get shaders from registry
        let vs_handle = registry
            .get_handle("forward_simple.vert")
            .expect("forward_simple.vert not found in shader registry");
        let fs_handle = registry
            .get_handle("forward_simple.frag")
            .expect("forward_simple.frag not found in shader registry");

        // Define vertex layout matching our Vertex struct:
        // position (3xf32 = 12 bytes) + normal (3xf32 = 12 bytes) + uv (2xf32 = 8 bytes) + color (4xf32 = 16 bytes) = 48 bytes
        let mut vertex_layout = VertexLayout::new();
        vertex_layout
            .add_attribute(VertexAttribute {
                location: 0,
                format: VertexFormat::Float32x3,
                offset: 0, // position
            })
            .add_attribute(VertexAttribute {
                location: 1,
                format: VertexFormat::Float32x3,
                offset: 12, // normal
            })
            .add_attribute(VertexAttribute {
                location: 2,
                format: VertexFormat::Float32x2,
                offset: 24, // uv
            })
            .add_attribute(VertexAttribute {
                location: 3,
                format: VertexFormat::Float32x4,
                offset: 32, // color
            })
            .add_binding(VertexBinding {
                binding: 0,
                stride: 48,
                input_rate: InputRate::Vertex,
            });

        builder
            .vertex_shader(vs_handle)
            .fragment_shader(fs_handle)
            .vertex_layout(vertex_layout)
            .depth_test(true)
            .depth_write(true)
            .cull_mode(CullMode::None); // Disable culling for debugging
    }

    fn prepare(&self, _context: &mut dyn PassPreparationContext) {
        // TODO: In the future, resource binding will be prepared here
        // For now, the backend handles this during execution
        log::trace!("Preparing forward simple pass");
    }

    fn execute(&self, context: &mut dyn PassExecutionContext) {
        log::info!(
            "Executing forward simple pass ({} vertices, {} indices)",
            self.vertex_count,
            self.index_count
        );

        // Get buffer pointers from resource IDs
        log::info!(
            "Getting vertex buffer ptr for resource {:?}",
            self.vertex_buffer
        );
        let vertex_buffer_ptr = context
            .get_buffer_ptr(self.vertex_buffer)
            .expect("Failed to get vertex buffer");
        log::info!(
            "Getting index buffer ptr for resource {:?}",
            self.index_buffer
        );
        let index_buffer_ptr = context
            .get_buffer_ptr(self.index_buffer)
            .expect("Failed to get index buffer");
        log::info!(
            "Getting camera buffer ptr for resource {:?}",
            self.camera_buffer
        );
        let camera_buffer_ptr = context
            .get_buffer_ptr(self.camera_buffer)
            .expect("Failed to get camera buffer");
        log::info!(
            "Getting lighting buffer ptr for resource {:?}",
            self.lighting_buffer
        );
        let lighting_buffer_ptr = context
            .get_buffer_ptr(self.lighting_buffer)
            .expect("Failed to get lighting buffer");

        // Bind vertex buffer
        log::info!("Binding vertex buffer");
        context
            .bind_vertex_buffer(0, vertex_buffer_ptr, 0)
            .expect("Failed to bind vertex buffer");
        context
            .bind_index_buffer(index_buffer_ptr, 0, IndexType::U32)
            .expect("Failed to bind index buffer");

        // Bind uniforms
        log::info!("Binding camera uniforms");
        context
            .bind_uniform_buffer(0, 0, camera_buffer_ptr, 0, 64) // CameraUniforms: viewProj (mat4) = 64 bytes
            .expect("Failed to bind camera uniforms");
        log::info!("Binding lighting uniforms");
        context
            .bind_uniform_buffer(0, 1, lighting_buffer_ptr, 0, 16 + 8 * 48) // LightingUniforms: ambient_light_count (16) + 8 lights (8*48 = 384) = 400 bytes
            .expect("Failed to bind lighting uniforms");

        // Bind shadow uniforms if present
        if let Some(shadow_uniforms_id) = self.shadow_uniforms {
            log::info!("Binding shadow uniforms for resource {:?}", shadow_uniforms_id);
            let shadow_buffer_ptr = context
                .get_buffer_ptr(shadow_uniforms_id)
                .expect("Failed to get shadow uniforms buffer");
            context
                .bind_uniform_buffer(0, 3, shadow_buffer_ptr, 0, 80) // lightSpaceMatrix (64) + shadowParams (16) = 80 bytes
                .expect("Failed to bind shadow uniforms");
            log::info!("Shadow uniforms bound successfully");
        }

        // Bind shadow map texture if present
        if let Some(shadow_map_id) = self.shadow_map {
            log::info!("Binding shadow map texture for resource {:?}", shadow_map_id);
            let shadow_texture_ptr = context
                .get_texture_ptr(shadow_map_id)
                .expect("Failed to get shadow map");
            log::info!("Got shadow texture ptr: {:?}", shadow_texture_ptr);
            context
                .bind_texture(0, 4, shadow_texture_ptr) // Binding 4 for combined image sampler
                .expect("Failed to bind shadow map texture");
            log::info!("Shadow map texture bound successfully");
        }

        // Push model matrix as push constants
        let model_matrix = self.transform.matrix();
        let normal_matrix = self.transform.normal_matrix();

        // Debug logging that works in both Windows and Linux
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("matrix_debug.log")
        {
            use std::io::Write;
            let _ = writeln!(f, "=== MATRIX DEBUG ===");
            let _ = writeln!(f, "Backend: {:?}", crate::camera::get_camera_backend());
            let _ = writeln!(f, "Model matrix:");
            let _ = writeln!(f, "  Row 0: {:?}", model_matrix[0]);
            let _ = writeln!(f, "  Row 1: {:?}", model_matrix[1]);
            let _ = writeln!(f, "  Row 2: {:?}", model_matrix[2]);
            let _ = writeln!(f, "  Row 3: {:?}", model_matrix[3]);
        }

        #[repr(C)]
        #[derive(Clone, Copy)]
        struct PushConstants {
            model: [[f32; 4]; 4],
            normal: [[f32; 4]; 4],
        }
        unsafe impl bytemuck::Pod for PushConstants {}
        unsafe impl bytemuck::Zeroable for PushConstants {}

        let push_data = PushConstants {
            model: model_matrix,
            normal: normal_matrix,
        };

        log::info!("Pushing constants");
        log::info!("  Model matrix row 0: {:?}", model_matrix[0]);
        log::info!("  Model matrix row 3: {:?}", model_matrix[3]);
        context
            .push_constants(
                0x1, // VERTEX stage only
                0,
                bytemuck::bytes_of(&push_data),
            )
            .expect("Failed to push constants");

        // Draw
        log::info!("Drawing indexed geometry ({} indices)", self.index_count);
        context
            .draw_indexed(self.index_count, 1, 0, 0, 0)
            .expect("Failed to draw indexed");

        log::info!("Forward simple pass execution complete");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_graph::{
        BufferUsageFlags, Extent3D, ExtentMode, Format, ImageUsageFlags, ResourceDescriptor,
        SampleCount,
    };

    #[test]
    fn test_forward_simple_pass_builder() {
        let mut graph = RenderGraph::new();

        // Create resources
        let color = graph.create_resource(
            "color",
            ResourceDescriptor::Image {
                format: Format::Bgra8Unorm,
                extent: ExtentMode::Absolute(Extent3D::new_2d(800, 600)),
                usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
                samples: SampleCount::One,
                mip_levels: 1,
            },
        );

        let depth = graph.create_resource(
            "depth",
            ResourceDescriptor::Image {
                format: Format::Depth32Float,
                extent: ExtentMode::Absolute(Extent3D::new_2d(800, 600)),
                usage: ImageUsageFlags::new(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
                samples: SampleCount::One,
                mip_levels: 1,
            },
        );

        let vertex_buffer = graph.create_resource(
            "vertices",
            ResourceDescriptor::Buffer {
                size: 1024,
                usage: BufferUsageFlags::new(BufferUsageFlags::VERTEX),
            },
        );

        let camera = graph.create_resource(
            "camera",
            ResourceDescriptor::Buffer {
                size: 128,
                usage: BufferUsageFlags::new(BufferUsageFlags::UNIFORM),
            },
        );

        let lighting = graph.create_resource(
            "lighting",
            ResourceDescriptor::Buffer {
                size: 256,
                usage: BufferUsageFlags::new(BufferUsageFlags::UNIFORM),
            },
        );

        // Build pass
        let _pass = ForwardSimplePass::builder()
            .color_output(color)
            .depth_output(depth)
            .vertex_buffer(vertex_buffer)
            .camera_buffer(camera)
            .lighting_buffer(lighting)
            .vertex_count(36)
            .build(&mut graph)
            .unwrap();

        // NOTE: We can't test graph.compile() here because buffers need producers
        // (either a transfer pass or external import), which requires backend integration.
        // For now, we just verify the pass was created successfully.

        // Verify pass count
        assert_eq!(graph.passes().len(), 1);
    }

    #[test]
    fn test_forward_simple_pass_missing_resources() {
        let mut graph = RenderGraph::new();

        // Try to build without required resources - should fail
        let result = ForwardSimplePass::builder()
            .vertex_count(36)
            .build(&mut graph);

        assert!(result.is_err());
    }
}
