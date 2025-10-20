//! GPU resource abstractions (buffers and textures)
//!
//! This module defines the core resource types used by the renderer:
//! - Buffers: vertex, index, uniform, staging
//! - Textures: 2D images with various formats
//! - Samplers: texture filtering and addressing
//!
//! These abstractions work across all backends (Vulkan, DirectX 12, wgpu).

use anyhow::Result;
use std::any::Any;

/// Buffer usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferUsage {
    /// Can be used as a vertex buffer
    pub vertex: bool,
    /// Can be used as an index buffer
    pub index: bool,
    /// Can be used as a uniform buffer
    pub uniform: bool,
    /// Can be used as a staging buffer for CPU->GPU transfers
    pub staging: bool,
    /// Can be used as a storage buffer
    pub storage: bool,
    /// Can be used as a transfer source
    pub transfer_src: bool,
    /// Can be used as a transfer destination
    pub transfer_dst: bool,
}

impl BufferUsage {
    /// Create vertex buffer usage
    pub fn vertex() -> Self {
        Self {
            vertex: true,
            index: false,
            uniform: false,
            staging: false,
            storage: false,
            transfer_src: false,
            transfer_dst: true,
        }
    }

    /// Create index buffer usage
    pub fn index() -> Self {
        Self {
            vertex: false,
            index: true,
            uniform: false,
            staging: false,
            storage: false,
            transfer_src: false,
            transfer_dst: true,
        }
    }

    /// Create uniform buffer usage
    pub fn uniform() -> Self {
        Self {
            vertex: false,
            index: false,
            uniform: true,
            staging: false,
            storage: false,
            transfer_src: false,
            transfer_dst: true,
        }
    }

    /// Create staging buffer usage (CPU-accessible)
    pub fn staging() -> Self {
        Self {
            vertex: false,
            index: false,
            uniform: false,
            staging: true,
            storage: false,
            transfer_src: true,
            transfer_dst: false,
        }
    }
}

/// Memory location preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryLocation {
    /// GPU-only memory (fast, not CPU-accessible)
    GpuOnly,
    /// CPU-visible memory (slower, CPU-accessible)
    CpuToGpu,
    /// GPU-readable, CPU-writable (for dynamic updates)
    GpuToCpu,
}

/// Buffer descriptor for creation
#[derive(Debug, Clone)]
pub struct BufferDescriptor {
    /// Size in bytes
    pub size: u64,
    /// Buffer usage flags
    pub usage: BufferUsage,
    /// Memory location preference
    pub memory_location: MemoryLocation,
    /// Debug label
    pub label: Option<String>,
}

/// Buffer trait for GPU memory buffers
pub trait Buffer: Send + Sync {
    /// Get the buffer size in bytes
    fn size(&self) -> u64;

    /// Get the buffer usage flags
    fn usage(&self) -> BufferUsage;

    /// Get the memory location
    fn memory_location(&self) -> MemoryLocation;

    /// Map the buffer for CPU access (if CPU-accessible)
    ///
    /// Returns a mutable slice to the buffer data. Only works for
    /// buffers created with CpuToGpu or GpuToCpu memory location.
    ///
    /// # Safety
    /// The returned slice is only valid until `unmap()` is called.
    fn map(&mut self) -> Result<&mut [u8]>;

    /// Unmap the buffer after CPU access
    fn unmap(&mut self);

    /// Get as Any for downcasting to backend-specific types
    fn as_any(&self) -> &dyn Any;

    /// Get as mutable Any for downcasting to backend-specific types
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Texture format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureFormat {
    /// 8-bit RGBA (sRGB color space)
    Rgba8Srgb,
    /// 8-bit RGBA (linear color space)
    Rgba8Unorm,
    /// 8-bit BGRA (sRGB color space)
    Bgra8Srgb,
    /// 8-bit BGRA (linear color space)
    Bgra8Unorm,
    /// 32-bit depth
    Depth32Float,
    /// 24-bit depth + 8-bit stencil
    Depth24PlusStencil8,
}

impl TextureFormat {
    /// Get the number of bytes per pixel
    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            TextureFormat::Rgba8Srgb
            | TextureFormat::Rgba8Unorm
            | TextureFormat::Bgra8Srgb
            | TextureFormat::Bgra8Unorm
            | TextureFormat::Depth24PlusStencil8 => 4,
            TextureFormat::Depth32Float => 4,
        }
    }

    /// Check if format is a depth format
    pub fn is_depth(&self) -> bool {
        matches!(
            self,
            TextureFormat::Depth32Float | TextureFormat::Depth24PlusStencil8
        )
    }
}

/// Texture usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureUsage {
    /// Can be sampled in shaders
    pub sampled: bool,
    /// Can be used as a render target
    pub render_target: bool,
    /// Can be used as a depth/stencil target
    pub depth_stencil: bool,
    /// Can be used as a transfer source
    pub transfer_src: bool,
    /// Can be used as a transfer destination
    pub transfer_dst: bool,
}

