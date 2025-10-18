//! Graphics backend abstraction layer
//!
//! This module provides a common interface for different graphics APIs
//! (Vulkan, DirectX 12, and wgpu), allowing the renderer to work with
//! any supported backend through a unified trait-based API.
//!
//! # Architecture
//!
//! The backend abstraction is built around six core traits:
//! - [`GraphicsBackend`] - Main backend interface
//! - [`Device`] - GPU device management
//! - [`CommandBuffer`] - Command recording
//! - [`Pipeline`] - Graphics pipeline state
//! - [`Resource`] - GPU resources (buffers, textures)
//! - [`Swapchain`] - Presentation surface
//!
//! # Backend Selection
//!
//! Backends are selected at runtime via [`BackendType`] enum and created
//! using the [`create_backend()`] factory function.
//!
//! # Coordinate Systems
//!
//! Different graphics APIs use different coordinate system conventions.
//! See `docs/COORDINATE_SYSTEMS.md` for detailed information.
//!
//! # Example
//!
//! ```no_run
//! use rusty_renderer::backends::{BackendType, create_backend};
//!
//! // Create a Vulkan backend with validation disabled
//! let backend = create_backend(BackendType::Vulkan, false)?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::Result;
use std::any::Any;

// Backend capabilities and quirks
pub mod capabilities;

// Backend module declarations (implementations in M2+)
pub mod directx;
pub mod vulkan;
pub mod wgpu_backend;

/// Supported graphics backend types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendType {
    /// Vulkan backend (primary, Linux focus)
    Vulkan,
    /// DirectX 12 backend (Windows, Proton testing)
    DirectX12,
    /// wgpu backend (portability layer)
    Wgpu,
}

impl BackendType {
    /// Get the backend name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            BackendType::Vulkan => "vulkan",
            BackendType::DirectX12 => "directx12",
            BackendType::Wgpu => "wgpu",
        }
    }
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Main graphics backend trait
///
/// This trait defines the primary interface for all graphics backends.
/// It handles initialization, frame management, and resource access.
pub trait GraphicsBackend: Send + Sync {
    /// Get the backend type
    fn backend_type(&self) -> BackendType;

    /// Initialize the backend with a window
    ///
    /// This should create all necessary graphics resources and prepare
    /// the backend for rendering.
    fn initialize(&mut self, window: &winit::window::Window) -> Result<()>;

    /// Begin a new frame
    ///
    /// Called at the start of each frame. Acquires the next swapchain image
    /// and prepares for command recording.
    fn begin_frame(&mut self) -> Result<()>;

    /// End the current frame and present
    ///
    /// Submits recorded commands and presents the rendered image.
    fn end_frame(&mut self) -> Result<()>;

    /// Handle window resize
    ///
    /// Called when the window is resized. Should recreate swapchain and
    /// any size-dependent resources.
    fn resize(&mut self, width: u32, height: u32) -> Result<()>;

    /// Cleanup resources
    ///
    /// Called before the backend is dropped. Should release all GPU resources.
    fn cleanup(&mut self);

    /// Get the device interface
    fn device(&self) -> &dyn Device;

    /// Get the swapchain interface
    fn swapchain(&self) -> &dyn Swapchain;
}

/// Device creation and management trait
///
/// Provides access to GPU device information and capabilities.
pub trait Device: Send + Sync {
    /// Get the device name/description
    fn name(&self) -> &str;

    /// Check if a feature is supported
    ///
    /// Feature names are backend-specific strings describing capabilities.
    fn supports_feature(&self, feature: &str) -> bool;

    /// Get the underlying backend-specific device
    ///
    /// Use this for advanced backend-specific operations via downcasting.
    fn as_any(&self) -> &dyn Any;
}

/// Command buffer recording trait
///
/// Provides a common interface for recording rendering commands.
pub trait CommandBuffer: Send + Sync {
    /// Begin recording commands
    fn begin(&mut self) -> Result<()>;

    /// End recording commands
    fn end(&mut self) -> Result<()>;

    /// Clear the color target
    fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) -> Result<()>;

    /// Bind a graphics pipeline
    fn bind_pipeline(&mut self, pipeline: &dyn Pipeline) -> Result<()>;

    /// Draw primitives
    ///
    /// # Arguments
    /// * `vertex_count` - Number of vertices to draw
    /// * `instance_count` - Number of instances to draw
    fn draw(&mut self, vertex_count: u32, instance_count: u32) -> Result<()>;
}

/// Graphics pipeline abstraction trait
///
/// Represents a complete graphics pipeline state (shaders, render state, etc.)
pub trait Pipeline: Send + Sync {
    /// Get the pipeline name/identifier
    fn name(&self) -> &str;

