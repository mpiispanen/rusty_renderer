//! Render graph resource types
//!
//! This module defines resources (images, buffers) used in the render graph
//! and tracks their lifetimes and usage.

use std::fmt;

/// Unique identifier for a resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(pub usize);

impl fmt::Display for ResourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Resource({})", self.0)
    }
}

/// Type of resource
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    /// Image/texture resource
    Image,
    /// Buffer resource
    Buffer,
    /// Sampler resource
    Sampler,
}

/// Image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// 8-bit RGBA
    Rgba8Unorm,
    /// 8-bit BGRA
    Bgra8Unorm,
    /// 16-bit RGBA float
    Rgba16Float,
    /// 32-bit RGBA float
    Rgba32Float,
    /// 24-bit depth, 8-bit stencil
    Depth24Stencil8,
    /// 32-bit depth float
    Depth32Float,
}

/// 3D extent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Extent3D {
    /// Create a 2D extent (depth = 1)
    pub fn new_2d(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            depth: 1,
        }
    }
}

/// Sample count for multisampling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleCount {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
}

/// Image usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageUsageFlags {
    bits: u32,
}

impl ImageUsageFlags {
    pub const TRANSFER_SRC: u32 = 1 << 0;
    pub const TRANSFER_DST: u32 = 1 << 1;
    pub const SAMPLED: u32 = 1 << 2;
    pub const STORAGE: u32 = 1 << 3;
    pub const COLOR_ATTACHMENT: u32 = 1 << 4;
    pub const DEPTH_STENCIL_ATTACHMENT: u32 = 1 << 5;

    pub fn new(bits: u32) -> Self {
        Self { bits }
    }

    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn contains(&self, flag: u32) -> bool {
        (self.bits & flag) != 0
    }
}

/// Buffer usage flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferUsageFlags {
    bits: u32,
}

impl BufferUsageFlags {
    pub const TRANSFER_SRC: u32 = 1 << 0;
    pub const TRANSFER_DST: u32 = 1 << 1;
    pub const UNIFORM: u32 = 1 << 2;
    pub const STORAGE: u32 = 1 << 3;
    pub const INDEX: u32 = 1 << 4;
    pub const VERTEX: u32 = 1 << 5;
    pub const INDIRECT: u32 = 1 << 6;

    pub fn new(bits: u32) -> Self {
        Self { bits }
    }

    pub fn empty() -> Self {
        Self { bits: 0 }
    }

    pub fn contains(&self, flag: u32) -> bool {
        (self.bits & flag) != 0
    }
}

/// Resource descriptor defining resource properties
#[derive(Debug, Clone)]
pub enum ResourceDescriptor {
    /// Image descriptor
    Image {
        format: Format,
        extent: Extent3D,
        usage: ImageUsageFlags,
        samples: SampleCount,
    },
    /// Buffer descriptor
    Buffer {
        size: usize,
        usage: BufferUsageFlags,
    },
    /// Sampler descriptor
    Sampler,
}

/// Resource lifetime tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLifetime {
    /// First pass that uses this resource
    pub first_use: Option<usize>,
    /// Last pass that uses this resource
    pub last_use: Option<usize>,
    /// Whether this resource can be aliased with others
    pub can_alias: bool,
}

impl ResourceLifetime {
    /// Create a new resource lifetime
    pub fn new() -> Self {
        Self {
            first_use: None,
            last_use: None,
            can_alias: true,
        }
    }

    /// Update lifetime to include a pass
    pub fn update(&mut self, pass_index: usize) {
        self.first_use = Some(
            self.first_use
                .map_or(pass_index, |first| first.min(pass_index)),
        );
        self.last_use = Some(
            self.last_use
                .map_or(pass_index, |last| last.max(pass_index)),
        );
    }

    /// Check if this resource is used in the given pass range
    pub fn overlaps(&self, other: &ResourceLifetime) -> bool {
        if let (Some(self_first), Some(self_last), Some(other_first), Some(other_last)) = (
            self.first_use,
            self.last_use,
            other.first_use,
            other.last_use,
        ) {
            !(self_last < other_first || other_last < self_first)
        } else {
            false
        }
    }
}

impl Default for ResourceLifetime {
    fn default() -> Self {
        Self::new()
    }
}

/// A resource in the render graph
#[derive(Debug, Clone)]
pub struct Resource {
    /// Unique identifier
    pub id: ResourceId,
    /// Resource name for debugging
    pub name: String,
    /// Resource type
    pub kind: ResourceKind,
    /// Resource descriptor
    pub descriptor: ResourceDescriptor,
    /// Lifetime tracking
    pub lifetime: ResourceLifetime,
}

impl Resource {
    /// Create a new resource
    pub fn new(id: ResourceId, name: impl Into<String>, descriptor: ResourceDescriptor) -> Self {
        let kind = match descriptor {
            ResourceDescriptor::Image { .. } => ResourceKind::Image,
            ResourceDescriptor::Buffer { .. } => ResourceKind::Buffer,
            ResourceDescriptor::Sampler => ResourceKind::Sampler,
        };

        Self {
            id,
            name: name.into(),
            kind,
            descriptor,
            lifetime: ResourceLifetime::new(),
        }
    }

    /// Get the resource name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Check if this is an image resource
    pub fn is_image(&self) -> bool {
        matches!(self.kind, ResourceKind::Image)
    }

    /// Check if this is a buffer resource
    pub fn is_buffer(&self) -> bool {
        matches!(self.kind, ResourceKind::Buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_creation() {
        let desc = ResourceDescriptor::Image {
            format: Format::Rgba8Unorm,
            extent: Extent3D::new_2d(1280, 720),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
        };

        let resource = Resource::new(ResourceId(0), "test_image", desc);
        assert_eq!(resource.name(), "test_image");
        assert!(resource.is_image());
        assert!(!resource.is_buffer());
    }

    #[test]
    fn test_lifetime_tracking() {
        let mut lifetime = ResourceLifetime::new();
        assert_eq!(lifetime.first_use, None);
        assert_eq!(lifetime.last_use, None);

        lifetime.update(5);
        assert_eq!(lifetime.first_use, Some(5));
        assert_eq!(lifetime.last_use, Some(5));

        lifetime.update(3);
        assert_eq!(lifetime.first_use, Some(3));
        assert_eq!(lifetime.last_use, Some(5));

        lifetime.update(7);
        assert_eq!(lifetime.first_use, Some(3));
        assert_eq!(lifetime.last_use, Some(7));
    }

    #[test]
    fn test_lifetime_overlap() {
        let mut lifetime1 = ResourceLifetime::new();
        lifetime1.update(2);
        lifetime1.update(5);

        let mut lifetime2 = ResourceLifetime::new();
        lifetime2.update(4);
        lifetime2.update(7);

        assert!(lifetime1.overlaps(&lifetime2));
        assert!(lifetime2.overlaps(&lifetime1));

        let mut lifetime3 = ResourceLifetime::new();
        lifetime3.update(8);
        lifetime3.update(10);

        assert!(!lifetime1.overlaps(&lifetime3));
        assert!(!lifetime3.overlaps(&lifetime1));
    }

    #[test]
    fn test_image_usage_flags() {
        let flags =
            ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::SAMPLED);
        assert!(flags.contains(ImageUsageFlags::COLOR_ATTACHMENT));
        assert!(flags.contains(ImageUsageFlags::SAMPLED));
        assert!(!flags.contains(ImageUsageFlags::STORAGE));
    }
}
