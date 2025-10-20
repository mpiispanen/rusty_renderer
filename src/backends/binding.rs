//! Shader resource binding abstractions
//!
//! This module provides backend-agnostic abstractions for binding shader resources
//! (uniform buffers, textures, samplers) to rendering pipelines.

use crate::backends::{Buffer, Sampler, Texture};
use std::sync::Arc;

/// Shader stage where a binding is visible
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShaderStage(u32);

impl ShaderStage {
    pub const VERTEX: u32 = 1 << 0;
    pub const FRAGMENT: u32 = 1 << 1;
    pub const COMPUTE: u32 = 1 << 2;
    pub const ALL_GRAPHICS: u32 = Self::VERTEX | Self::FRAGMENT;
    pub const ALL: u32 = Self::VERTEX | Self::FRAGMENT | Self::COMPUTE;

    pub const fn new(flags: u32) -> Self {
        Self(flags)
    }

    pub const fn bits(&self) -> u32 {
        self.0
    }

    pub const fn contains(&self, other: u32) -> bool {
        (self.0 & other) == other
    }
}

/// Texture dimension for binding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureDimension {
    D1,
    D2,
    D3,
    Cube,
    D2Array,
}

/// Type of shader binding
#[derive(Debug, Clone)]
pub enum ShaderBinding {
    /// Uniform buffer binding
    UniformBuffer {
        binding: u32,
        size: u64,
        stage: ShaderStage,
    },
    /// Storage buffer binding (for compute shaders)
    StorageBuffer {
        binding: u32,
        size: u64,
        stage: ShaderStage,
        readonly: bool,
    },
    /// Texture binding
    Texture {
        binding: u32,
        dimension: TextureDimension,
        stage: ShaderStage,
    },
    /// Sampler binding
    Sampler { binding: u32, stage: ShaderStage },
}

impl ShaderBinding {
    /// Get the binding index
    pub fn binding(&self) -> u32 {
        match self {
            Self::UniformBuffer { binding, .. }
            | Self::StorageBuffer { binding, .. }
            | Self::Texture { binding, .. }
            | Self::Sampler { binding, .. } => *binding,
        }
    }

    /// Get the shader stage
    pub fn stage(&self) -> ShaderStage {
        match self {
            Self::UniformBuffer { stage, .. }
            | Self::StorageBuffer { stage, .. }
            | Self::Texture { stage, .. }
            | Self::Sampler { stage, .. } => *stage,
        }
    }
}

/// Layout describing a set of shader bindings
#[derive(Debug, Clone)]
pub struct BindGroupLayout {
    bindings: Vec<ShaderBinding>,
}

impl BindGroupLayout {
    /// Create a new bind group layout
    pub fn new(bindings: Vec<ShaderBinding>) -> Self {
        Self { bindings }
    }

    /// Get the bindings
    pub fn bindings(&self) -> &[ShaderBinding] {
        &self.bindings
    }

    /// Get number of bindings
    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    /// Find a binding by index
    pub fn get_binding(&self, binding: u32) -> Option<&ShaderBinding> {
        self.bindings.iter().find(|b| b.binding() == binding)
    }
}

/// Resource bound to a shader binding
#[derive(Clone)]
pub enum BoundResource {
    UniformBuffer(Arc<dyn Buffer>),
    StorageBuffer(Arc<dyn Buffer>),
    Texture(Arc<dyn Texture>),
    Sampler(Arc<dyn Sampler>),
}

/// A set of bound resources matching a layout
pub struct BindGroup {
    layout: BindGroupLayout,
    resources: Vec<(u32, BoundResource)>,
}

impl BindGroup {
    /// Create a new bind group
    pub fn new(layout: BindGroupLayout) -> Self {
        Self {
            layout,
            resources: Vec::new(),
        }
    }

