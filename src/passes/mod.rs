//! Render passes
//!
//! This module contains reusable render pass definitions. Each pass is
//! responsible for a specific rendering task and can be composed into
//! render graphs.

pub mod forward;
pub mod triangle_pass;
pub mod vertex_buffer_triangle;

pub use forward::ForwardPass;
pub use triangle_pass::TrianglePass;
pub use vertex_buffer_triangle::{VertexBufferTrianglePass, VertexBufferTrianglePassBuilder};
