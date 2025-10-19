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
pub trait PassCallback: Send + Sync {
    /// Execute the pass
    ///
    /// # Arguments
    /// * `context` - Execution context with access to backend and resources
    fn execute(&self, context: &mut dyn PassExecutionContext);
}

/// Execution context provided to pass callbacks
pub trait PassExecutionContext {
    /// Get the backend type as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;

    /// Get the backend type as Any for mutable downcasting
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
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
