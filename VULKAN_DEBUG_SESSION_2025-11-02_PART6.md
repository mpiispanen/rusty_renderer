# Vulkan Cube Rendering Debug Session - 2025-11-02

## Issue
Vulkan backend was rendering only a clear color (black screen) when trying to render the cube scene, despite:
- Vertex data being correctly loaded (36 vertices, 1728 bytes)
- Draw calls being issued successfully  
- No Vulkan validation errors
- Triangle scene working correctly

## Root Cause
**Matrix layout mismatch between CPU and GPU**

The application was building view and projection matrices in **row-major** format (standard for C/C++/Rust), but HLSL shaders use **column-major** matrix layout by default. This caused incorrect transformations that placed all geometry behind the camera.

### Evidence
- ViewProj matrix row 3 showed `[0.0, 0.0, -0.1001, 0.0]` with w=0 instead of the expected w=1.0
- Test calculation showed vertex at `[-0.5, -0.5, 0.5]` transformed to clip space with **negative w** (-0.05)
- Negative w means the vertex is behind the camera and gets clipped

## Solution
Added matrix transpose before uploading to GPU:

```rust
// src/app.rs
fn transpose_mat4(m: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    [
        [m[0][0], m[1][0], m[2][0], m[3][0]],
        [m[0][1], m[1][1], m[2][1], m[3][1]],
        [m[0][2], m[1][2], m[2][2], m[3][2]],
        [m[0][3], m[1][3], m[2][3], m[3][3]],
    ]
}

// In build_render_graph:
let view_proj_transposed = Self::transpose_mat4(view_proj);
let camera_uniforms = CameraUniforms { view_proj: view_proj_transposed };
```

## Changes Made
1. Added `transpose_mat4()` helper function
2. Modified camera uniform creation to transpose the viewProj matrix before upload
3. Added debug logging to print first 3 vertices of geometry
4. Fixed clippy warning in vulkan/mod.rs (map_or -> is_some_and)

## Verification
- ✅ Cube scene now renders correctly  
- ✅ Triangle scene still works
- ✅ No Vulkan validation errors (only performance warning about unused UV attribute)
- ✅ `cargo fmt` passes
- ✅ `cargo clippy --release -- -D warnings` passes

## Technical Notes

### HLSL Matrix Layout
- HLSL uses **column-major** layout by default
- In HLSL, `mul(M, v)` treats M as column-major
- CPU code built matrices in row-major (standard for Rust/C++)
- Solution: Transpose before upload OR use `row_major` annotation in HLSL

### Why Triangle Scene Worked
- Triangle pass uses hardcoded vertex positions passed directly to shader
- No matrix transformations involved, so layout mismatch didn't affect it

## Related Files
- `src/app.rs` - Matrix transpose and camera setup
- `src/backends/vulkan/mod.rs` - Clippy fix
- `shaders/hlsl/forward_simple.hlsl` - Shader using matrices
- `scenes/cube.toml` - Cube geometry definition

## Next Steps
1. Continue migration to render graph resource management
2. Apply same transpose fix to model matrices in push constants (already transposed implicitly)
3. Consider documenting matrix layout conventions in architecture docs
4. Test DirectX backend to ensure it also handles matrices correctly
