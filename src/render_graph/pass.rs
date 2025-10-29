//! Render graph pass types
//!
//! This module defines render passes and their execution callbacks.

use crate::render_graph::resource::ResourceId;
use std::fmt;

/// Unique identifier for a render pass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PassId(pub usize);

impl fmt::Display for PassId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Pass({})", self.0)
    }
}

/// Type of render pass
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassKind {
    /// Graphics pass (vertex/fragment shaders)
    Graphics,
    /// Compute pass (compute shaders)
    Compute,
    /// Transfer/copy pass
    Transfer,
}

/// Pipeline stage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineStage {
    bits: u32,
}

impl PipelineStage {
    pub const TOP_OF_PIPE: u32 = 1 << 0;
    pub const VERTEX_INPUT: u32 = 1 << 1;
    pub const VERTEX_SHADER: u32 = 1 << 2;
    pub const FRAGMENT_SHADER: u32 = 1 << 3;
    pub const COLOR_ATTACHMENT_OUTPUT: u32 = 1 << 4;
    pub const COMPUTE_SHADER: u32 = 1 << 5;
    pub const TRANSFER: u32 = 1 << 6;
    pub const BOTTOM_OF_PIPE: u32 = 1 << 7;

    pub fn new(bits: u32) -> Self {
        Self { bits }
    }

    pub fn contains(&self, flag: u32) -> bool {
        (self.bits & flag) != 0
    }
}

/// Resource access type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    /// Read-only access
    Read,
    /// Write-only access
    Write,
    /// Read-write access
    ReadWrite,
}

/// Image layout (primarily for Vulkan)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageLayout {
    Undefined,
    General,
    ColorAttachment,
    DepthStencilAttachment,
    ShaderReadOnly,
    TransferSrc,
    TransferDst,
    Present,
}

/// How a pass accesses a resource
#[derive(Debug, Clone)]
pub struct ResourceAccess {
    /// Resource being accessed
    pub resource: ResourceId,
    /// Type of access
    pub access_type: AccessType,
    /// Pipeline stage where access occurs
    pub stage: PipelineStage,
    /// Image layout (if resource is an image)
    pub layout: Option<ImageLayout>,
}

impl ResourceAccess {
    /// Create a new resource access
    pub fn new(
        resource: ResourceId,
        access_type: AccessType,
        stage: PipelineStage,
        layout: Option<ImageLayout>,
    ) -> Self {
        Self {
            resource,
            access_type,
            stage,
            layout,
        }
    }

    /// Create a read-only access
    pub fn read(resource: ResourceId, stage: PipelineStage) -> Self {
        Self::new(resource, AccessType::Read, stage, None)
    }

    /// Create a write-only access
    pub fn write(resource: ResourceId, stage: PipelineStage) -> Self {
        Self::new(resource, AccessType::Write, stage, None)
    }

    /// Create a read-write access
    pub fn read_write(resource: ResourceId, stage: PipelineStage) -> Self {
        Self::new(resource, AccessType::ReadWrite, stage, None)
    }
}

/// Pass execution callback
///
/// This trait is implemented by passes to define their execution behavior.
/// The callback receives a command buffer-like interface and resource map.
///
/// Execution happens in two phases:
/// 1. `prepare()` - Called before render pass begins, for resource setup (bind groups, etc.)
/// 2. `execute()` - Called during render pass, for recording draw commands
pub trait PassCallback: Send + Sync {
    /// Prepare resources before pass execution (optional)
    ///
    /// This method is called before the render pass begins, allowing backends
    /// to create resources that must exist outside the render pass (e.g., wgpu bind groups).
    ///
    /// Default implementation does nothing (for backends that don't need preparation).
    ///
    /// # Arguments
    /// * `context` - Preparation context for resource setup
    fn prepare(&self, _context: &mut dyn PassPreparationContext) {
        // Default: no preparation needed
    }

    /// Execute the pass
    ///
    /// # Arguments
    /// * `context` - Execution context with access to backend and resources
    fn execute(&self, context: &mut dyn PassExecutionContext);
}

