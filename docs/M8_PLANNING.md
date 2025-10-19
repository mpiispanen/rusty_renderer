# Milestone 8: Real Rendering Pipeline

**Goal**: Transform from hardcoded placeholders to a production-ready renderer capable of loading and rendering glTF models with proper vertex/index buffers, shaders, and textures.

**Status**: Planning  
**Created**: 2025-10-19  
**Target Completion**: TBD

## Vision

Move from proof-of-concept triangle rendering with hardcoded geometry to a complete rendering pipeline that can load real 3D models, use vertex/index buffers, bind textures, and render complex scenes across all backends without any placeholder or workaround code.

## Current State Analysis

### What We Have (Placeholders)
- ✅ Render graph architecture
- ✅ Backend abstraction working
- ✅ Triangle renders on all backends
- ❌ **Hardcoded triangle vertices in shaders**
- ❌ **No vertex/index buffer support**
- ❌ **No texture loading or binding**
- ❌ **No shader resource binding (descriptors/bind groups)**
- ❌ **No model loading (glTF)**
- ❌ **No material system**
- ❌ **Placeholder pass callbacks**

### What We Need

**Resource Management**:
- Vertex buffer creation and upload
- Index buffer creation and upload
- Texture loading (from files)
- Texture creation on GPU
- Staging buffers for uploads
- Memory allocation strategies

**Shader System**:
- Real shaders with uniform/descriptor bindings
- Shader compilation for all backends (SPIR-V, DXIL, WGSL)
- Descriptor/bind group layouts
- Push constants / root constants
- Vertex input descriptions

**Model Loading**:
- glTF 2.0 parser
- Mesh extraction (positions, normals, UVs, indices)
- Material data extraction
- Texture references
- Scene hierarchy (basic)

**Rendering Pipeline**:
- Vertex/index buffer binding
- Texture binding and sampling
- Descriptor set management (Vulkan)
- Bind group management (wgpu)
- Root signature setup (DirectX)
- Draw indexed commands

## Milestone Structure

### M8.1: Resource Management Foundation
**Goal**: Create buffer and texture abstractions that work across all backends

**Deliverables**:
- Buffer trait and implementations (Vulkan, DX12, wgpu)
- Buffer creation with usage flags (vertex, index, uniform, staging)
- Memory allocation and mapping
- Texture trait and implementations
- Texture creation with formats and dimensions
- Staging buffer upload system
- Resource lifetime management

**Acceptance Criteria**:
- Can create vertex buffer on all backends
- Can upload data to buffers
- Can create 2D textures from memory
- Memory is properly managed (no leaks)
- Tests pass on all platforms

### M8.2: Vertex/Index Buffer Rendering
**Goal**: Render geometry using proper vertex/index buffers instead of hardcoded data

**Deliverables**:
- Vertex format definition (position, normal, UV, color)
- Vertex buffer binding in render passes
- Index buffer binding
- Draw indexed API
- Update triangle demo to use buffers
- Shader updates to read from vertex buffers

**Acceptance Criteria**:
- Triangle renders using vertex buffer data
- Can render indexed geometry (cube, quad)
- Works on all three backends
- No hardcoded vertices in shaders
- Visual regression tests pass

### M8.3: Shader Resource Binding
**Goal**: Implement descriptor sets/bind groups for uniform buffers and textures

**Deliverables**:
- Descriptor set layout (Vulkan)
- Bind group layout (wgpu)
- Root signature (DirectX 12)
- Uniform buffer support
- Texture binding support
- Sampler objects
- Update shaders with bindings

**Acceptance Criteria**:
- Can bind uniform buffers
- Can bind textures and sample them
- MVP matrix support working
- Texture rendering on quad works
- All backends consistent

### M8.4: Texture Loading and Sampling
**Goal**: Load textures from files and render textured geometry

**Deliverables**:
- Image loading (PNG, JPG via `image` crate)
- Texture upload pipeline (staging → GPU)
- Sampler creation with filtering modes
- Texture binding in render passes
- Textured quad demo
- MIP map generation (optional)

**Acceptance Criteria**:
- Can load texture from file
- Can render textured quad
- Filtering works (linear, nearest)
- Works on all backends
- No texture corruption

### M8.5: glTF Model Loading
**Goal**: Load and parse glTF 2.0 models

**Deliverables**:
- glTF parser integration (use `gltf` crate)
- Mesh extraction (primitives, attributes)
- Material data extraction
- Texture reference resolution
- Scene structure (basic node hierarchy)
- Buffer data extraction

**Acceptance Criteria**:
- Can load simple glTF model
- Extract vertices, normals, UVs, indices
- Extract material parameters
- Extract texture references
- No crashes on malformed files

### M8.6: Material System
**Goal**: Implement basic PBR material support

**Deliverables**:
- Material trait definition
- PBR material parameters (baseColor, metallic, roughness)
- Material → shader binding
- Multiple materials per model
- Texture slots (albedo, normal, metallic/roughness)

**Acceptance Criteria**:
- Can render model with multiple materials
- Texture-based materials work
- Color-based materials work
- Material parameters affect rendering

### M8.7: Complete glTF Rendering
**Goal**: Render complete glTF models with textures and materials

