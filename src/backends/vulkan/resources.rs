//! Vulkan resource implementations (buffers, textures, samplers)

use crate::backends::resources::*;
use anyhow::{Context, Result};
use std::any::Any;
use vulkanalia::prelude::v1_0::*;

/// Vulkan buffer implementation
pub struct VulkanBuffer {
    pub(super) buffer: vk::Buffer,
    pub(super) memory: vk::DeviceMemory,
    size: u64,
    usage: BufferUsage,
    memory_location: MemoryLocation,
    device: Device,
}

impl VulkanBuffer {
    /// Create a new Vulkan buffer
    pub fn new(
        device: Device,
        desc: &BufferDescriptor,
        physical_device: vk::PhysicalDevice,
        instance: &Instance,
    ) -> Result<Self> {
        let mut usage_flags = vk::BufferUsageFlags::empty();

        // Convert our usage flags to Vulkan flags
        if desc.usage.vertex {
            usage_flags |= vk::BufferUsageFlags::VERTEX_BUFFER;
        }
        if desc.usage.index {
            usage_flags |= vk::BufferUsageFlags::INDEX_BUFFER;
        }
        if desc.usage.uniform {
            usage_flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
        }
        if desc.usage.storage {
            usage_flags |= vk::BufferUsageFlags::STORAGE_BUFFER;
        }
        if desc.usage.transfer_src {
            usage_flags |= vk::BufferUsageFlags::TRANSFER_SRC;
        }
        if desc.usage.transfer_dst {
            usage_flags |= vk::BufferUsageFlags::TRANSFER_DST;
        }

        // Create buffer
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(desc.size)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };

        // Get memory requirements
        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        // Find suitable memory type
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let memory_type_index = find_memory_type(
            mem_requirements.memory_type_bits,
            memory_properties,
            &desc.memory_location,
        )
        .context("Failed to find suitable memory type for buffer")?;

        // Allocate memory
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unsafe {
            device
                .allocate_memory(&alloc_info, None)
                .context("Failed to allocate buffer memory")?
        };

        // Bind memory to buffer
        unsafe {
            device
                .bind_buffer_memory(buffer, memory, 0)
                .context("Failed to bind buffer memory")?;
        }

        log::debug!(
            "Created Vulkan buffer: {} bytes, usage: {:?}, type_index: {}",
            desc.size,
            desc.usage,
            memory_type_index
        );

        Ok(Self {
            buffer,
            memory,
            size: desc.size,
            usage: desc.usage,
            memory_location: desc.memory_location,
            device,
        })
    }

    /// Get the underlying Vulkan buffer handle
    pub fn vk_buffer(&self) -> vk::Buffer {
        self.buffer
    }

    /// Alias for vk_buffer (for consistency with other handle() methods)
    pub fn handle(&self) -> vk::Buffer {
        self.buffer
    }
}

impl crate::backends::Buffer for VulkanBuffer {
    fn size(&self) -> u64 {
        self.size
    }

    fn usage(&self) -> BufferUsage {
        self.usage
    }

    fn memory_location(&self) -> MemoryLocation {
        self.memory_location
    }

    fn map(&mut self) -> Result<&mut [u8]> {
        // Only CPU-visible memory can be mapped
        match self.memory_location {
            MemoryLocation::GpuOnly => {
                anyhow::bail!("Cannot map GPU-only buffer. Use staging buffer for uploads.")
            }
            MemoryLocation::CpuToGpu | MemoryLocation::GpuToCpu => {
                // Note: For production use, consider keeping buffers persistently mapped
                // This is a simplified implementation that maps on each call
                anyhow::bail!("map() not yet implemented - use upload_to_buffer() instead")
            }
        }
    }