/// Declarative render pass trait (new API)
///
/// This trait provides a cleaner, more declarative way to define render passes.
/// Unlike the low-level `PassCallback`, this trait allows passes to declare
/// their resource dependencies and requirements upfront.
///
/// # Example
/// ```ignore
/// struct MyPass {
///     color_output: ResourceId,
///     depth_input: ResourceId,
/// }
///
/// impl DeclarativePass for MyPass {
///     fn name(&self) -> &str {
///         "my_pass"
///     }
///
///     fn kind(&self) -> PassKind {
///         PassKind::Graphics
///     }
///
///     fn declare_resources(&self, graph: &mut RenderGraph) {
///         // Optionally create resources here
///     }
///
///     fn declare_dependencies(&self, builder: &mut PassBuilder) {
///         builder
///             .read(self.depth_input, PipelineStage::new(PipelineStage::FRAGMENT_SHADER))
///             .with_layout(ImageLayout::DepthStencilAttachment)
///             .write(self.color_output, PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT))
///             .with_layout(ImageLayout::ColorAttachment);
///     }
///
///     fn execute(&self, ctx: &mut dyn PassExecutionContext) {
///         // Record rendering commands
///     }
/// }
/// ```
pub trait DeclarativePass: Send + Sync {
    /// Get the pass name
    fn name(&self) -> &str;

    /// Get the pass kind (Graphics, Compute, or Transfer)
    fn kind(&self) -> PassKind {
        PassKind::Graphics
    }

    /// Declare resources that this pass needs
    ///
    /// This optional method allows passes to create resources in the graph.
    /// The default implementation does nothing.
    ///
    /// # Arguments
    /// * `graph` - The render graph to add resources to
    fn declare_resources(&self, _graph: &mut crate::render_graph::RenderGraph) {
        // Default: no resources to declare
    }

    /// Declare this pass's resource dependencies
    ///
    /// Use the builder to declare which resources this pass reads from
    /// and writes to, along with pipeline stages and layouts.
    ///
    /// # Arguments
    /// * `builder` - PassBuilder for declaring dependencies
    fn declare_dependencies(&self, builder: &mut PassBuilder);

    /// Declare pipeline configuration (shaders and state)
    ///
    /// This optional method allows graphics and compute passes to specify
    /// their pipeline requirements. The default implementation does nothing,
    /// which is appropriate for transfer-only passes.
    ///
    /// The render graph will use this declaration to automatically create
    /// and cache the pipeline object for this pass.
    ///
    /// # Arguments
    /// * `builder` - PipelineBuilder for configuring shaders and state
    /// * `registry` - Shader registry for loading shaders
    ///
    /// # Example
    /// ```ignore
    /// fn declare_pipeline(&self, builder: &mut PipelineBuilder, registry: &ShaderRegistry) {
    ///     let vs = registry.get_shader("forward.vert").unwrap();
    ///     let fs = registry.get_shader("forward.frag").unwrap();
    ///     
    ///     builder
    ///         .vertex_shader(vs)
    ///         .fragment_shader(fs)
    ///         .depth_test(true)
    ///         .depth_write(true)
    ///         .cull_mode(CullMode::Back);
    /// }
    /// ```
    fn declare_pipeline(
        &self,
        _builder: &mut crate::render_graph::PipelineBuilder,
        _registry: &crate::render_graph::ShaderRegistry,
    ) {
        // Default: no pipeline (compute/transfer passes or passes that manage their own pipeline)
    }

    /// Prepare resources before pass execution (optional)
    ///
    /// This method is called before the render pass begins.
    /// The default implementation does nothing.
    ///
    /// # Arguments
    /// * `context` - Preparation context for resource setup
    fn prepare(&self, _context: &mut dyn PassPreparationContext) {
        // Default: no preparation needed
    }

    /// Execute the pass
    ///
    /// # Arguments
    /// * `context` - Execution context with access to backend and resources
    fn execute(&self, context: &mut dyn PassExecutionContext);
}

/// Adapter to bridge DeclarativePass to PassCallback
///
/// This allows DeclarativePass implementations to work with the existing
/// render graph execution system.
pub(crate) struct DeclarativePassAdapter<T: DeclarativePass> {
    pass: T,
}

