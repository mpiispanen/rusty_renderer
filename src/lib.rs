//! Rusty Renderer - A multi-backend 3D renderer in Rust
//!
//! This crate provides a flexible 3D rendering framework with support for
//! multiple graphics backends (Vulkan, DirectX 12, wgpu).

// Module declarations - public for testing and library use
pub mod app;
pub mod application;
pub mod backends;
pub mod camera;
pub mod config;
pub mod lighting;
pub mod passes;
pub mod pipelines;
pub mod profiling;
pub mod render_graph;
pub mod resources;
pub mod scene;
pub mod shaders;
pub mod testing;
pub mod ui;

// Re-export main types for convenience
pub use app::App as RenderEngine;
pub use config::{Backend as RenderBackend, Config as RenderConfig};
