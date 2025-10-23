# wgpu Backend Solutions - Making It Work

**Goal:** Implement texture support in wgpu while keeping the same backend trait

---

## Solution 1: Store Bind Groups in Context ⭐ RECOMMENDED

### Approach
Store created bind groups in `WgpuPassContext` to keep them alive for the render pass duration.

### Implementation

```rust
struct WgpuPassContext {
    render_pass: *mut (),
    backend: *mut WgpuBackend,
    uniform_buffers: Vec<(*const std::ffi::c_void, u32, u64, u64)>,
    texture_bindings: Vec<(*const std::ffi::c_void, u32, u32)>,
    push_constant_data: Vec<u8>,
    
    // NEW: Store created bind groups to keep them alive
    bind_groups: Vec<wgpu::BindGroup>,
    // NEW: Store temporary buffers (for push constants)
    temp_buffers: Vec<wgpu::Buffer>,
}

impl WgpuPassContext {
    fn draw(&mut self, ...) -> Result<()> {
        let backend = unsafe { &*self.backend };
        let device = backend.device.as_ref().unwrap();
        
        if self.uniform_buffers.len() >= 2 {
            // Create entries for group 0
            let mut entries = Vec::new();
            
            // Add uniform buffers
            for (ptr, binding, offset, size) in &self.uniform_buffers {
                let buffer_ref = unsafe { &*(*ptr as *const WgpuBuffer) };
                entries.push(wgpu::BindGroupEntry {
                    binding: *binding,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer_ref.buffer,
                        offset: *offset,
                        size: std::num::NonZeroU64::new(*size),
                    }),
                });
            }
            
            // Add textures
            for (tex_ptr, _set, binding) in &self.texture_bindings {
                let texture = unsafe { &*(*tex_ptr as *const WgpuTexture) };
                entries.push(wgpu::BindGroupEntry {
                    binding: *binding,
                    resource: wgpu::BindingResource::TextureView(texture.view()),
                });
            }
            
            // Add sampler
            if !self.texture_bindings.is_empty() {
                if let Some(ref sampler) = backend.default_sampler {
                    entries.push(wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    });
                }
            }
            
            // Create bind group 0 and STORE IT
            let bind_group_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Set 0: Uniforms + Textures"),
                layout: &backend.bind_group_layouts[0],
                entries: &entries,
            });
            
            // Bind it
            self.render_pass().set_bind_group(0, &bind_group_0, &[]);
            
            // STORE IT to keep it alive
            self.bind_groups.push(bind_group_0);
            
            // Create bind group 1 for transform
            if !self.push_constant_data.is_empty() {
                let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Transform Buffer"),
                    contents: &self.push_constant_data,
                    usage: wgpu::BufferUsages::UNIFORM,
                });
                
                let bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Set 1: Transform"),
                    layout: &backend.bind_group_layouts[1],
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: transform_buffer.as_entire_binding(),
                    }],
                });
                
                self.render_pass().set_bind_group(1, &bind_group_1, &[]);
                
                // STORE BOTH to keep them alive
                self.temp_buffers.push(transform_buffer);
                self.bind_groups.push(bind_group_1);
            }
        }
        
        // Now draw - bind groups are still alive!
        self.render_pass().draw(first_vertex..(first_vertex + vertex_count),
                               first_instance..(first_instance + instance_count));
        Ok(())
    }
}
```

### Pros
✅ Minimal changes to architecture  
✅ No unsafe code needed (beyond existing)  
✅ Same trait interface  
✅ Simple to understand  
✅ Works with existing pattern  

### Cons
⚠️ Slightly more memory per context (a few KB)  
⚠️ Bind groups created per draw (but cached in vec)  

### Effort
**Low** - ~30 minutes implementation

---

## Solution 2: Two-Phase Execution

### Approach
Split pass execution into "prepare" and "execute" phases.

### Implementation

```rust
// New trait method (optional, defaults to no-op)
pub trait PassCallback: Send + Sync {
    fn execute(&self, context: &mut dyn PassExecutionContext);
    
    // NEW: Optional prepare phase
    fn prepare(&self, context: &mut dyn PassPreparationContext) {
        // Default: do nothing
    }
}

pub trait PassPreparationContext {
    // Collect resources needed
    fn collect_uniforms(&mut self, ...);
    fn collect_textures(&mut self, ...);
}

// In wgpu backend:
struct WgpuBackend {
    // NEW: Pre-created bind groups
    prepared_bind_groups: HashMap<PassId, Vec<wgpu::BindGroup>>,
}

impl GraphicsBackend for WgpuBackend {
    fn execute_graph(&mut self, graph: &RenderGraph) -> Result<()> {
        // PHASE 1: Prepare all bind groups before render pass
        for pass_id in &execution_order {
            if let Some(pass) = graph.get_pass(pass_id) {
                if let Some(callback) = &pass.callback {
                    let mut prep_context = WgpuPrepContext::new(self);
                    callback.prepare(&mut prep_context);
                    
                    // Create bind groups NOW (outside render pass)
                    let bind_groups = prep_context.create_bind_groups();
                    self.prepared_bind_groups.insert(pass_id, bind_groups);
                }
            }
        }
        
        // PHASE 2: Execute render pass with pre-created bind groups
        for pass_id in execution_order {
            let bind_groups = self.prepared_bind_groups.get(&pass_id).unwrap();
            let mut context = WgpuPassContext::new(&mut render_pass, bind_groups);
            callback.execute(&mut context);
        }
        
        Ok(())
    }
}
```