    fn unmap(&mut self) {
        // Note: With persistent mapping, we would unmap here
        // For now, this is a no-op since we don't support map()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Drop for VulkanBuffer {
    fn drop(&mut self) {
        log::info!(
            "Dropping Vulkan buffer: {} bytes, handle {:?}",
            self.size,
            self.buffer
        );
        log::debug!("Drop backtrace: {}", std::backtrace::Backtrace::capture());
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
        log::trace!("Destroyed Vulkan buffer: {} bytes", self.size);
    }
}

/// Vulkan texture implementation
pub struct VulkanTexture {
    pub(super) image: vk::Image,
    pub(super) image_view: vk::ImageView,
    pub(super) memory: vk::DeviceMemory,
    width: u32,
    height: u32,
    format: TextureFormat,
    usage: TextureUsage,
    mip_levels: u32,
    device: Device,
}

impl VulkanTexture {
    /// Create a new Vulkan texture
    pub fn new(
        device: Device,
        desc: &TextureDescriptor,
        physical_device: vk::PhysicalDevice,
        instance: &Instance,
    ) -> Result<Self> {
        let vk_format = texture_format_to_vk(desc.format)?;

        // Convert usage flags
        let mut usage_flags = vk::ImageUsageFlags::empty();
        if desc.usage.sampled {
            usage_flags |= vk::ImageUsageFlags::SAMPLED;
        }
        if desc.usage.render_target {
            usage_flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
        }
        if desc.usage.depth_stencil {
            usage_flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
        }
        if desc.usage.transfer_src {
            usage_flags |= vk::ImageUsageFlags::TRANSFER_SRC;
        }
        if desc.usage.transfer_dst {
            usage_flags |= vk::ImageUsageFlags::TRANSFER_DST;
        }

        // Create image
        let image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::_2D)
            .extent(vk::Extent3D {
                width: desc.width,
                height: desc.height,
                depth: 1,
            })
            .mip_levels(desc.mip_levels)
            .array_layers(1)
            .format(vk_format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::_1);

        let image = unsafe { device.create_image(&image_info, None)? };

        // Get memory requirements
        let mem_requirements = unsafe { device.get_image_memory_requirements(image) };

        // Find suitable memory type (GPU-only for images)
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        let memory_type_index = find_memory_type(
            mem_requirements.memory_type_bits,
            memory_properties,
            &MemoryLocation::GpuOnly,
        )
        .context("Failed to find suitable memory type for texture")?;

        // Allocate memory
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unsafe {
            device
                .allocate_memory(&alloc_info, None)
                .context("Failed to allocate texture memory")?
        };

        // Bind memory to image
        unsafe {
            device
                .bind_image_memory(image, memory, 0)
                .context("Failed to bind texture memory")?;
        }

        // Create image view
        let aspect_mask = if desc.format.is_depth() {
            vk::ImageAspectFlags::DEPTH
        } else {
            vk::ImageAspectFlags::COLOR
        };

        let view_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::_2D)
            .format(vk_format)
            .subresource_range(
                vk::ImageSubresourceRange::builder()
                    .aspect_mask(aspect_mask)
                    .base_mip_level(0)
                    .level_count(desc.mip_levels)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            );

        let image_view = unsafe { device.create_image_view(&view_info, None)? };

        log::debug!(
            "Created Vulkan texture: {}x{}, format: {:?}, mips: {}",
            desc.width,
            desc.height,
            desc.format,
            desc.mip_levels
        );

        Ok(Self {
            image,
            image_view,
            memory,
            width: desc.width,
            height: desc.height,
            format: desc.format,
            usage: desc.usage,
            mip_levels: desc.mip_levels,
            device,
        })
    }

    /// Get the underlying Vulkan image handle
    pub fn vk_image(&self) -> vk::Image {
        self.image
    }

    /// Get the underlying Vulkan image view handle
    #[allow(dead_code)] // Will be used in M8.2+ for descriptor binding
    pub fn vk_image_view(&self) -> vk::ImageView {
        self.image_view
    }
}

impl crate::backends::Texture for VulkanTexture {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn format(&self) -> TextureFormat {
        self.format
    }

    fn usage(&self) -> TextureUsage {
        self.usage
    }

    fn mip_levels(&self) -> u32 {
        self.mip_levels
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Drop for VulkanTexture {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image_view(self.image_view, None);
            self.device.destroy_image(self.image, None);
            self.device.free_memory(self.memory, None);
        }
        log::trace!("Destroyed Vulkan texture: {}x{}", self.width, self.height);
    }
}

/// Vulkan sampler implementation
pub struct VulkanSampler {
    sampler: vk::Sampler,
    device: Device,
}

