# Coordinate System Differences Across Graphics APIs

## Overview

Different graphics APIs use different coordinate system conventions. This document describes these differences and how Rusty Renderer handles them to ensure consistent output across all backends.

## NDC (Normalized Device Coordinates)

### X-Axis (Consistent)
All backends use the same X-axis convention:
- `-1.0` = Left edge
- `+1.0` = Right edge

### Y-Axis (Different!)

| Backend | Y-Axis Direction | Top | Bottom | Origin Style |
|---------|------------------|-----|--------|--------------|
| **Vulkan** | DOWN | `-1.0` | `+1.0` | DirectX-style |
| **DirectX 12** | DOWN | `-1.0` | `+1.0` | DirectX-style |
| **wgpu/WebGPU** | UP | `+1.0` | `-1.0` | OpenGL-style |
| **OpenGL** | UP | `+1.0` | `-1.0` | OpenGL-style |

### Z-Axis (Depth)

| Backend | Near | Far | Range |
|---------|------|-----|-------|
| **Vulkan** | `0.0` | `1.0` | `[0, 1]` |
| **DirectX 12** | `0.0` | `1.0` | `[0, 1]` |
| **wgpu/WebGPU** | `0.0` | `1.0` | `[0, 1]` |
| **OpenGL** | `-1.0` | `+1.0` | `[-1, 1]` |

## Window/Screen Coordinates

### Origin

| Backend | Origin | Y-Direction |
|---------|--------|-------------|
| **Vulkan** | Top-Left | DOWN |
| **DirectX 12** | Top-Left | DOWN |
| **wgpu/WebGPU** | Top-Left | DOWN |

All backends use top-left origin for window/screen space (pixel coordinates).

## Impact on Implementation

### 1. Vertex Shaders

When hardcoding vertices, Y coordinates must be adjusted per backend:

```glsl
// Vulkan GLSL - Y points DOWN
vec2 positions[3] = vec2[](
    vec2(0.0, -0.5),  // Bottom center
    vec2(0.5, 0.5),   // Top right
    vec2(-0.5, 0.5)   // Top left
);
```

```wgsl
// wgpu WGSL - Y points UP (flip Y!)
var positions = array<vec2<f32>, 3>(
    vec2<f32>(0.0, 0.5),    // Bottom center (flipped)
    vec2<f32>(0.5, -0.5),   // Top right (flipped)
    vec2<f32>(-0.5, -0.5)   // Top left (flipped)
);
```

### 2. Projection Matrices

Projection matrices must account for Y-axis differences:

**Vulkan/DirectX:**
```rust
let projection = Mat4::perspective_lh(fov, aspect, near, far);
```

**wgpu/WebGPU:**
```rust
let projection = Mat4::perspective_rh(fov, aspect, near, far);
// Or flip Y in the projection matrix
projection[1][1] *= -1.0;
```

### 3. Texture Coordinates

UV coordinates are typically consistent (origin at top-left), but sampling may differ.

## Rusty Renderer's Approach

### Current Implementation (M4)

For the triangle example, we handle Y-flip in shaders:
- Each backend's shader is adjusted to produce identical output
- Vulkan: Y values as-is
- wgpu: Y values flipped

### Future Implementation (M5+)

When we add a render graph and vertex buffers, we'll implement:

1. **Backend Capabilities Enum:**
```rust
pub struct BackendCapabilities {
    pub y_axis_up: bool,  // true for wgpu/OpenGL, false for Vulkan/DX
    pub depth_range_zero_to_one: bool,  // true for most modern APIs
    pub clip_space_origin: ClipSpaceOrigin,
}
```

2. **Automatic Coordinate Transformation:**
```rust
impl Pipeline {
    fn adjust_for_backend(&mut self, caps: &BackendCapabilities) {
        if caps.y_axis_up {
            // Automatically flip Y in vertex shader or projection matrix
            self.flip_y_axis = true;
        }
    }
}
```

3. **Unified Vertex Format:**
- Application provides vertices in a canonical format (e.g., Y-up)
- Engine automatically transforms for each backend

## Best Practices

### For Application Developers

1. **Use engine-provided coordinate system** - Don't worry about backend differences
2. **Provide assets in canonical format** - Engine handles conversion
3. **Use engine's projection matrix helpers** - Automatically correct

### For Engine Developers

1. **Document assumptions** - Make coordinate system clear in comments
2. **Test cross-backend** - Ensure identical output across backends
3. **Centralize transformations** - Don't scatter Y-flip logic everywhere
4. **Use backend capabilities** - Query rather than hard-code

## Testing Cross-Backend Consistency

### Visual Verification
```bash
# Render same scene with different backends
rusty_renderer --backend vulkan --scene test
rusty_renderer --backend wgpu --scene test
rusty_renderer --backend directx --scene test

# Compare outputs visually or via screenshots
```

### Automated Testing
```rust
#[test]
fn test_coordinate_consistency() {
    // Render triangle with each backend
    // Compare vertex positions in screen space
    // Ensure identical output
}
```

## References

- Vulkan Specification: https://registry.khronos.org/vulkan/specs/1.3/html/
- WebGPU Specification: https://www.w3.org/TR/webgpu/
- DirectX 12 Documentation: https://docs.microsoft.com/en-us/windows/win32/direct3d12/
- "Coordinate Systems in Graphics" (Real-Time Rendering, 4th Ed.)

## Related Files

- `shaders/triangle.vert` - Vulkan vertex shader
- `shaders/wgsl/triangle.wgsl` - wgpu vertex shader (Y-flipped)
- `src/backends/mod.rs` - Backend traits (future: add capabilities)
- `docs/DESIGN.md` - Overall architecture

---

**Last Updated:** 2025-10-16  
**Milestone:** M4 (Multi-Backend Triangle)
