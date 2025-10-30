//! Simplified forward rendering pass with full render graph integration
//!
//! This pass demonstrates the target architecture where ALL resources are
//! managed by the render graph. No buffers or textures are passed directly.

use crate::render_graph::{
    AccessType, ImageLayout, PassCallback, PassExecutionContext, PassId, PassKind,
    PassPreparationContext, PipelineBuilder, PipelineStage, RenderGraph, RenderPass,
    ResourceAccess, ResourceId, ShaderRegistry,
};
use crate::scene::Transform;
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
    /// Get the pass ID
    pub fn pass_id(&self) -> PassId {
        self.pass_id
    }

    /// Create a builder for the forward pass
    pub fn builder() -> ForwardSimplePassBuilder {
        ForwardSimplePassBuilder::new()
    }
}

/// Builder for ForwardSimplePass
pub struct ForwardSimplePassBuilder {
    color_output: Option<ResourceId>,
    depth_output: Option<ResourceId>,
    vertex_buffer: Option<ResourceId>,
    camera_buffer: Option<ResourceId>,
    lighting_buffer: Option<ResourceId>,
    material_buffer: Option<ResourceId>,
    albedo_texture: Option<ResourceId>,
    transform: Transform,
    vertex_count: u32,
    name: String,
}

impl ForwardSimplePassBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            color_output: None,
            depth_output: None,
            vertex_buffer: None,
            camera_buffer: None,
            lighting_buffer: None,
            material_buffer: None,
            albedo_texture: None,
            transform: Transform::default(),
            vertex_count: 0,
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
    pub fn build(self, graph: &mut RenderGraph) -> Result<ForwardSimplePass> {
        use crate::render_graph::{ShaderDescriptor, ShaderStage};

        // Register shaders
        graph.register_shader(
            "forward.vert",
            ShaderDescriptor::from_file("shaders/hlsl/forward.hlsl", ShaderStage::Vertex)
                .with_entry_point("VSMain"),
        );
        graph.register_shader(
            "forward.frag",
            ShaderDescriptor::from_file("shaders/hlsl/forward.hlsl", ShaderStage::Fragment)
                .with_entry_point("PSMain"),
        );

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

        // Set up execution callback
        let callback = ForwardSimplePassCallback {
            transform: self.transform,
            vertex_count: self.vertex_count,
            vertex_buffer,
            camera_buffer,
            lighting_buffer,
            material_buffer: self.material_buffer,
            albedo_texture: self.albedo_texture,
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
    vertex_buffer: ResourceId,
    camera_buffer: ResourceId,
    lighting_buffer: ResourceId,
    material_buffer: Option<ResourceId>,
    albedo_texture: Option<ResourceId>,
}

impl PassCallback for ForwardSimplePassCallback {
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
            .cull_mode(CullMode::Back);
    }

    fn prepare(&self, _context: &mut dyn PassPreparationContext) {
        // TODO: In the future, resource binding will be prepared here
        // For now, the backend handles this during execution
        log::trace!("Preparing forward simple pass");
    }

    fn execute(&self, _context: &mut dyn PassExecutionContext) {
        // TODO: Full execution implementation
        // For now, the backend's execute_graph handles the actual rendering
        // Once we have proper context methods, this will:
        // 1. Bind pipeline
        // 2. Push model+normal matrices as push constants
        // 3. Bind descriptor sets (camera, lighting, material, texture)
        // 4. Bind vertex buffer
        // 5. Draw vertices

        log::info!(
            "Executing forward simple pass ({} vertices)",
            self.vertex_count
        );
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