impl<T: DeclarativePass> DeclarativePassAdapter<T> {
    pub(crate) fn new(pass: T) -> Self {
        Self { pass }
    }
}

impl<T: DeclarativePass> PassCallback for DeclarativePassAdapter<T> {
    fn prepare(&self, context: &mut dyn PassPreparationContext) {
        self.pass.prepare(context);
    }

    fn execute(&self, context: &mut dyn PassExecutionContext) {
        self.pass.execute(context);
    }
}

/// Preparation context for resource setup before pass execution
///
/// This trait allows passes to set up resources that need to exist before
/// the render pass begins (e.g., bind groups in wgpu).
///
/// Backends that don't need preparation can provide a no-op implementation.
pub trait PassPreparationContext {
    /// Get the backend type as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Get the backend type as Any for mutable downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Prepare a uniform buffer binding
    ///
    /// Allows backends to create bind groups or descriptor sets before the pass begins.
    ///
    /// # Arguments
    /// * `set` - Descriptor set index
    /// * `binding` - Binding index within the set
    /// * `buffer_ptr` - Pointer to the buffer implementation
    /// * `offset` - Offset in bytes into the buffer
    /// * `size` - Size in bytes of the uniform data
    fn prepare_uniform_buffer(
        &mut self,
        set: u32,
        binding: u32,
        buffer_ptr: *const std::ffi::c_void,
        offset: u64,
        size: u64,
    ) -> anyhow::Result<()>;

    /// Prepare a texture binding
    ///
    /// # Arguments
    /// * `set` - Descriptor set index
    /// * `binding` - Binding index within the set
    /// * `texture_ptr` - Pointer to the texture implementation
    fn prepare_texture(
        &mut self,
        set: u32,
        binding: u32,
        texture_ptr: *const std::ffi::c_void,
    ) -> anyhow::Result<()>;

    /// Prepare push constants
    ///
    /// # Arguments
    /// * `stage_flags` - Shader stages that will access the constants
    /// * `offset` - Offset in bytes into the push constant block
    /// * `size` - Size of the push constant data
    fn prepare_push_constants(
        &mut self,
        stage_flags: u32,
        offset: u32,
        size: u32,
    ) -> anyhow::Result<()>;
}

/// Execution context provided to pass callbacks
///
/// This trait provides access to rendering commands during pass execution.
/// Backends implement this to expose their command recording capabilities.
pub trait PassExecutionContext {
    /// Get the backend type as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Get the backend type as Any for mutable downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    /// Bind a vertex buffer for rendering (M8.2)
    ///
    /// # Arguments
    /// * `binding` - Binding slot (typically 0 for the first vertex buffer)
    /// * `buffer_id` - ID of the buffer resource (for resource tracking)
    /// * `buffer_ptr` - Pointer to the buffer implementation
    /// * `offset` - Offset in bytes into the buffer
    fn bind_vertex_buffer(
        &mut self,
        binding: u32,
        buffer_ptr: *const std::ffi::c_void,
        offset: u64,
    ) -> anyhow::Result<()>;

    /// Bind an index buffer for indexed rendering (M8.2)
    ///
    /// # Arguments
    /// * `buffer_ptr` - Pointer to the buffer implementation
    /// * `offset` - Offset in bytes into the buffer
    /// * `index_type` - The type of indices (U16 or U32)
    fn bind_index_buffer(
        &mut self,
        buffer_ptr: *const std::ffi::c_void,
        offset: u64,
        index_type: IndexType,
    ) -> anyhow::Result<()>;

    /// Draw primitives using vertex data (M8.2)
    ///
    /// # Arguments
    /// * `vertex_count` - Number of vertices to draw
    /// * `instance_count` - Number of instances to draw
    /// * `first_vertex` - Index of the first vertex
    /// * `first_instance` - Index of the first instance
    fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> anyhow::Result<()>;

