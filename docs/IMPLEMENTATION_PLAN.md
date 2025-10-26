# Implementation Plan - Next Steps

**Created:** 2025-10-21  
**Updated:** 2025-10-26  
**Status:** Active Plan

This document outlines concrete implementation tasks for the short-term roadmap.

---

## Phase 1: Backend Parity (1 Week)

### Task 1.1: DirectX Rendering Fixes (~3-4 hours)

**Goal:** Fix DirectX rendering to match Vulkan output

**Steps:**
1. Implement depth testing
   - Create depth/stencil buffer
   - Configure depth state in pipeline
   - Clear depth buffer each frame

2. Fix backface culling
   - Verify cull mode settings
   - Check winding order consistency
   - Test with textured cube

3. Add texture support
   - Implement descriptor tables
   - Upload textures to GPU
   - Bind textures in render pass

**Acceptance Criteria:**
- [ ] DirectX renders identical to Vulkan
- [ ] Depth testing working
- [ ] Backface culling correct
- [ ] Textures display properly

**Files to Modify:**
- `src/backends/directx/dx12_impl.rs`
- `src/backends/directx/mod.rs`
- `shaders/forward.hlsl` (new)
- Update pipeline/root signature creation

---

### Task 1.3: Shader Conversion Pipeline (~3-4 hours)

**Goal:** HLSL as source, auto-convert to other formats

**Architecture:**
```
forward.hlsl (source of truth)
    ├─> forward.wgsl (manual port initially, auto later)
    └─> forward.spv (compile with DXC to SPIR-V for Vulkan)
```

**Steps:**
1. Install shader compilers
   ```bash
   # DXC for HLSL -> SPIR-V
   # spirv-cross for SPIR-V -> WGSL (future)
   ```

2. Create build script
   ```rust
   // build.rs
   fn compile_shaders() {
       compile_hlsl_to_spirv("shaders/forward.hlsl", "forward.vert.spv");
       // Manual WGSL port for now
   }
   ```

3. Update backends to load appropriate format
   - Vulkan: Load .spv
   - wgpu: Load .wgsl
   - DirectX: Load .hlsl or compiled bytecode

**Acceptance Criteria:**
- [ ] HLSL shaders compile to SPIR-V
- [ ] All backends use correct shader format
- [ ] Shader changes propagate to all backends
- [ ] Hot-reload working (future)

---

## Phase 2: Resource Management Refactor (1-2 Weeks)

### Task 2.1: Pass Requirement System (~2-3 days)

**Goal:** Passes declare their requirements, render graph provisions them

**Design:**
```rust
pub trait RenderPassRequirements {
    fn shaders(&self) -> Vec<ShaderRequirement>;
    fn buffers(&self) -> Vec<BufferRequirement>;
    fn textures(&self) -> Vec<TextureRequirement>;
    fn pipeline_state(&self) -> PipelineState;
}

pub struct ShaderRequirement {
    pub stage: ShaderStage,
    pub path: PathBuf,
    pub entry_point: String,
}

pub struct BufferRequirement {
    pub name: String,
    pub size: usize,
    pub usage: BufferUsage,
    pub update_frequency: UpdateFrequency, // PerFrame, PerDraw, Static
}
```

**Implementation:**
1. Define requirement traits/structs
2. Implement for ForwardPass
3. Add requirement validation
4. Document pattern for new passes

**Acceptance Criteria:**
- [ ] ForwardPass declares all requirements
- [ ] Requirements validated at graph build time
- [ ] Clear error messages for missing requirements
- [ ] Documentation for pass authors

---

### Task 2.2: Automatic Resource Allocation (~3-4 days)

**Goal:** Render graph creates resources based on requirements

**Architecture:**
```rust
impl RenderGraph {
    pub fn allocate_resources(&mut self, backend: &mut dyn GraphicsBackend) {
        for pass in &self.passes {
            // Create shaders
            for shader_req in pass.shaders() {
                let shader = self.shader_cache.get_or_load(shader_req)?;
                pass.attach_shader(shader);
            }
            
            // Create buffers
            for buffer_req in pass.buffers() {
                let buffer = backend.create_buffer(&buffer_req.into())?;
                pass.attach_buffer(buffer_req.name, buffer);
            }
            
            // Create pipelines
            let pipeline = backend.create_pipeline(pass.pipeline_state())?;
            pass.attach_pipeline(pipeline);
        }
    }
}
```

**Steps:**
1. Add shader cache system
2. Add resource allocation logic to render graph
3. Update pass interface to receive allocated resources
4. Handle resource lifetimes correctly

**Acceptance Criteria:**
- [ ] Resources allocated automatically
- [ ] Proper cleanup on graph destruction
- [ ] Resources shared when possible
- [ ] Memory usage tracked and logged

---

### Task 2.3: Shader Hot-Reload (~2-3 days)

**Goal:** Detect shader file changes and recompile/reload

**Implementation:**
```rust
use notify::Watcher;

struct ShaderCache {
    watcher: RecommendedWatcher,
    shaders: HashMap<PathBuf, CompiledShader>,
    dirty: Arc<Mutex<HashSet<PathBuf>>>,
}

impl ShaderCache {
    pub fn check_for_changes(&mut self) -> Vec<PathBuf> {
        let mut dirty = self.dirty.lock().unwrap();
        let changed = dirty.drain().collect();
        changed
    }
    
    pub fn reload(&mut self, path: &Path) -> Result<CompiledShader> {
        compile_shader(path)
    }
}
```

