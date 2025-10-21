# wgpu Push Constants Implementation Guide

## Current Status
wgpu backend has a stub for `push_constants()` that does nothing.

## The Problem
wgpu doesn't support push constants like Vulkan. We need to use a different approach.

## Solution: Use a Staging Buffer + Immediate Buffer Update

Since wgpu supports `write_buffer()` on the queue, we can update a uniform buffer immediately before drawing.

## Implementation

### Step 1: Add Transform Buffer to WgpuBackend

```rust
// In src/backends/wgpu_backend/mod.rs
pub struct WgpuBackend {
    // ... existing fields ...
    
    // Transform buffer for push constant emulation
    transform_buffer: Option<wgpu::Buffer>,
    transform_bind_group: Option<wgpu::BindGroup>,
}
```

### Step 2: Create Transform Buffer During Init

```rust
impl WgpuBackend {
    pub fn new(enable_validation: bool) -> Result<Self> {
        // ... existing initialization ...
        
        Ok(Self {
            // ... existing fields ...
            transform_buffer: None,
            transform_bind_group: None,
        })
    }
    
    fn create_transform_buffer(&mut self) -> Result<()> {
        let device = self.device.as_ref().context("Device not initialized")?;
        
        // Create buffer for model + normal matrices (128 bytes)
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Transform Uniform Buffer"),
            size: 128,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Create bind group layout for transforms
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Transform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Transform Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
            ],
        });
        
        self.transform_buffer = Some(buffer);
        self.transform_bind_group = Some(bind_group);
        
        Ok(())
    }
}
```

### Step 3: Implement push_constants in WgpuPassContext

```rust
// Add field to WgpuPassContext
struct WgpuPassContext {
    render_pass: *mut (),
    backend: *mut WgpuBackend,
    pending_transform_data: Option<Vec<u8>>, // Store data until draw
}

impl PassExecutionContext for WgpuPassContext {
    fn push_constants(
        &mut self,
        _stage_flags: u32,
        offset: u32,
        data: &[u8],
    ) -> Result<()> {
        log::debug!(
            "WgpuPassContext: Buffering {} bytes at offset {} for next draw",
            data.len(),
            offset
        );
        
        // Store the data - we'll upload it before the next draw call
        if self.pending_transform_data.is_none() {
            self.pending_transform_data = Some(vec![0u8; 128]);
        }
        
        if let Some(buffer) = &mut self.pending_transform_data {
            let start = offset as usize;
            let end = start + data.len();
            buffer[start..end].copy_from_slice(data);
        }
        
        Ok(())
    }
    
    fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) -> Result<()> {
        // Upload transform data before drawing
        if let Some(data) = &self.pending_transform_data {
            let backend = self.backend();
            if let (Some(queue), Some(buffer)) = (&backend.queue, &backend.transform_buffer) {
                queue.write_buffer(buffer, 0, data);
            }
            
            // Bind the transform buffer
            if let Some(bind_group) = &backend.transform_bind_group {
                self.render_pass().set_bind_group(2, bind_group, &[]); // Set 2 for transforms
            }
        }
        
        // Now do the actual draw
        self.render_pass().draw(
            first_vertex..(first_vertex + vertex_count),
            first_instance..(first_instance + instance_count),
        );
        
        log::debug!("WgpuPassContext: Draw call completed");
        Ok(())
    }
}
```

### Step 4: Update WGSL Shader

wgpu needs WGSL shaders, not GLSL. Create `shaders/forward.wgsl`:

```wgsl
// Vertex shader
struct CameraUniforms {
    view_proj: mat4x4<f32>,
};

struct PushConstants {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(2) @binding(0) var<uniform> push: PushConstants;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_pos = push.model * vec4<f32>(in.position, 1.0);
    out.world_pos = world_pos.xyz;
    out.position = camera.view_proj * world_pos;
    out.normal = (push.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz;
    out.uv = in.uv;
    out.color = in.color;
    
    return out;
}

// Fragment shader would go here...
```

## Estimated Time
- Implementation: 1-2 hours
- Testing: 30 minutes
- Total: 1.5-2.5 hours

## Testing
```bash
# wgpu should work on Linux
cargo run -- --scene scenes/cube.toml --pipeline forward --backend wgpu
```

## Alternative: Simpler Approach

If you want wgpu working faster, you could:
1. Skip transforms entirely for now (render at origin)
2. Just get the lighting working
3. Add transforms later

This would only take ~30 minutes.

---

**Note:** This is a complete implementation guide. The actual code would need to be integrated carefully with error handling and proper initialization order.