    /// Draw indexed primitives (M8.2)
    ///
    /// # Arguments
    /// * `index_count` - Number of indices to draw
    /// * `instance_count` - Number of instances to draw
    /// * `first_index` - Index of the first index in the index buffer
    /// * `vertex_offset` - Offset added to vertex indices
    /// * `first_instance` - Index of the first instance
    fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) -> anyhow::Result<()>;

    /// Bind a uniform buffer (M8.3)
    ///
    /// Binds a uniform buffer to the specified descriptor set and binding.
    /// The buffer must have been created with BufferUsage::UNIFORM.
    ///
    /// # Arguments
    /// * `set` - Descriptor set index (typically 0 for global uniforms)
    /// * `binding` - Binding index within the set
    /// * `buffer_ptr` - Pointer to the buffer implementation
    /// * `offset` - Offset in bytes into the buffer
    /// * `size` - Size in bytes of the uniform data
    fn bind_uniform_buffer(
        &mut self,
        set: u32,
        binding: u32,
        buffer_ptr: *const std::ffi::c_void,
        offset: u64,
        size: u64,
    ) -> anyhow::Result<()>;

    /// Push constants to the shader (for per-draw data like model matrices)
    ///
    /// # Arguments
    /// * `stage_flags` - Shader stages that will access the constants
    /// * `offset` - Offset in bytes into the push constant block
    /// * `data` - Raw bytes to push
    fn push_constants(
        &mut self,
        stage_flags: u32, // Shader stage flags
        offset: u32,
        data: &[u8],
    ) -> anyhow::Result<()>;

    /// Bind a texture for sampling in shaders (M10 Phase 4)
    ///
    /// # Arguments
    /// * `set` - Descriptor set index
    /// * `binding` - Binding index within the set
    /// * `texture_ptr` - Pointer to the texture implementation
    fn bind_texture(
        &mut self,
        set: u32,
        binding: u32,
        texture_ptr: *const std::ffi::c_void,
    ) -> anyhow::Result<()>;
}

/// Index buffer data type (re-exported for pass callbacks)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexType {
    /// 16-bit unsigned integer indices
    U16,
    /// 32-bit unsigned integer indices
    U32,
}

/// A render pass in the render graph
pub struct RenderPass {
    /// Unique identifier
    pub id: PassId,
    /// Pass name for debugging
    pub name: String,
    /// Pass type
    pub kind: PassKind,
    /// Input resources (read)
    pub inputs: Vec<ResourceAccess>,
    /// Output resources (write)
    pub outputs: Vec<ResourceAccess>,
    /// Execution callback
    pub callback: Option<Box<dyn PassCallback>>,
}

/// Builder for declaratively configuring pass dependencies
///
/// PassBuilder provides a clean API for passes to declare their resource
/// dependencies without manually managing ResourceAccess objects.
///
/// # Example
/// ```ignore
/// fn declare_dependencies(&self, builder: &mut PassBuilder) {
///     builder
///         .read(self.depth_buffer, PipelineStage::FRAGMENT_SHADER)
///         .write(self.color_buffer, PipelineStage::COLOR_ATTACHMENT_OUTPUT)
///         .with_layout(ImageLayout::ColorAttachment);
/// }
/// ```
pub struct PassBuilder {
    #[allow(dead_code)]
    pass_id: PassId,
    inputs: Vec<ResourceAccess>,
    outputs: Vec<ResourceAccess>,
}

impl PassBuilder {
    /// Create a new pass builder
    pub fn new(pass_id: PassId) -> Self {
        Self {
            pass_id,
            inputs: Vec::new(),
            outputs: Vec::new(),
        }
    }

    /// Declare a read dependency on a resource
    ///
    /// # Arguments
    /// * `resource` - Resource to read from
    /// * `stage` - Pipeline stage where the read occurs
    pub fn read(&mut self, resource: ResourceId, stage: PipelineStage) -> &mut Self {
        self.inputs.push(ResourceAccess::read(resource, stage));
        self
    }

    /// Declare a write dependency on a resource
    ///
    /// # Arguments
    /// * `resource` - Resource to write to
    /// * `stage` - Pipeline stage where the write occurs
    pub fn write(&mut self, resource: ResourceId, stage: PipelineStage) -> &mut Self {
        self.outputs.push(ResourceAccess::write(resource, stage));
        self
    }