**Steps:**
1. Add notify crate dependency
2. Implement file watching
3. Add recompilation logic
4. Update pipelines with new shaders
5. Handle errors gracefully (keep old shader on failure)

**Acceptance Criteria:**
- [ ] File changes detected within 1 second
- [ ] Shaders recompiled automatically
- [ ] Pipelines updated in next frame
- [ ] Error feedback shown to user

---

## Phase 3: Shadow Mapping (2 Weeks)

### Task 3.1: Depth-Only Render Pass (~2 days)

**Goal:** Basic shadow map generation

**Implementation:**
```rust
pub struct ShadowMapPass {
    resolution: u32,
    depth_texture: ResourceId,
    light_view_proj: Mat4,
}

impl RenderPass for ShadowMapPass {
    fn execute(&self, ctx: &mut dyn PassExecutionContext) {
        // Render scene from light's POV
        // Write only depth
        // No color output
    }
}
```

**Steps:**
1. Create depth texture resource
2. Implement depth-only rendering
3. Calculate light view-projection matrix
4. Test with simple scene

**Acceptance Criteria:**
- [ ] Depth texture created correctly
- [ ] Scene rendered from light POV
- [ ] Depth values stored properly
- [ ] Can visualize depth map

---

### Task 3.2: PCF Shadow Filtering (~2-3 days)

**Goal:** Soft shadows via Percentage Closer Filtering

**Shader:**
```glsl
float shadowPCF(sampler2D shadowMap, vec4 shadowCoord, int kernelSize) {
    float shadow = 0.0;
    vec2 texelSize = 1.0 / textureSize(shadowMap, 0);
    
    for(int x = -kernelSize; x <= kernelSize; ++x) {
        for(int y = -kernelSize; y <= kernelSize; ++y) {
            vec2 offset = vec2(x, y) * texelSize;
            float depth = texture(shadowMap, shadowCoord.xy + offset).r;
            shadow += shadowCoord.z > depth ? 0.0 : 1.0;
        }
    }
    
    int samples = (kernelSize * 2 + 1) * (kernelSize * 2 + 1);
    return shadow / samples;
}
```

**Steps:**
1. Add shadow map sampling to forward shader
2. Implement PCF filtering
3. Add configurable kernel size
4. Test with different scene configurations

---

### Task 3.3: Cascaded Shadow Maps (~4-5 days)

**Goal:** High-quality shadows for large scenes

**Architecture:**
```rust
pub struct CSMPass {
    cascade_count: usize, // 2-4 cascades
    resolutions: Vec<u32>, // e.g., [4096, 2048, 1024, 512]
    split_distances: Vec<f32>, // Camera frustum splits
    cascades: Vec<ShadowCascade>,
}

struct ShadowCascade {
    depth_texture: ResourceId,
    light_view_proj: Mat4,
    split_near: f32,
    split_far: f32,
}
```

**Steps:**
1. Calculate frustum splits
2. Render each cascade
3. Select cascade in shader based on depth
4. Blend between cascades
5. Debug visualization

**Acceptance Criteria:**
- [ ] Multiple cascades rendering
- [ ] Smooth cascade transitions
- [ ] Configurable cascade count
- [ ] Debug view showing cascade splits

---

## Phase 4: Forward+ and Deferred (3 Weeks)

### Task 4.1: Forward+ Light Culling (~1 week)

**Compute Shader:**
```glsl
layout(local_size_x = 16, local_size_y = 16) in;

void main() {
    ivec2 tileID = ivec2(gl_WorkGroupID.xy);
    
    // Calculate tile frustum
    Frustum tileFrustum = calculateTileFrustum(tileID);
    
    // Test each light against frustum
    for (int i = 0; i < lightCount; i++) {
        if (intersects(lights[i], tileFrustum)) {
            addLightToTile(tileID, i);
        }
    }
}
```

**Steps:**
1. Implement compute pass for light culling
2. Create tile light lists buffer
3. Update forward shader to use tile data
4. Benchmark vs regular forward

---

### Task 4.2: Deferred G-Buffer (~1 week)

**G-Buffer Layout:**
```
RT0: RGB = Albedo,     A = Metallic
RT1: RGB = Normal,     A = Roughness  
RT2: RGB = Position,   A = AO
RT3: RGB = Emission,   A = Unused
```

**Steps:**
1. Design G-buffer layout
2. Implement geometry pass
3. Implement lighting pass
4. Test with simple scene

---

## Timeline Summary

| Week | Phase | Tasks |
|------|-------|-------|
| 1 | Multi-Backend | wgpu, DirectX, shader pipeline |
| 2-3 | Resource Mgmt | Requirements, allocation, hot-reload |
| 4-5 | Shadows | Basic, PCF, CSM |
| 6-8 | Forward+/Deferred | Light culling, G-buffer, lighting |

---

## Success Metrics

After completing this plan:
- [ ] All 3 backends working identically
- [ ] 0 hardcoded resources
- [ ] Hot-reload working for shaders
- [ ] 3 shadow techniques implemented
- [ ] Forward, Forward+, Deferred renderers working
- [ ] Can switch techniques at runtime
- [ ] Performance metrics collected

---

**Next Review:** Weekly progress check-ins
