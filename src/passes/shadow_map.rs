//! Shadow map generation pass
//!
//! Renders the scene from the light's perspective to generate a depth map
//! that can be used for shadow calculations in the forward pass.

use crate::render_graph::{
    AccessType, BufferUsageFlags, Format, ImageLayout, ImageUsageFlags, PassCallback,
    PassExecutionContext, PassId, PassKind, PipelineBuilder, PipelineStage, RenderGraph,
    RenderPass, ResourceAccess, ResourceDescriptor, ResourceId, ShaderDescriptor, ShaderRegistry,
    ShaderStage,
};
use anyhow::Result;
use glam::{Mat4, Vec3};

/// Shadow map pass - renders scene depth from light's perspective
pub struct ShadowMapPass {
    pass_id: PassId,
}

impl ShadowMapPass {
    /// Register shaders required by shadow mapping
    pub fn register_shaders(graph: &mut RenderGraph) {
        graph.register_shader(
            "shadow_map.vert",
            ShaderDescriptor::from_compiled("shaders/shadow_map.vert.spv", ShaderStage::Vertex)
                .with_entry_point("VSMain"),
        );
        graph.register_shader(
            "shadow_map.frag",
            ShaderDescriptor::from_compiled("shaders/shadow_map.frag.spv", ShaderStage::Fragment)
                .with_entry_point("PSMain"),
        );
    }

    /// Create a builder for the shadow map pass
    pub fn builder() -> ShadowMapPassBuilder {
        ShadowMapPassBuilder::new()
    }

    /// Prepare scene resources for shadow mapping
    ///
    /// This creates the shadow map depth texture and light uniform buffer
    pub fn prepare_resources(
        graph: &mut RenderGraph,
        light_direction: Vec3,
        resolution: u32,
    ) -> ShadowMapResources {
        use crate::render_graph::{Extent3D, ExtentMode};

        // Calculate light view-projection matrix
        let light_position = -light_direction.normalize() * 10.0;
        let light_view = Mat4::look_at_rh(light_position, Vec3::ZERO, Vec3::Y);
        let light_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 20.0);
        let light_view_proj = light_proj * light_view;

        // Create shadow map depth texture
        let shadow_map = graph.create_resource(
            "shadow_map",
            ResourceDescriptor::Image {
                format: Format::Depth32Float,
                extent: ExtentMode::Absolute(Extent3D::new_2d(resolution, resolution)),
                usage: ImageUsageFlags::new(
                    ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ImageUsageFlags::SAMPLED,
                ),
                mip_levels: 1,
                samples: crate::render_graph::SampleCount::One,
            },
        );

        // Create light uniform buffer
        let light_matrix_data: [f32; 16] = light_view_proj.to_cols_array();
        let light_uniforms = graph.declare_buffer_with_data(
            "shadow_light_uniforms",
            bytemuck::cast_slice(&light_matrix_data).to_vec(),
            BufferUsageFlags::new(BufferUsageFlags::UNIFORM),
        );

        ShadowMapResources {
            shadow_map,
            light_uniforms,
            light_view_proj,
        }
    }
}

/// Resources created for shadow mapping
pub struct ShadowMapResources {
    pub shadow_map: ResourceId,
    pub light_uniforms: ResourceId,
    pub light_view_proj: Mat4,
}

/// Builder for shadow map pass
pub struct ShadowMapPassBuilder {
    shadow_map_output: Option<ResourceId>,
    vertex_buffer: Option<ResourceId>,
    index_buffer: Option<ResourceId>,
    light_uniforms: Option<ResourceId>,
    index_count: u32,
    name: String,
}

impl ShadowMapPassBuilder {
    pub fn new() -> Self {
        Self {
            shadow_map_output: None,
            vertex_buffer: None,
            index_buffer: None,
            light_uniforms: None,
            index_count: 0,
            name: "shadow_map".to_string(),
        }
    }

    pub fn shadow_map_output(mut self, resource: ResourceId) -> Self {
        self.shadow_map_output = Some(resource);
        self
    }

    pub fn vertex_buffer(mut self, resource: ResourceId) -> Self {
        self.vertex_buffer = Some(resource);
        self
    }

    pub fn index_buffer(mut self, resource: ResourceId) -> Self {
        self.index_buffer = Some(resource);
        self
    }

    pub fn light_uniforms(mut self, resource: ResourceId) -> Self {
        self.light_uniforms = Some(resource);
        self
    }