    /// Declare a read-write dependency on a resource
    ///
    /// # Arguments
    /// * `resource` - Resource to read from and write to
    /// * `stage` - Pipeline stage where the access occurs
    pub fn read_write(&mut self, resource: ResourceId, stage: PipelineStage) -> &mut Self {
        self.outputs
            .push(ResourceAccess::read_write(resource, stage));
        self
    }

    /// Set the image layout for the last added dependency
    ///
    /// This is a convenience method that modifies the most recently added
    /// input or output dependency.
    pub fn with_layout(&mut self, layout: ImageLayout) -> &mut Self {
        if let Some(last) = self.outputs.last_mut().or_else(|| self.inputs.last_mut()) {
            last.layout = Some(layout);
        }
        self
    }

    /// Take ownership of inputs and outputs (internal use)
    pub(crate) fn build(self) -> (Vec<ResourceAccess>, Vec<ResourceAccess>) {
        (self.inputs, self.outputs)
    }
}

impl RenderPass {
    /// Create a new render pass
    pub fn new(id: PassId, name: impl Into<String>, kind: PassKind) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            inputs: Vec::new(),
            outputs: Vec::new(),
            callback: None,
        }
    }

    /// Set the execution callback
    pub fn with_callback(mut self, callback: Box<dyn PassCallback>) -> Self {
        self.callback = Some(callback);
        self
    }

    /// Add an input resource
    pub fn add_input(&mut self, access: ResourceAccess) {
        self.inputs.push(access);
    }

    /// Add an output resource
    pub fn add_output(&mut self, access: ResourceAccess) {
        self.outputs.push(access);
    }

    /// Get all resources accessed by this pass
    pub fn all_resources(&self) -> impl Iterator<Item = ResourceId> + '_ {
        self.inputs
            .iter()
            .chain(self.outputs.iter())
            .map(|access| access.resource)
    }

    /// Check if this pass reads from a resource
    pub fn reads_resource(&self, resource: ResourceId) -> bool {
        self.inputs.iter().any(|access| access.resource == resource)
    }

    /// Check if this pass writes to a resource
    pub fn writes_resource(&self, resource: ResourceId) -> bool {
        self.outputs
            .iter()
            .any(|access| access.resource == resource)
    }
}

impl fmt::Debug for RenderPass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderPass")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("kind", &self.kind)
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .field("callback", &self.callback.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pass_creation() {
        let pass = RenderPass::new(PassId(0), "test_pass", PassKind::Graphics);
        assert_eq!(pass.id, PassId(0));
        assert_eq!(pass.name, "test_pass");
        assert_eq!(pass.kind, PassKind::Graphics);
        assert!(pass.inputs.is_empty());
        assert!(pass.outputs.is_empty());
    }

    #[test]
    fn test_resource_access() {
        let mut pass = RenderPass::new(PassId(0), "test_pass", PassKind::Graphics);

        let res1 = ResourceId(1);
        let res2 = ResourceId(2);

        pass.add_input(ResourceAccess::read(
            res1,
            PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
        ));
        pass.add_output(ResourceAccess::write(
            res2,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
        ));

        assert!(pass.reads_resource(res1));
        assert!(!pass.writes_resource(res1));
        assert!(!pass.reads_resource(res2));
        assert!(pass.writes_resource(res2));

        let all_resources: Vec<_> = pass.all_resources().collect();
        assert_eq!(all_resources.len(), 2);
        assert!(all_resources.contains(&res1));
        assert!(all_resources.contains(&res2));
    }

    #[test]
    fn test_access_type() {
        let res = ResourceId(0);
        let stage = PipelineStage::new(PipelineStage::FRAGMENT_SHADER);

        let read = ResourceAccess::read(res, stage);
        assert_eq!(read.access_type, AccessType::Read);

        let write = ResourceAccess::write(res, stage);
        assert_eq!(write.access_type, AccessType::Write);

        let rw = ResourceAccess::read_write(res, stage);
        assert_eq!(rw.access_type, AccessType::ReadWrite);
    }
}