    /// Get the underlying backend-specific pipeline
    ///
    /// Use this for advanced backend-specific operations via downcasting.
    fn as_any(&self) -> &dyn Any;
}

/// Resource management trait
///
/// Represents GPU resources such as buffers and textures.
pub trait Resource: Send + Sync {
    /// Get the resource size in bytes
    fn size(&self) -> usize;

    /// Get a description of the resource type
    fn resource_type(&self) -> &str;

    /// Get the underlying backend-specific resource
    ///
    /// Use this for advanced backend-specific operations via downcasting.
    fn as_any(&self) -> &dyn Any;
}

/// Swapchain/presentation surface management trait
///
/// Manages the presentation surface and image acquisition/presentation.
pub trait Swapchain: Send + Sync {
    /// Get the current swapchain width in pixels
    fn width(&self) -> u32;

    /// Get the current swapchain height in pixels
    fn height(&self) -> u32;

    /// Get the current frame index
    fn current_frame(&self) -> usize;

    /// Acquire the next image for rendering
    ///
    /// This should block or wait until an image is available.
    fn acquire_next_image(&mut self) -> Result<()>;

    /// Present the rendered image
    ///
    /// Submits the image to the presentation queue.
    fn present(&mut self) -> Result<()>;

    /// Recreate the swapchain
    ///
    /// Called when the window is resized or other swapchain properties change.
    fn recreate(&mut self, width: u32, height: u32) -> Result<()>;
}

/// Create a graphics backend of the specified type
///
/// # Arguments
/// * `backend_type` - The type of backend to create
/// * `enable_validation` - Enable validation/debug layers
///
/// # Returns
/// A boxed graphics backend ready for initialization
///
/// # Example
/// ```no_run
/// use rusty_renderer::backends::{BackendType, create_backend};
///
/// let backend = create_backend(BackendType::Vulkan, true)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn create_backend(
    backend_type: BackendType,
    enable_validation: bool,
) -> Result<Box<dyn GraphicsBackend>> {
    match backend_type {
        BackendType::Vulkan => Ok(Box::new(vulkan::VulkanBackend::new(enable_validation)?)),
        BackendType::DirectX12 => Ok(Box::new(directx::DirectXBackend::new(enable_validation)?)),
        BackendType::Wgpu => Ok(Box::new(wgpu_backend::WgpuBackend::new(enable_validation)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_equality() {
        assert_eq!(BackendType::Vulkan, BackendType::Vulkan);
        assert_ne!(BackendType::Vulkan, BackendType::DirectX12);
        assert_ne!(BackendType::Vulkan, BackendType::Wgpu);
        assert_ne!(BackendType::DirectX12, BackendType::Wgpu);
    }

    #[test]
    fn test_backend_type_display() {
        assert_eq!(BackendType::Vulkan.to_string(), "vulkan");
        assert_eq!(BackendType::DirectX12.to_string(), "directx12");
        assert_eq!(BackendType::Wgpu.to_string(), "wgpu");
    }

    #[test]
    fn test_backend_type_as_str() {
        assert_eq!(BackendType::Vulkan.as_str(), "vulkan");
        assert_eq!(BackendType::DirectX12.as_str(), "directx12");
        assert_eq!(BackendType::Wgpu.as_str(), "wgpu");
    }

    #[test]
    fn test_backend_type_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(BackendType::Vulkan);
        set.insert(BackendType::DirectX12);
        set.insert(BackendType::Wgpu);
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_backend_creation() {
        // All backends should be creatable (even if stubs)
        let vulkan = create_backend(BackendType::Vulkan, false);
        assert!(vulkan.is_ok(), "Failed to create Vulkan backend");

        let directx = create_backend(BackendType::DirectX12, false);
        assert!(directx.is_ok(), "Failed to create DirectX backend");

        let wgpu = create_backend(BackendType::Wgpu, false);
        assert!(wgpu.is_ok(), "Failed to create wgpu backend");
    }

    #[test]
    fn test_backend_type_identification() {
        let vulkan = create_backend(BackendType::Vulkan, false).unwrap();
        assert_eq!(vulkan.backend_type(), BackendType::Vulkan);

        let directx = create_backend(BackendType::DirectX12, false).unwrap();
        assert_eq!(directx.backend_type(), BackendType::DirectX12);

        let wgpu = create_backend(BackendType::Wgpu, false).unwrap();
        assert_eq!(wgpu.backend_type(), BackendType::Wgpu);
    }
}
