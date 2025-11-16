# Graphics API Coordinate Systems

Different graphics APIs use different coordinate system conventions. This document explains these differences and how we handle them in Rusty Renderer.

## NDC (Normalized Device Coordinates) Systems

### Vulkan
- **X-axis**: -1 (left) to +1 (right)
- **Y-axis**: -1 (top) to +1 (bottom) - **Y points DOWN**
- **Z-axis**: 0 (near) to 1 (far)

### DirectX 12
- **X-axis**: -1 (left) to +1 (right)
- **Y-axis**: -1 (bottom) to +1 (top) - **Y points UP**
- **Z-axis**: 0 (near) to 1 (far)

### wgpu (WebGPU)
- **X-axis**: -1 (left) to +1 (right)
- **Y-axis**: -1 (bottom) to +1 (top) - **Y points UP**
- **Z-axis**: 0 (near) to 1 (far)

## Key Differences

The main difference is the **Y-axis direction**:
- **Vulkan**: Y points DOWN (top = -1, bottom = +1)
- **DirectX 12**: Y points UP (top = +1, bottom = -1)
- **wgpu**: Y points UP (top = +1, bottom = -1)

## Our Solution

We handle the Y-axis difference **at the projection matrix level** in `src/camera/mod.rs`:

### Projection Matrix Approach

For Vulkan, we negate the Y-axis component of the projection matrix to account for its inverted NDC Y-axis:

```rust
pub fn perspective_projection(fov_degrees: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4 {
    let base_proj = Mat4::perspective_rh(fov_degrees.to_radians(), aspect_ratio, near, far);
    
    match get_camera_backend() {
        CameraBackend::Vulkan => {
            // Vulkan NDC: Y goes from -1 (top) to +1 (bottom) - INVERTED
            // Negate Y-axis in projection matrix to compensate
            let mut proj = base_proj;
            proj.y_axis *= -1.0;
            proj
        }
        CameraBackend::DirectX => {
            // DirectX NDC: Y goes from -1 (bottom) to +1 (top) - STANDARD
            base_proj
        }
    }
}
```

This approach:
- **Keeps shaders identical** between backends (both use HLSL compiled to SPIR-V/DXIL)
- **Handles the flip transparently** in the camera system
- **Works correctly** for all camera types (perspective, orthographic, etc.)
- **Is the standard solution** used by major engines

### Legacy: Shader-level Flipping

Previous versions flipped Y coordinates in shaders for specific backends:

### Triangle Example

**Vulkan shader** (`shaders/triangle.vert`):
```glsl
vec2 positions[3] = vec2[](
    vec2(0.0, -0.5),  // Bottom center (Y = -0.5, renders at bottom)
    vec2(0.5, 0.5),   // Top right (Y = 0.5, renders at top)
    vec2(-0.5, 0.5)   // Top left (Y = 0.5, renders at top)
);
```

**DirectX shader** (`shaders/hlsl/triangle.hlsl`):
```hlsl
float2 positions[3] = {
    float2(0.0, 0.5),    // Bottom center (flipped from -0.5)
    float2(0.5, -0.5),   // Top right (flipped from 0.5)
    float2(-0.5, -0.5)   // Top left (flipped from 0.5)
};
```

**wgpu shader** (`shaders/wgsl/triangle.wgsl`):
```wgsl
var positions = array<vec2<f32>, 3>(
    vec2<f32>(0.0, 0.5),    // Bottom center (flipped from -0.5)
    vec2<f32>(0.5, -0.5),   // Top right (flipped from 0.5)
    vec2<f32>(-0.5, -0.5)   // Top left (flipped from 0.5)
);
```

## Implementation Notes

1. **Shader-level flipping**: We flip Y coordinates in the vertex shader for DirectX and wgpu backends. This is the most efficient approach as it happens once per vertex.

2. **Consistent output**: All backends render the same visual output with red at the bottom center, green at top right, and blue at top left.

3. **Future camera systems**: When implementing camera/projection matrices, we'll need to account for these differences in the projection matrix construction.

4. **Documentation**: Each shader file includes comments explaining the coordinate system and any transformations applied.

## Testing

To verify coordinate system consistency:

```bash
# Test Vulkan (reference)
cargo run --release -- --backend vulkan --max-frames 5

# Test DirectX (on Windows or with Proton)
cargo run --release -- --backend directx --max-frames 5

# Test wgpu (cross-platform)
cargo run --release -- --backend wgpu --max-frames 5
```

All three should render the same triangle orientation.

## References

- [Vulkan Coordinate Systems](https://www.khronos.org/registry/vulkan/specs/1.3/html/chap24.html#vertexpostproc-clipping)
- [DirectX Coordinate Systems](https://docs.microsoft.com/en-us/windows/win32/direct3d9/coordinate-systems)
- [WebGPU Coordinate Systems](https://gpuweb.github.io/gpuweb/#coordinate-systems)
