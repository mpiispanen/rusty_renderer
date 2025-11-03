//! Render passes
//!
//! This module contains reusable render pass definitions. Each pass is
//! responsible for a specific rendering task and can be composed into
//! render graphs.

pub mod forward;
pub mod forward_declarative;
pub mod forward_pass_builder;
pub mod forward_simple;

pub mod vertex_buffer_triangle;

pub use forward::ForwardPass;
pub use forward_declarative::ForwardDeclarativePass;
pub use forward_pass_builder::{ForwardRenderPass, ForwardRenderPassBuilder};
pub use forward_simple::{
    ForwardSimplePass, ForwardSimplePassBuilder, ForwardSimpleSceneResources,
};

pub use vertex_buffer_triangle::{VertexBufferTrianglePass, VertexBufferTrianglePassBuilder};
