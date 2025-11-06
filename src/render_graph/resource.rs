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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    /// Create a 3D extent
    pub fn new_3d(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }
}

/// Extent mode for flexible resource sizing
///
/// Allows resources to be sized relative to the swapchain or with absolute dimensions.
#[derive(Debug, Clone, Copy)]
pub enum ExtentMode {
    /// Absolute size in pixels
    Absolute(Extent3D),
    /// Match swapchain size exactly (1.0x scale)
    Swapchain,
    /// Scale relative to swapchain (e.g., 0.5 = half size, 2.0 = double size)
    SwapchainScaled(f32),
}

impl PartialEq for ExtentMode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExtentMode::Absolute(a), ExtentMode::Absolute(b)) => a == b,
            (ExtentMode::Swapchain, ExtentMode::Swapchain) => true,
            (ExtentMode::SwapchainScaled(a), ExtentMode::SwapchainScaled(b)) => {
                (a - b).abs() < f32::EPSILON
            }
            _ => false,
        }
    }
}

impl ExtentMode {
    /// Resolve the actual extent given the swapchain dimensions
    ///
    /// # Arguments
    /// * `swapchain_width` - Current swapchain width
    /// * `swapchain_height` - Current swapchain height
    ///
    /// # Returns
    /// The resolved Extent3D
    pub fn resolve(&self, swapchain_width: u32, swapchain_height: u32) -> Extent3D {
        match self {
            ExtentMode::Absolute(extent) => *extent,
            ExtentMode::Swapchain => Extent3D::new_2d(swapchain_width, swapchain_height),
            ExtentMode::SwapchainScaled(scale) => {
                let width = (swapchain_width as f32 * scale).max(1.0) as u32;
                let height = (swapchain_height as f32 * scale).max(1.0) as u32;
                Extent3D::new_2d(width, height)
            }
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

/// Sampler filter mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    Nearest,
    Linear,
}

/// Sampler address mode (texture wrapping)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMode {
    Repeat,
    MirroredRepeat,
    ClampToEdge,
    ClampToBorder,
}

/// Sampler descriptor
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SamplerDescriptor {
    /// Minification filter
    pub min_filter: FilterMode,
    /// Magnification filter
    pub mag_filter: FilterMode,
    /// Mipmap filter
    pub mipmap_filter: FilterMode,
    /// Address mode for U coordinate
    pub address_mode_u: AddressMode,
    /// Address mode for V coordinate
    pub address_mode_v: AddressMode,
    /// Address mode for W coordinate
    pub address_mode_w: AddressMode,
    /// Maximum anisotropy (1.0 = disabled, 16.0 = max)
    pub max_anisotropy: f32,
}

impl Default for SamplerDescriptor {
    fn default() -> Self {
        Self {
            min_filter: FilterMode::Linear,
            mag_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            max_anisotropy: 1.0,
        }
    }
}

/// Resource descriptor defining resource properties
#[derive(Debug, Clone)]
pub enum ResourceDescriptor {
    /// Image descriptor
    Image {
        format: Format,
        extent: ExtentMode,
        usage: ImageUsageFlags,
        samples: SampleCount,
        mip_levels: u32,
    },
    /// Buffer descriptor
    Buffer {
        size: usize,
        usage: BufferUsageFlags,
    },
    /// Sampler descriptor
    Sampler(SamplerDescriptor),
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

/// Resource initialization data
///
/// Used to upload initial data to a resource after allocation.
#[derive(Debug, Clone)]
pub enum ResourceInitData {
    /// No initial data - resource will be uninitialized
    None,
    /// Upload data from a buffer
    Buffer(Vec<u8>),
}

impl Default for ResourceInitData {
    fn default() -> Self {
        Self::None
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
    /// Whether this is an external resource (not managed by render graph)
    pub external: bool,
    /// Initial data to upload to the resource
    pub init_data: ResourceInitData,
}

impl Resource {
    /// Create a new resource
    pub fn new(id: ResourceId, name: impl Into<String>, descriptor: ResourceDescriptor) -> Self {
        let kind = match descriptor {
            ResourceDescriptor::Image { .. } => ResourceKind::Image,
            ResourceDescriptor::Buffer { .. } => ResourceKind::Buffer,
            ResourceDescriptor::Sampler(_) => ResourceKind::Sampler,
        };

        Self {
            id,
            name: name.into(),
            kind,
            descriptor,
            lifetime: ResourceLifetime::new(),
            external: false,
            init_data: ResourceInitData::None,
        }
    }

    /// Set initial data for this resource
    pub fn with_init_data(mut self, data: Vec<u8>) -> Self {
        self.init_data = ResourceInitData::Buffer(data);
        self
    }

    /// Set initial data for this resource (mutable)
    pub fn set_init_data(&mut self, data: Vec<u8>) {
        self.init_data = ResourceInitData::Buffer(data);
    }

    /// Mark this resource as external (not managed by render graph)
    pub fn mark_external(&mut self) {
        self.external = true;
    }

    /// Check if this resource is external
    pub fn is_external(&self) -> bool {
        self.external
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
            extent: ExtentMode::Absolute(Extent3D::new_2d(1280, 720)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
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
