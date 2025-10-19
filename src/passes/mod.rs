//! Render passes
//!
//! This module contains reusable render pass definitions. Each pass is
//! responsible for a specific rendering task and can be composed into
//! render graphs.

pub mod triangle_pass;

pub use triangle_pass::TrianglePass;
