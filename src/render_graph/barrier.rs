//! Barrier insertion and synchronization
//!
//! This module implements automatic barrier insertion for proper synchronization
//! between render passes, including pipeline barriers, layout transitions, and
//! memory dependencies.

use crate::render_graph::pass::{AccessType, ImageLayout, PassId, PipelineStage, ResourceAccess};
use crate::render_graph::resource::{ResourceId, ResourceKind};
use std::collections::HashMap;

/// Type of barrier
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarrierType {
    /// Pipeline barrier (execution dependency)
    Pipeline,
    /// Memory barrier (memory dependency)
    Memory,
    /// Image layout transition
    ImageTransition,
}

/// Memory access flags for barriers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryAccess {
    bits: u32,
}

impl MemoryAccess {
    pub const NONE: u32 = 0;
    pub const READ: u32 = 1 << 0;
    pub const WRITE: u32 = 1 << 1;
    pub const COLOR_ATTACHMENT_READ: u32 = 1 << 2;
    pub const COLOR_ATTACHMENT_WRITE: u32 = 1 << 3;
    pub const DEPTH_STENCIL_READ: u32 = 1 << 4;
    pub const DEPTH_STENCIL_WRITE: u32 = 1 << 5;
    pub const TRANSFER_READ: u32 = 1 << 6;
    pub const TRANSFER_WRITE: u32 = 1 << 7;
    pub const SHADER_READ: u32 = 1 << 8;
    pub const SHADER_WRITE: u32 = 1 << 9;

    pub fn new(bits: u32) -> Self {
        Self { bits }
    }

    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn contains(&self, flag: u32) -> bool {
        (self.bits & flag) != 0
    }

    /// Convert AccessType to MemoryAccess flags
    pub fn from_access_type(access_type: AccessType, layout: Option<ImageLayout>) -> Self {
        let mut bits = 0;

        match access_type {
            AccessType::Read => {
                bits |= Self::READ;
                if let Some(layout) = layout {
                    match layout {
                        ImageLayout::ColorAttachment => bits |= Self::COLOR_ATTACHMENT_READ,
                        ImageLayout::DepthStencilAttachment => bits |= Self::DEPTH_STENCIL_READ,
                        ImageLayout::ShaderReadOnly => bits |= Self::SHADER_READ,
                        ImageLayout::TransferSrc => bits |= Self::TRANSFER_READ,
                        _ => {}
                    }
                }
            }
            AccessType::Write => {
                bits |= Self::WRITE;
                if let Some(layout) = layout {
                    match layout {
                        ImageLayout::ColorAttachment => bits |= Self::COLOR_ATTACHMENT_WRITE,
                        ImageLayout::DepthStencilAttachment => bits |= Self::DEPTH_STENCIL_WRITE,
                        ImageLayout::TransferDst => bits |= Self::TRANSFER_WRITE,
                        _ => {}
                    }
                }
            }
            AccessType::ReadWrite => {
                bits |= Self::READ | Self::WRITE;
                bits |= Self::SHADER_READ | Self::SHADER_WRITE;
            }
        }

        Self { bits }
    }
}

/// Memory barrier for synchronization
#[derive(Debug, Clone)]
pub struct MemoryBarrier {
    /// Source access mask
    pub src_access: MemoryAccess,
    /// Destination access mask
    pub dst_access: MemoryAccess,
    /// Source pipeline stage
    pub src_stage: PipelineStage,
    /// Destination pipeline stage
    pub dst_stage: PipelineStage,
}

impl MemoryBarrier {
    /// Create a new memory barrier
    pub fn new(
        src_access: MemoryAccess,
        dst_access: MemoryAccess,
        src_stage: PipelineStage,
        dst_stage: PipelineStage,
    ) -> Self {
        Self {
            src_access,
            dst_access,
            src_stage,
            dst_stage,
        }
    }
}

/// Image barrier for layout transitions and synchronization
#[derive(Debug, Clone)]
pub struct ImageBarrier {
    /// Resource being transitioned
    pub resource: ResourceId,
    /// Source access mask
    pub src_access: MemoryAccess,
    /// Destination access mask
    pub dst_access: MemoryAccess,
    /// Source pipeline stage
    pub src_stage: PipelineStage,
    /// Destination pipeline stage
    pub dst_stage: PipelineStage,
    /// Old layout
    pub old_layout: ImageLayout,
    /// New layout
    pub new_layout: ImageLayout,
}

