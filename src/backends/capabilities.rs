//! Backend capabilities and quirks
//!
//! Different graphics APIs have different conventions and capabilities.
//! This module defines traits and structures to query and handle these differences.

/// Clip space origin convention
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipSpaceOrigin {
    /// Origin at top-left (Vulkan, DirectX)
    TopLeft,
    /// Origin at bottom-left (OpenGL, Metal)
    BottomLeft,
}

/// Backend-specific capabilities and conventions
#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    /// Name of the backend
    pub name: &'static str,

    /// Y-axis points upward in NDC (true for OpenGL/wgpu, false for Vulkan/DirectX)
    pub y_axis_up: bool,

    /// Depth range is [0, 1] (true) or [-1, 1] (false)
    pub depth_range_zero_to_one: bool,

    /// Clip space origin (top-left or bottom-left)
    pub clip_space_origin: ClipSpaceOrigin,

    /// Supports ray tracing
    pub supports_ray_tracing: bool,

    /// Supports mesh shaders
    pub supports_mesh_shaders: bool,

    /// Supports compute shaders
    pub supports_compute_shaders: bool,
}

impl BackendCapabilities {
    /// Create capabilities for Vulkan backend
    pub fn vulkan() -> Self {
        Self {
            name: "Vulkan",
            y_axis_up: false, // Y points DOWN
            depth_range_zero_to_one: true,
            clip_space_origin: ClipSpaceOrigin::TopLeft,
            supports_ray_tracing: true,
            supports_mesh_shaders: true,
            supports_compute_shaders: true,
        }
    }

    /// Create capabilities for DirectX 12 backend
    pub fn directx12() -> Self {
        Self {
            name: "DirectX 12",
            y_axis_up: false, // Y points DOWN
            depth_range_zero_to_one: true,
            clip_space_origin: ClipSpaceOrigin::TopLeft,
            supports_ray_tracing: true,
            supports_mesh_shaders: true,
            supports_compute_shaders: true,
        }
    }

    /// Create capabilities for wgpu backend
    pub fn wgpu() -> Self {
        Self {
            name: "wgpu",
            y_axis_up: true, // Y points UP (like OpenGL)
            depth_range_zero_to_one: true,
            clip_space_origin: ClipSpaceOrigin::TopLeft,
            supports_ray_tracing: false, // Not in WebGPU spec yet
            supports_mesh_shaders: false,
            supports_compute_shaders: true,
        }
    }

    /// Check if Y-axis needs to be flipped relative to canonical Y-up convention
    pub fn needs_y_flip(&self) -> bool {
        // If backend has Y pointing down, we need to flip
        !self.y_axis_up
    }

    /// Get the Y-axis multiplier for transformations
    /// Returns -1.0 if Y needs to be flipped, 1.0 otherwise
    pub fn y_axis_multiplier(&self) -> f32 {
        if self.needs_y_flip() {
            -1.0
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulkan_capabilities() {
        let caps = BackendCapabilities::vulkan();
        assert_eq!(caps.name, "Vulkan");
        assert!(!caps.y_axis_up);
        assert!(caps.depth_range_zero_to_one);
        assert!(caps.needs_y_flip());
        assert_eq!(caps.y_axis_multiplier(), -1.0);
    }

    #[test]
    fn test_wgpu_capabilities() {
        let caps = BackendCapabilities::wgpu();
        assert_eq!(caps.name, "wgpu");
        assert!(caps.y_axis_up);
        assert!(!caps.needs_y_flip());
        assert_eq!(caps.y_axis_multiplier(), 1.0);
    }

    #[test]
    fn test_coordinate_system_differences() {
        let vulkan = BackendCapabilities::vulkan();
        let wgpu = BackendCapabilities::wgpu();

        // Vulkan and wgpu have opposite Y-axis conventions
        assert_ne!(vulkan.y_axis_up, wgpu.y_axis_up);
        assert_ne!(vulkan.needs_y_flip(), wgpu.needs_y_flip());
    }
}
