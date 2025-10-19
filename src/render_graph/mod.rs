//! Render graph system
//!
//! This module implements a frame graph / render graph system that automatically
//! manages dependencies between render passes, resource lifetimes, and barrier
//! insertion for optimal GPU performance.

mod barrier;
mod graph;
mod pass;
mod resource;

pub use barrier::{Barrier, BarrierType, MemoryBarrier};
pub use graph::RenderGraph;
pub use pass::{PassCallback, PassKind, RenderPass};
pub use resource::{Resource, ResourceDescriptor, ResourceKind, ResourceLifetime};