impl ImageBarrier {
    /// Create a new image barrier
    pub fn new(
        resource: ResourceId,
        src_access: MemoryAccess,
        dst_access: MemoryAccess,
        src_stage: PipelineStage,
        dst_stage: PipelineStage,
        old_layout: ImageLayout,
        new_layout: ImageLayout,
    ) -> Self {
        Self {
            resource,
            src_access,
            dst_access,
            src_stage,
            dst_stage,
            old_layout,
            new_layout,
        }
    }
}

/// A barrier between two passes
#[derive(Debug, Clone)]
pub struct Barrier {
    /// Pass that comes before the barrier
    pub src_pass: PassId,
    /// Pass that comes after the barrier
    pub dst_pass: PassId,
    /// Type of barrier
    pub barrier_type: BarrierType,
    /// Memory barrier (if applicable)
    pub memory_barrier: Option<MemoryBarrier>,
    /// Image barriers (for layout transitions)
    pub image_barriers: Vec<ImageBarrier>,
}

impl Barrier {
    /// Create a new barrier
    pub fn new(src_pass: PassId, dst_pass: PassId) -> Self {
        Self {
            src_pass,
            dst_pass,
            barrier_type: BarrierType::Pipeline,
            memory_barrier: None,
            image_barriers: Vec::new(),
        }
    }

    /// Add a memory barrier
    pub fn with_memory_barrier(mut self, barrier: MemoryBarrier) -> Self {
        self.memory_barrier = Some(barrier);
        self.barrier_type = BarrierType::Memory;
        self
    }

    /// Add an image barrier
    pub fn add_image_barrier(&mut self, barrier: ImageBarrier) {
        self.image_barriers.push(barrier);
        if self.barrier_type != BarrierType::Memory {
            self.barrier_type = BarrierType::ImageTransition;
        }
    }

    /// Check if this barrier has any actual synchronization
    pub fn is_empty(&self) -> bool {
        self.memory_barrier.is_none() && self.image_barriers.is_empty()
    }
}

/// Tracks the last known state of a resource
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields will be used for advanced barrier optimization
struct ResourceState {
    /// Last pass that accessed this resource
    last_pass: PassId,
    /// Last access type
    last_access: AccessType,
    /// Last pipeline stage
    last_stage: PipelineStage,
    /// Last image layout (for images)
    last_layout: Option<ImageLayout>,
}

/// Barrier insertion analyzer
pub struct BarrierInserter {
    /// Current state of each resource
    resource_states: HashMap<ResourceId, ResourceState>,
}

impl BarrierInserter {
    /// Create a new barrier inserter
    pub fn new() -> Self {
        Self {
            resource_states: HashMap::new(),
        }
    }

    /// Analyze resource access and generate barriers between two passes
    pub fn analyze_transition(
        &mut self,
        src_pass: PassId,
        dst_pass: PassId,
        src_outputs: &[ResourceAccess],
        dst_inputs: &[ResourceAccess],
        resource_kinds: &HashMap<ResourceId, ResourceKind>,
    ) -> Barrier {
        let mut barrier = Barrier::new(src_pass, dst_pass);

        // Find resources that are written by src and read by dst
        for src_output in src_outputs {
            for dst_input in dst_inputs {
                if src_output.resource == dst_input.resource {
                    // This resource has a dependency
                    self.insert_barrier_for_resource(
                        &mut barrier,
                        src_output,
                        dst_input,
                        resource_kinds.get(&src_output.resource),
                    );
                }
            }
        }

        // Update resource states
        for output in src_outputs {
            self.resource_states.insert(
                output.resource,
                ResourceState {
                    last_pass: src_pass,
                    last_access: output.access_type,
                    last_stage: output.stage,
                    last_layout: output.layout,
                },
            );
        }

        barrier
    }