    pub fn index_count(mut self, count: u32) -> Self {
        self.index_count = count;
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Build the pass and add it to the render graph
    pub fn build(self, graph: &mut RenderGraph) -> Result<ShadowMapPass> {
        // Register shaders
        ShadowMapPass::register_shaders(graph);

        // Validate required resources
        let shadow_map_output = self
            .shadow_map_output
            .ok_or_else(|| anyhow::anyhow!("shadow_map_output is required"))?;
        let vertex_buffer = self
            .vertex_buffer
            .ok_or_else(|| anyhow::anyhow!("vertex_buffer is required"))?;
        let index_buffer = self
            .index_buffer
            .ok_or_else(|| anyhow::anyhow!("index_buffer is required"))?;
        let light_uniforms = self
            .light_uniforms
            .ok_or_else(|| anyhow::anyhow!("light_uniforms is required"))?;

        if self.index_count == 0 {
            return Err(anyhow::anyhow!("index_count must be greater than 0"));
        }

        let pass_id = graph.next_pass_id();
        let mut pass = RenderPass::new(pass_id, &self.name, PassKind::Graphics);

        // Output: Depth attachment
        pass.add_output(ResourceAccess::new(
            shadow_map_output,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
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

        // Input: Light uniforms
        pass.add_input(ResourceAccess::new(
            light_uniforms,
            AccessType::Read,
            PipelineStage::new(PipelineStage::VERTEX_SHADER),
            None,
        ));

        // Set up execution callback
        let callback = ShadowMapPassCallback {
            index_count: self.index_count,
            vertex_buffer,
            index_buffer,
            light_uniforms,
        };

        pass = pass.with_callback(Box::new(callback));

        graph.add_pass(pass);

        Ok(ShadowMapPass { pass_id })
    }
}

impl Default for ShadowMapPassBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution callback for shadow map pass
struct ShadowMapPassCallback {
    index_count: u32,
    vertex_buffer: ResourceId,
    index_buffer: ResourceId,
    light_uniforms: ResourceId,
}

impl PassCallback for ShadowMapPassCallback {
    fn declare_pipeline(&self, builder: &mut PipelineBuilder, registry: &ShaderRegistry) {
        use crate::render_graph::{
            CullMode, InputRate, VertexAttribute, VertexBinding, VertexFormat, VertexLayout,
        };

        // Get shaders from registry
        let vs_handle = registry
            .get_handle("shadow_map.vert")
            .expect("shadow_map.vert not found in shader registry");
        let fs_handle = registry
            .get_handle("shadow_map.frag")
            .expect("shadow_map.frag not found in shader registry");

        // Define vertex layout matching our Vertex struct
        let mut vertex_layout = VertexLayout::new();
        vertex_layout
            .add_attribute(VertexAttribute {
                location: 0,
                format: VertexFormat::Float32x3,
                offset: 0,
            })
            .add_attribute(VertexAttribute {
                location: 1,
                format: VertexFormat::Float32x3,
                offset: 12,
            })
            .add_attribute(VertexAttribute {
                location: 2,
                format: VertexFormat::Float32x2,
                offset: 24,
            })
            .add_attribute(VertexAttribute {
                location: 3,
                format: VertexFormat::Float32x4,
                offset: 32,
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
            .cull_mode(CullMode::Back);
    }

    fn execute(&self, context: &mut dyn PassExecutionContext) {
        use crate::render_graph::IndexType;

        // Get buffer pointers from resource IDs
        let vertex_buffer_ptr = context
            .get_buffer_ptr(self.vertex_buffer)
            .expect("Failed to get vertex buffer");
        let index_buffer_ptr = context
            .get_buffer_ptr(self.index_buffer)
            .expect("Failed to get index buffer");
        let light_buffer_ptr = context
            .get_buffer_ptr(self.light_uniforms)
            .expect("Failed to get light uniforms buffer");

        // Bind vertex buffer
        context
            .bind_vertex_buffer(0, vertex_buffer_ptr, 0)
            .expect("Failed to bind vertex buffer");

        // Bind index buffer
        context
            .bind_index_buffer(index_buffer_ptr, 0, IndexType::U32)
            .expect("Failed to bind index buffer");

        // Bind light uniforms
        context
            .bind_uniform_buffer(0, 0, light_buffer_ptr, 0, 64)
            .expect("Failed to bind light uniforms");

        // Draw indexed
        context
            .draw_indexed(self.index_count, 1, 0, 0, 0)
            .expect("Failed to draw indexed");
    }
}