**Deliverables**:
- Complete rendering pipeline
- Model → GPU upload
- Per-mesh rendering
- Texture binding per material
- Transform hierarchies (basic)
- Camera integration

**Acceptance Criteria**:
- Can load and render glTF model (e.g., DamagedHelmet)
- Textures display correctly
- Multiple meshes render correctly
- Works on all three backends
- No placeholder code remains
- Visual regression tests with reference images

## Technical Design

### Buffer Management

```rust
pub trait Buffer {
    fn size(&self) -> u64;
    fn usage(&self) -> BufferUsage;
    fn map(&mut self) -> Result<&mut [u8]>;
    fn unmap(&mut self);
    fn as_any(&self) -> &dyn Any;
}

pub struct BufferDescriptor {
    pub size: u64,
    pub usage: BufferUsage,
    pub memory_location: MemoryLocation,
}

impl GraphicsBackend {
    fn create_buffer(&mut self, desc: &BufferDescriptor) -> Result<Box<dyn Buffer>>;
    fn upload_to_buffer(&mut self, buffer: &dyn Buffer, data: &[u8], offset: u64) -> Result<()>;
}
```

### Texture Management

```rust
pub trait Texture {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn format(&self) -> TextureFormat;
    fn as_any(&self) -> &dyn Any;
}

pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub mip_levels: u32,
}

impl GraphicsBackend {
    fn create_texture(&mut self, desc: &TextureDescriptor) -> Result<Box<dyn Texture>>;
    fn upload_to_texture(&mut self, texture: &dyn Texture, data: &[u8]) -> Result<()>;
    fn create_sampler(&mut self, desc: &SamplerDescriptor) -> Result<Box<dyn Sampler>>;
}
```

### Descriptor/Bind Group Layout

```rust
pub struct BindingDescriptor {
    pub binding: u32,
    pub ty: BindingType,
    pub stage: ShaderStage,
}

pub enum BindingType {
    UniformBuffer,
    StorageBuffer,
    Texture,
    Sampler,
}

pub trait DescriptorSet {
    fn bind_buffer(&mut self, binding: u32, buffer: &dyn Buffer);
    fn bind_texture(&mut self, binding: u32, texture: &dyn Texture);
    fn bind_sampler(&mut self, binding: u32, sampler: &dyn Sampler);
}
```

### Model Loading

```rust
pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub textures: Vec<LoadedTexture>,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material_index: usize,
}

pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

pub struct Material {
    pub base_color: [f32; 4],
    pub base_color_texture: Option<usize>,
    pub metallic: f32,
    pub roughness: f32,
    pub metallic_roughness_texture: Option<usize>,
}

pub fn load_gltf(path: &Path) -> Result<Model>;
```

## Dependencies

**New Crates**:
- `gltf` - glTF 2.0 parsing
- Already have: `image` - Texture loading

**Existing Code to Enhance**:
- `backends/` - Add buffer/texture methods
- `render_graph/` - Add resource types
- `passes/` - Update with real rendering
- `shaders/` - Replace hardcoded geometry

## Risks and Challenges

1. **Descriptor Set Complexity**: Each backend has different descriptor management
2. **Memory Management**: Need proper allocation strategies
3. **Shader Compilation**: Need to compile to 3 different formats
4. **Synchronization**: Proper barriers for texture uploads
5. **Testing**: Hard to validate correctness across backends

## Success Criteria

### Functional
- ✅ Load glTF model from file
- ✅ Render with textures
- ✅ Multiple materials work
- ✅ All backends produce identical results
- ✅ No hardcoded geometry or workarounds
- ✅ Clean, maintainable code

### Quality
- ✅ Comprehensive tests for each component
- ✅ Visual regression tests with reference images
- ✅ Memory leak detection
- ✅ Performance profiling data
- ✅ Documentation for all public APIs

### Technical Debt
- ❌ No placeholder code
- ❌ No TODO comments in critical paths
- ❌ No backend-specific workarounds
- ❌ No hardcoded values

## Timeline Estimate

| Phase | Estimated Time | Complexity |
|-------|----------------|------------|
| M8.1: Resource Foundation | 3-4 days | High |
| M8.2: Vertex/Index Buffers | 2-3 days | Medium |
| M8.3: Descriptor Binding | 4-5 days | Very High |
| M8.4: Texture Loading | 2-3 days | Medium |
| M8.5: glTF Loading | 2-3 days | Medium |
| M8.6: Material System | 3-4 days | High |
| M8.7: Complete Pipeline | 3-4 days | High |
| **Total** | **19-26 days** | |

## Out of Scope (Future Milestones)

- Advanced PBR features (IBL, BRDF)
- Normal mapping
- Shadow mapping
- Multiple lights
- Animation support
- Complex scene hierarchies
- LOD systems
- Instancing

These will be addressed in M9 and beyond once the foundation is solid.

## References

- [glTF 2.0 Specification](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html)
- [Vulkan Descriptor Sets](https://vkguide.dev/docs/chapter-4/descriptors/)
- [DirectX 12 Root Signatures](https://learn.microsoft.com/en-us/windows/win32/direct3d12/root-signatures)
- [wgpu Bind Groups](https://sotrh.github.io/learn-wgpu/beginner/tutorial5-textures/)