    /// Insert appropriate barrier for a specific resource
    fn insert_barrier_for_resource(
        &mut self,
        barrier: &mut Barrier,
        src_access: &ResourceAccess,
        dst_access: &ResourceAccess,
        resource_kind: Option<&ResourceKind>,
    ) {
        let src_memory = MemoryAccess::from_access_type(src_access.access_type, src_access.layout);
        let dst_memory = MemoryAccess::from_access_type(dst_access.access_type, dst_access.layout);

        // Check if this is an image with a layout transition
        if let Some(ResourceKind::Image) = resource_kind {
            if let (Some(old_layout), Some(new_layout)) = (src_access.layout, dst_access.layout) {
                if old_layout != new_layout {
                    // Need an image layout transition
                    barrier.add_image_barrier(ImageBarrier::new(
                        src_access.resource,
                        src_memory,
                        dst_memory,
                        src_access.stage,
                        dst_access.stage,
                        old_layout,
                        new_layout,
                    ));
                    return;
                }
            }
        }

        // Otherwise, just a memory barrier if needed
        if src_memory.bits != 0 || dst_memory.bits != 0 {
            barrier.memory_barrier = Some(MemoryBarrier::new(
                src_memory,
                dst_memory,
                src_access.stage,
                dst_access.stage,
            ));
        }
    }

    /// Optimize barriers by merging adjacent ones
    pub fn optimize_barriers(barriers: Vec<Barrier>) -> Vec<Barrier> {
        // For now, just filter out empty barriers
        // Future optimization: merge consecutive barriers with same src/dst
        barriers.into_iter().filter(|b| !b.is_empty()).collect()
    }
}

impl Default for BarrierInserter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_graph::resource::ResourceKind;

    #[test]
    fn test_memory_access_from_access_type() {
        let read = MemoryAccess::from_access_type(AccessType::Read, None);
        assert!(read.contains(MemoryAccess::READ));
        assert!(!read.contains(MemoryAccess::WRITE));

        let write = MemoryAccess::from_access_type(AccessType::Write, None);
        assert!(write.contains(MemoryAccess::WRITE));
        assert!(!write.contains(MemoryAccess::READ));

        let rw = MemoryAccess::from_access_type(AccessType::ReadWrite, None);
        assert!(rw.contains(MemoryAccess::READ));
        assert!(rw.contains(MemoryAccess::WRITE));
    }

    #[test]
    fn test_barrier_creation() {
        let barrier = Barrier::new(PassId(0), PassId(1));
        assert_eq!(barrier.src_pass, PassId(0));
        assert_eq!(barrier.dst_pass, PassId(1));
        assert!(barrier.is_empty());
    }

    #[test]
    fn test_image_layout_transition() {
        let mut inserter = BarrierInserter::new();
        let resource = ResourceId(0);

        let src = ResourceAccess::new(
            resource,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            Some(ImageLayout::ColorAttachment),
        );

        let dst = ResourceAccess::new(
            resource,
            AccessType::Read,
            PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
            Some(ImageLayout::ShaderReadOnly),
        );

        let mut kinds = HashMap::new();
        kinds.insert(resource, ResourceKind::Image);

        let barrier = inserter.analyze_transition(PassId(0), PassId(1), &[src], &[dst], &kinds);

        assert!(!barrier.is_empty());
        assert_eq!(barrier.image_barriers.len(), 1);
        assert_eq!(
            barrier.image_barriers[0].old_layout,
            ImageLayout::ColorAttachment
        );
        assert_eq!(
            barrier.image_barriers[0].new_layout,
            ImageLayout::ShaderReadOnly
        );
    }

    #[test]
    fn test_memory_barrier_no_layout_change() {
        let mut inserter = BarrierInserter::new();
        let resource = ResourceId(0);

        let src = ResourceAccess::new(
            resource,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            Some(ImageLayout::ColorAttachment),
        );

        let dst = ResourceAccess::new(
            resource,
            AccessType::Read,
            PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
            Some(ImageLayout::ColorAttachment), // Same layout
        );

        let mut kinds = HashMap::new();
        kinds.insert(resource, ResourceKind::Image);

        let barrier = inserter.analyze_transition(PassId(0), PassId(1), &[src], &[dst], &kinds);

        // Should have memory barrier but no image barrier
        assert!(!barrier.is_empty());
        assert!(barrier.memory_barrier.is_some());
        assert_eq!(barrier.image_barriers.len(), 0);
    }

    #[test]
    fn test_optimize_barriers_filters_empty() {
        let barriers = vec![
            Barrier::new(PassId(0), PassId(1)),
            Barrier::new(PassId(1), PassId(2)).with_memory_barrier(MemoryBarrier::new(
                MemoryAccess::new(MemoryAccess::WRITE),
                MemoryAccess::new(MemoryAccess::READ),
                PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
                PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
            )),
            Barrier::new(PassId(2), PassId(3)),
        ];

        let optimized = BarrierInserter::optimize_barriers(barriers);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].src_pass, PassId(1));
    }
}