impl TextureUsage {
    /// Create texture usage for sampling
    pub fn sampled() -> Self {
        Self {
            sampled: true,
            render_target: false,
            depth_stencil: false,
            transfer_src: false,
            transfer_dst: true,
        }
    }

    /// Create texture usage for render target
    pub fn render_target() -> Self {
        Self {
            sampled: false,
            render_target: true,
            depth_stencil: false,
            transfer_src: true,
            transfer_dst: false,
        }
    }

    /// Create texture usage for depth/stencil
    pub fn depth_stencil() -> Self {
        Self {
            sampled: false,
            render_target: false,
            depth_stencil: true,
            transfer_src: false,
            transfer_dst: false,
        }
    }
}

/// Texture descriptor for creation
#[derive(Debug, Clone)]
pub struct TextureDescriptor<'a> {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Texture format
    pub format: TextureFormat,
    /// Texture usage flags
    pub usage: TextureUsage,
    /// Number of mip levels (1 = no mipmaps)
    pub mip_levels: u32,
    /// Optional initial data to upload (RGBA8 format, must match dimensions)
    pub initial_data: Option<&'a [u8]>,
    /// Debug label
    pub label: Option<String>,
}

/// Texture trait for GPU textures
pub trait Texture: Send + Sync {
    /// Get the texture width
    fn width(&self) -> u32;

    /// Get the texture height
    fn height(&self) -> u32;

    /// Get the texture format
    fn format(&self) -> TextureFormat;

    /// Get the texture usage
    fn usage(&self) -> TextureUsage;

    /// Get the number of mip levels
    fn mip_levels(&self) -> u32;

    /// Get as Any for downcasting to backend-specific types
    fn as_any(&self) -> &dyn Any;

    /// Get as mutable Any for downcasting to backend-specific types
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Sampler filter mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterMode {
    /// Nearest neighbor filtering
    Nearest,
    /// Linear filtering
    Linear,
}

/// Sampler address mode (texture wrap behavior)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddressMode {
    /// Clamp to edge
    ClampToEdge,
    /// Repeat (wrap)
    Repeat,
    /// Mirror repeat
    MirrorRepeat,
}

/// Sampler descriptor for creation
#[derive(Debug, Clone)]
pub struct SamplerDescriptor {
    /// Magnification filter
    pub mag_filter: FilterMode,
    /// Minification filter
    pub min_filter: FilterMode,
    /// Mipmap filter
    pub mipmap_filter: FilterMode,
    /// Address mode for U coordinate
    pub address_mode_u: AddressMode,
    /// Address mode for V coordinate
    pub address_mode_v: AddressMode,
    /// Address mode for W coordinate
    pub address_mode_w: AddressMode,
    /// Debug label
    pub label: Option<String>,
}

impl Default for SamplerDescriptor {
    fn default() -> Self {
        Self {
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            label: None,
        }
    }
}

/// Sampler trait for texture sampling
pub trait Sampler: Send + Sync {
    /// Get as Any for downcasting to backend-specific types
    fn as_any(&self) -> &dyn Any;

    /// Get as mutable Any for downcasting to backend-specific types
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_usage_vertex() {
        let usage = BufferUsage::vertex();
        assert!(usage.vertex);
        assert!(!usage.index);
        assert!(usage.transfer_dst);
    }

    #[test]
    fn test_buffer_usage_index() {
        let usage = BufferUsage::index();
        assert!(!usage.vertex);
        assert!(usage.index);
        assert!(usage.transfer_dst);
    }

    #[test]
    fn test_buffer_usage_uniform() {
        let usage = BufferUsage::uniform();
        assert!(usage.uniform);
        assert!(usage.transfer_dst);
    }

    #[test]
    fn test_buffer_usage_staging() {
        let usage = BufferUsage::staging();
        assert!(usage.staging);
        assert!(usage.transfer_src);
        assert!(!usage.transfer_dst);
    }

    #[test]
    fn test_texture_format_bytes() {
        assert_eq!(TextureFormat::Rgba8Srgb.bytes_per_pixel(), 4);
        assert_eq!(TextureFormat::Depth32Float.bytes_per_pixel(), 4);
    }

    #[test]
    fn test_texture_format_is_depth() {
        assert!(!TextureFormat::Rgba8Srgb.is_depth());
        assert!(TextureFormat::Depth32Float.is_depth());
        assert!(TextureFormat::Depth24PlusStencil8.is_depth());
    }

    #[test]
    fn test_texture_usage_sampled() {
        let usage = TextureUsage::sampled();
        assert!(usage.sampled);
        assert!(usage.transfer_dst);
    }

    #[test]
    fn test_sampler_descriptor_default() {
        let desc = SamplerDescriptor::default();
        assert_eq!(desc.mag_filter, FilterMode::Linear);
        assert_eq!(desc.address_mode_u, AddressMode::ClampToEdge);
    }
}
