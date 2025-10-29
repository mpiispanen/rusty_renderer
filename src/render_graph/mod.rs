//! Render graph system
//!
//! This module implements a frame graph / render graph system that automatically
//! manages dependencies between render passes, resource lifetimes, and barrier
//! insertion for optimal GPU performance.

mod barrier;
pub mod graph;
mod pass;
mod resource;
mod shader;

pub use barrier::{Barrier, BarrierType, ImageBarrier, MemoryAccess, MemoryBarrier};
pub use graph::{CompiledGraph, RenderGraph};
pub use pass::{
    AccessType, DeclarativePass, ImageLayout, IndexType, PassBuilder, PassCallback,
    PassExecutionContext, PassId, PassKind, PassPreparationContext, PipelineStage, RenderPass,
    ResourceAccess,
};
pub use resource::{
    AddressMode, BufferUsageFlags, Extent3D, ExtentMode, FilterMode, Format, ImageUsageFlags,
    Resource, ResourceDescriptor, ResourceId, ResourceKind, ResourceLifetime, SampleCount,
    SamplerDescriptor,
};
pub use shader::{
    CompiledShader, ShaderDescriptor, ShaderError, ShaderHandle, ShaderRegistry, ShaderSource,
    ShaderStage,
};