### Pros
✅ Clean separation of concerns  
✅ Bind groups created optimally  
✅ Could enable caching/reuse  

### Cons
⚠️ New trait method (backward compat needed)  
⚠️ More complex architecture  
⚠️ Passes need to implement prepare()  

### Effort
**Medium** - ~2-3 hours implementation

---

## Solution 3: Backend-Specific Pass Type

### Approach
Create wgpu-specific pass types that handle binding internally.

### Implementation

```rust
// Keep PassExecutionContext as-is, but don't use it for bind groups in wgpu

// wgpu-specific pass
pub struct WgpuForwardPass {
    vertex_buffer: Arc<WgpuBuffer>,
    camera_buffer: Arc<WgpuBuffer>,
    lighting_buffer: Arc<WgpuBuffer>,
    material_buffer: Option<Arc<WgpuBuffer>>,
    texture: Option<Arc<WgpuTexture>>,
    transform: Mat4,
    vertex_count: u32,
    
    // Pre-created bind groups (created in new())
    bind_group_0: Option<wgpu::BindGroup>,
    bind_group_1: Option<wgpu::BindGroup>,
}

impl WgpuForwardPass {
    pub fn new(
        backend: &mut WgpuBackend,
        vertex_buffer: Arc<WgpuBuffer>,
        camera_buffer: Arc<WgpuBuffer>,
        // ... other params
    ) -> Self {
        // Create bind groups HERE (outside render pass)
        let bind_group_0 = backend.device.create_bind_group(...);
        let bind_group_1 = backend.device.create_bind_group(...);
        
        Self {
            vertex_buffer,
            // ... other fields
            bind_group_0: Some(bind_group_0),
            bind_group_1: Some(bind_group_1),
        }
    }
}

impl PassCallback for WgpuForwardPass {
    fn execute(&self, context: &mut dyn PassExecutionContext) {
        // Downcast to wgpu context
        let wgpu_ctx = context.as_any_mut()
            .downcast_mut::<WgpuPassContext>()
            .unwrap();
        
        // Use pre-created bind groups
        wgpu_ctx.render_pass().set_bind_group(0, self.bind_group_0.as_ref().unwrap(), &[]);
        wgpu_ctx.render_pass().set_bind_group(1, self.bind_group_1.as_ref().unwrap(), &[]);
        
        // Bind vertex buffer
        wgpu_ctx.bind_vertex_buffer(...);
        
        // Draw
        wgpu_ctx.draw(...);
    }
}
```

### Pros
✅ Full control over wgpu specifics  
✅ Optimal bind group management  
✅ No changes to trait  

### Cons
⚠️ Duplicate pass implementations per backend  
⚠️ More code to maintain  
⚠️ Less abstraction  

### Effort
**Medium** - ~2-3 hours per pass type

---

## Comparison Matrix

| Solution | Effort | Maintainability | Performance | Abstraction |
|----------|--------|-----------------|-------------|-------------|
| **1. Store in Context** | ⭐⭐⭐ Low | ⭐⭐⭐ Good | ⭐⭐ Good | ⭐⭐⭐ High |
| **2. Two-Phase** | ⭐⭐ Medium | ⭐⭐ Fair | ⭐⭐⭐ Best | ⭐⭐ Medium |
| **3. Backend-Specific** | ⭐⭐ Medium | ⭐ Poor | ⭐⭐⭐ Best | ⭐ Low |

---

## Recommendation: Solution 1 (Store in Context)

**Why:**
1. **Minimal changes** - Add 2 fields to WgpuPassContext
2. **Simple** - Easy to understand and debug
3. **Works immediately** - No architectural changes
4. **Same trait** - No breaking changes
5. **Good enough performance** - Bind groups are cheap to create

**Implementation Steps:**

1. Add fields to `WgpuPassContext`:
   ```rust
   bind_groups: Vec<wgpu::BindGroup>,
   temp_buffers: Vec<wgpu::Buffer>,
   ```

2. In `draw()`, push bind groups to the vec:
   ```rust
   self.bind_groups.push(bind_group_0);
   ```

3. Add `DeviceExt` import for `create_buffer_init`

4. Update WGSL shader to use group 1 for transforms

5. Test!

**Time to implement:** 30-60 minutes  
**Risk:** Low  
**Benefit:** Full wgpu texture support ✅

---

## Alternative: Solution 2 for Long-Term

If we want optimal performance and plan to add many features, Solution 2 (Two-Phase) is better long-term:

- Cleaner architecture
- Better for future features (shadows, post-processing)
- Could enable bind group caching
- More "rusty" with explicit phases

But requires more refactoring and affects all backends.

---

## Conclusion

**Start with Solution 1** to get wgpu textures working quickly.  
**Consider Solution 2** later if we need better performance or add more complex rendering features.

Both solutions keep the same backend trait interface! 🎯
