//! Vulkan descriptor set management
//!
//! This module handles descriptor sets, descriptor pools, and descriptor set layouts
//! for binding shader resources (uniform buffers, textures, samplers).

use crate::backends::binding::{
    BindGroup, BindGroupLayout, BoundResource, ShaderBinding, ShaderStage,
};
use anyhow::{Context, Result};
use vulkanalia::prelude::v1_0::*;

/// Manages descriptor pools and allocation
pub struct DescriptorPoolManager {
    device: Device,
    pools: Vec<vk::DescriptorPool>,
    current_pool_index: usize,
}

impl DescriptorPoolManager {
    /// Create a new descriptor pool manager
    pub fn new(device: Device) -> Self {
        Self {
            device,
            pools: Vec::new(),
            current_pool_index: 0,
        }
    }

    /// Create a new descriptor pool
    fn create_pool(&self) -> Result<vk::DescriptorPool> {
        // Pool sizes for different descriptor types
        let pool_sizes = [
            // Uniform buffers
            vk::DescriptorPoolSize::builder()
                .type_(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(100)
                .build(),
            // Storage buffers
            vk::DescriptorPoolSize::builder()
                .type_(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(100)
                .build(),
            // Sampled images
            vk::DescriptorPoolSize::builder()
                .type_(vk::DescriptorType::SAMPLED_IMAGE)
                .descriptor_count(100)
                .build(),
            // Samplers
            vk::DescriptorPoolSize::builder()
                .type_(vk::DescriptorType::SAMPLER)
                .descriptor_count(100)
                .build(),
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(100)
            .pool_sizes(&pool_sizes)
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET);

        unsafe {
            self.device
                .create_descriptor_pool(&pool_info, None)
                .context("Failed to create descriptor pool")
        }
    }

    /// Allocate a descriptor set from the pool
    pub fn allocate(&mut self, layout: vk::DescriptorSetLayout) -> Result<vk::DescriptorSet> {
        // Ensure we have at least one pool
        if self.pools.is_empty() {
            let pool = self.create_pool()?;
            self.pools.push(pool);
        }

        let layouts = [layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pools[self.current_pool_index])
            .set_layouts(&layouts);

        match unsafe { self.device.allocate_descriptor_sets(&alloc_info) } {
            Ok(sets) => Ok(sets[0]),
            Err(_) => {
                // Pool is full, create a new one
                let pool = self.create_pool()?;
                self.pools.push(pool);
                self.current_pool_index = self.pools.len() - 1;

                let alloc_info = vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(self.pools[self.current_pool_index])
                    .set_layouts(&layouts);

                unsafe {
                    self.device
                        .allocate_descriptor_sets(&alloc_info)
                        .context("Failed to allocate descriptor set")
                        .map(|sets| sets[0])
                }
            }
        }
    }

    /// Destroy all pools
    pub fn destroy(&mut self) {
        unsafe {
            for pool in &self.pools {
                self.device.destroy_descriptor_pool(*pool, None);
            }
        }
        self.pools.clear();
    }
}

/// Convert ShaderStage to Vulkan shader stage flags
fn shader_stage_to_vk(stage: ShaderStage) -> vk::ShaderStageFlags {
    let mut flags = vk::ShaderStageFlags::empty();

    if stage.contains(ShaderStage::VERTEX) {
        flags |= vk::ShaderStageFlags::VERTEX;
    }
    if stage.contains(ShaderStage::FRAGMENT) {
        flags |= vk::ShaderStageFlags::FRAGMENT;
    }
    if stage.contains(ShaderStage::COMPUTE) {
        flags |= vk::ShaderStageFlags::COMPUTE;
    }

    flags
}

/// Create a Vulkan descriptor set layout from a bind group layout
pub fn create_descriptor_set_layout(
    device: &Device,
    layout: &BindGroupLayout,
) -> Result<vk::DescriptorSetLayout> {
    let mut bindings = Vec::new();

    for binding in layout.bindings() {
        let descriptor_type = match binding {
            ShaderBinding::UniformBuffer { .. } => vk::DescriptorType::UNIFORM_BUFFER,
            ShaderBinding::StorageBuffer { .. } => {
                // In Vulkan, both readonly and writable storage buffers use STORAGE_BUFFER
                vk::DescriptorType::STORAGE_BUFFER
            }
            ShaderBinding::Texture { .. } => vk::DescriptorType::SAMPLED_IMAGE,
            ShaderBinding::Sampler { .. } => vk::DescriptorType::SAMPLER,
        };

        let vk_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(binding.binding())
            .descriptor_type(descriptor_type)
            .descriptor_count(1)
            .stage_flags(shader_stage_to_vk(binding.stage()))
            .build();

        bindings.push(vk_binding);
    }

    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

    unsafe {
        device
            .create_descriptor_set_layout(&layout_info, None)
            .context("Failed to create descriptor set layout")
    }
}

/// Update a descriptor set with bound resources
pub fn update_descriptor_set(
    device: &Device,
    descriptor_set: vk::DescriptorSet,
    bind_group: &BindGroup,
) -> Result<()> {
    let mut writes: Vec<vk::WriteDescriptorSet> = Vec::new();
    let mut buffer_infos: Vec<vk::DescriptorBufferInfo> = Vec::new();
    let mut _image_infos: Vec<vk::DescriptorImageInfo> = Vec::new(); // For M8.4

    for (binding, resource) in bind_group.resources() {
        match resource {
            BoundResource::UniformBuffer(buffer) | BoundResource::StorageBuffer(buffer) => {
                // Downcast to get Vulkan buffer handle
                let vk_buffer = buffer
                    .as_any()
                    .downcast_ref::<crate::backends::vulkan::resources::VulkanBuffer>()
                    .context("Failed to downcast to VulkanBuffer")?;

                let descriptor_type = match resource {
                    BoundResource::UniformBuffer(_) => vk::DescriptorType::UNIFORM_BUFFER,
                    BoundResource::StorageBuffer(_) => vk::DescriptorType::STORAGE_BUFFER,
                    _ => unreachable!(),
                };

                let buffer_info = vk::DescriptorBufferInfo::builder()
                    .buffer(vk_buffer.handle())
                    .offset(0)
                    .range(vk::WHOLE_SIZE)
                    .build();

                buffer_infos.push(buffer_info);

                let write = vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set)
                    .dst_binding(*binding)
                    .dst_array_element(0)
                    .descriptor_type(descriptor_type)
                    .buffer_info(std::slice::from_ref(buffer_infos.last().unwrap()))
                    .build();

                writes.push(write);
            }
            BoundResource::Texture(_texture) => {
                // TODO: Implement texture binding in M8.4
                log::warn!("Texture binding not yet implemented");
            }
            BoundResource::Sampler(_sampler) => {
                // TODO: Implement sampler binding in M8.4
                log::warn!("Sampler binding not yet implemented");
            }
        }
    }

    if !writes.is_empty() {
        unsafe {
            let copies: &[vk::CopyDescriptorSet] = &[];
            device.update_descriptor_sets(&writes, copies);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_stage_conversion() {
        let vertex = ShaderStage::new(ShaderStage::VERTEX);
        let vk_vertex = shader_stage_to_vk(vertex);
        assert!(vk_vertex.contains(vk::ShaderStageFlags::VERTEX));
        assert!(!vk_vertex.contains(vk::ShaderStageFlags::FRAGMENT));

        let all_graphics = ShaderStage::new(ShaderStage::ALL_GRAPHICS);
        let vk_all = shader_stage_to_vk(all_graphics);
        assert!(vk_all.contains(vk::ShaderStageFlags::VERTEX));
        assert!(vk_all.contains(vk::ShaderStageFlags::FRAGMENT));
    }
}