    /// Get the layout
    pub fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }

    /// Bind a uniform buffer
    pub fn bind_uniform_buffer(&mut self, binding: u32, buffer: Arc<dyn Buffer>) {
        self.resources
            .push((binding, BoundResource::UniformBuffer(buffer)));
    }

    /// Bind a storage buffer
    pub fn bind_storage_buffer(&mut self, binding: u32, buffer: Arc<dyn Buffer>) {
        self.resources
            .push((binding, BoundResource::StorageBuffer(buffer)));
    }

    /// Bind a texture
    pub fn bind_texture(&mut self, binding: u32, texture: Arc<dyn Texture>) {
        self.resources
            .push((binding, BoundResource::Texture(texture)));
    }

    /// Bind a sampler
    pub fn bind_sampler(&mut self, binding: u32, sampler: Arc<dyn Sampler>) {
        self.resources
            .push((binding, BoundResource::Sampler(sampler)));
    }

    /// Get all bound resources
    pub fn resources(&self) -> &[(u32, BoundResource)] {
        &self.resources
    }

    /// Get a specific bound resource
    pub fn get_resource(&self, binding: u32) -> Option<&BoundResource> {
        self.resources
            .iter()
            .find(|(b, _)| *b == binding)
            .map(|(_, r)| r)
    }
}

/// Builder for bind group layouts
pub struct BindGroupLayoutBuilder {
    bindings: Vec<ShaderBinding>,
}

impl BindGroupLayoutBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Add a uniform buffer binding
    pub fn uniform_buffer(mut self, binding: u32, size: u64, stage: ShaderStage) -> Self {
        self.bindings.push(ShaderBinding::UniformBuffer {
            binding,
            size,
            stage,
        });
        self
    }

    /// Add a storage buffer binding
    pub fn storage_buffer(
        mut self,
        binding: u32,
        size: u64,
        stage: ShaderStage,
        readonly: bool,
    ) -> Self {
        self.bindings.push(ShaderBinding::StorageBuffer {
            binding,
            size,
            stage,
            readonly,
        });
        self
    }

    /// Add a texture binding
    pub fn texture(
        mut self,
        binding: u32,
        dimension: TextureDimension,
        stage: ShaderStage,
    ) -> Self {
        self.bindings.push(ShaderBinding::Texture {
            binding,
            dimension,
            stage,
        });
        self
    }

    /// Add a sampler binding
    pub fn sampler(mut self, binding: u32, stage: ShaderStage) -> Self {
        self.bindings
            .push(ShaderBinding::Sampler { binding, stage });
        self
    }

    /// Build the layout
    pub fn build(self) -> BindGroupLayout {
        BindGroupLayout::new(self.bindings)
    }
}

impl Default for BindGroupLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_stage_flags() {
        let vertex = ShaderStage::new(ShaderStage::VERTEX);
        assert!(vertex.contains(ShaderStage::VERTEX));
        assert!(!vertex.contains(ShaderStage::FRAGMENT));

        let all_graphics = ShaderStage::new(ShaderStage::ALL_GRAPHICS);
        assert!(all_graphics.contains(ShaderStage::VERTEX));
        assert!(all_graphics.contains(ShaderStage::FRAGMENT));
    }

    #[test]
    fn test_bind_group_layout_creation() {
        let layout = BindGroupLayoutBuilder::new()
            .uniform_buffer(0, 64, ShaderStage::new(ShaderStage::VERTEX))
            .texture(
                1,
                TextureDimension::D2,
                ShaderStage::new(ShaderStage::FRAGMENT),
            )
            .sampler(2, ShaderStage::new(ShaderStage::FRAGMENT))
            .build();

        assert_eq!(layout.binding_count(), 3);
        assert!(layout.get_binding(0).is_some());
        assert!(layout.get_binding(1).is_some());
        assert!(layout.get_binding(2).is_some());
        assert!(layout.get_binding(3).is_none());
    }

    #[test]
    fn test_bind_group_creation() {
        let layout = BindGroupLayoutBuilder::new()
            .uniform_buffer(0, 64, ShaderStage::new(ShaderStage::VERTEX))
            .build();

        let bind_group = BindGroup::new(layout);
        assert_eq!(bind_group.layout().binding_count(), 1);
    }

    #[test]
    fn test_shader_binding_methods() {
        let binding = ShaderBinding::UniformBuffer {
            binding: 5,
            size: 256,
            stage: ShaderStage::new(ShaderStage::VERTEX),
        };

        assert_eq!(binding.binding(), 5);
        assert_eq!(binding.stage().bits(), ShaderStage::VERTEX);
    }
}
