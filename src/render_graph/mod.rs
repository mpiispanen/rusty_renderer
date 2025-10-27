//! Render graph system
//!
//! This module implements a frame graph / render graph system that automatically
//! manages dependencies between render passes, resource lifetimes, and barrier
//! insertion for optimal GPU performance.

mod barrier;
pub mod graph;
mod pass;
mod resource;

pub use barrier::{Barrier, BarrierType, ImageBarrier, MemoryAccess, MemoryBarrier};
pub use graph::{CompiledGraph, RenderGraph};
pub use pass::{
    AccessType, ImageLayout, IndexType, PassCallback, PassExecutionContext, PassId, PassKind,
    PassPreparationContext, PipelineStage, RenderPass, ResourceAccess,
};
pub use resource::{
    Extent3D, Format, ImageUsageFlags, Resource, ResourceDescriptor, ResourceId, ResourceKind,
    ResourceLifetime, SampleCount,
};