impl VulkanSampler {
    /// Create a new Vulkan sampler
    pub fn new(device: Device, desc: &SamplerDescriptor) -> Result<Self> {
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(filter_mode_to_vk(desc.mag_filter))
            .min_filter(filter_mode_to_vk(desc.min_filter))
            .mipmap_mode(mipmap_filter_to_vk(desc.mipmap_filter))
            .address_mode_u(address_mode_to_vk(desc.address_mode_u))
            .address_mode_v(address_mode_to_vk(desc.address_mode_v))
            .address_mode_w(address_mode_to_vk(desc.address_mode_w))
            .mip_lod_bias(0.0)
            .anisotropy_enable(false)
            .max_anisotropy(1.0)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .min_lod(0.0)
            .max_lod(vk::LOD_CLAMP_NONE)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false);

        let sampler = unsafe { device.create_sampler(&sampler_info, None)? };

        log::debug!(
            "Created Vulkan sampler: mag={:?}, min={:?}",
            desc.mag_filter,
            desc.min_filter
        );

        Ok(Self { sampler, device })
    }

    /// Get the underlying Vulkan sampler handle
    #[allow(dead_code)] // Will be used in M8.2+ for descriptor binding
    pub fn vk_sampler(&self) -> vk::Sampler {
        self.sampler
    }
}

impl crate::backends::Sampler for VulkanSampler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl Drop for VulkanSampler {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_sampler(self.sampler, None);
        }
        log::trace!("Destroyed Vulkan sampler");
    }
}

// Helper functions

/// Find a suitable memory type based on requirements and location preference
fn find_memory_type(
    type_filter: u32,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    location: &MemoryLocation,
) -> Option<u32> {
    // Determine required property flags based on memory location
    let required_flags = match location {
        MemoryLocation::GpuOnly => vk::MemoryPropertyFlags::DEVICE_LOCAL,
        MemoryLocation::CpuToGpu => {
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
        }
        MemoryLocation::GpuToCpu => {
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_CACHED
        }
    };

    // Try to find exact match first
    for i in 0..memory_properties.memory_type_count {
        let is_required_type = (type_filter & (1 << i)) != 0;
        let has_required_properties = memory_properties.memory_types[i as usize]
            .property_flags
            .contains(required_flags);

        if is_required_type && has_required_properties {
            return Some(i);
        }
    }

    // Fallback: for CPU-visible memory, try without CACHED flag
    if matches!(location, MemoryLocation::GpuToCpu) {
        let fallback_flags =
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;

        for i in 0..memory_properties.memory_type_count {
            let is_required_type = (type_filter & (1 << i)) != 0;
            let has_required_properties = memory_properties.memory_types[i as usize]
                .property_flags
                .contains(fallback_flags);

            if is_required_type && has_required_properties {
                return Some(i);
            }
        }
    }

    None
}

/// Convert TextureFormat to Vulkan format
fn texture_format_to_vk(format: TextureFormat) -> Result<vk::Format> {
    Ok(match format {
        TextureFormat::Rgba8Srgb => vk::Format::R8G8B8A8_SRGB,
        TextureFormat::Rgba8Unorm => vk::Format::R8G8B8A8_UNORM,
        TextureFormat::Bgra8Srgb => vk::Format::B8G8R8A8_SRGB,
        TextureFormat::Bgra8Unorm => vk::Format::B8G8R8A8_UNORM,
        TextureFormat::Depth32Float => vk::Format::D32_SFLOAT,
        TextureFormat::Depth24PlusStencil8 => vk::Format::D24_UNORM_S8_UINT,
    })
}

/// Convert FilterMode to Vulkan filter
fn filter_mode_to_vk(mode: FilterMode) -> vk::Filter {
    match mode {
        FilterMode::Nearest => vk::Filter::NEAREST,
        FilterMode::Linear => vk::Filter::LINEAR,
    }
}

/// Convert FilterMode to Vulkan mipmap mode
fn mipmap_filter_to_vk(mode: FilterMode) -> vk::SamplerMipmapMode {
    match mode {
        FilterMode::Nearest => vk::SamplerMipmapMode::NEAREST,
        FilterMode::Linear => vk::SamplerMipmapMode::LINEAR,
    }
}

/// Convert AddressMode to Vulkan address mode
fn address_mode_to_vk(mode: AddressMode) -> vk::SamplerAddressMode {
    match mode {
        AddressMode::ClampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
        AddressMode::Repeat => vk::SamplerAddressMode::REPEAT,
        AddressMode::MirrorRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
    }
}
