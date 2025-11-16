# Coordinate System Fix: Vulkan/DirectX Y-Axis Parity

## Problem

DirectX and Vulkan backends were rendering with different Y-axis orientations, resulting in models appearing upside-down or mirrored between backends. This was visible when comparing helmet and cube renders.

## Root Cause

**Vulkan and DirectX have different NDC (Normalized Device Coordinates) Y-axis conventions:**

### Vulkan NDC
- X: -1 (left) to +1 (right)
- **Y: -1 (top) to +1 (bottom)** ← INVERTED!
- Z: 0 (near) to 1 (far)

### DirectX 12 NDC
- X: -1 (left) to +1 (right)
- **Y: -1 (bottom) to +1 (top)** ← STANDARD
- Z: 0 (near) to 1 (far)

The renderer was using the same projection matrix for both backends, which caused Vulkan to render with an inverted Y-axis compared to DirectX.

## Solution

Applied Y-axis negation in the projection matrix for Vulkan only, in `src/camera/mod.rs`:

```rust
pub fn perspective_projection(fov_degrees: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4 {
    let base_proj = Mat4::perspective_rh(fov_degrees.to_radians(), aspect_ratio, near, far);
    
    match get_camera_backend() {
        CameraBackend::Vulkan => {
            // Vulkan NDC: Y goes from -1 (top) to +1 (bottom) - INVERTED
            // We need to flip the Y-axis in the projection matrix to account for this
            let mut proj = base_proj;
            proj.y_axis *= -1.0;  // ← KEY FIX
            proj
        }
        CameraBackend::DirectX => {
            // DirectX NDC: Y goes from -1 (bottom) to +1 (top) - STANDARD
            base_proj
        }
    }
}
```

## Why This Approach

1. **Standard Practice**: This is the canonical solution used by Unity, Unreal, and other major engines
2. **Shader Compatibility**: Keeps shaders identical between backends (both use HLSL)
3. **Transparent**: Happens automatically in the camera system
4. **Performance**: Single matrix multiplication, no per-vertex overhead
5. **Correct**: Works for all projection types (perspective, orthographic, etc.)

## Alternative Approaches Considered

1. **Viewport Flip**: Vulkan supports negative viewport height, but this affects winding order and requires shader adjustments
2. **Shader-level Flip**: Would require different shaders per backend, defeating the purpose of unified HLSL
3. **Geometry Flip**: Would break lighting normals and texture coordinates

## Verification

The fix was tested with:
- Cube scene (geometric shapes show orientation clearly)
- Damaged Helmet model (complex GLTF asset with textures)
- Both headless and windowed modes
- Both native Linux and Proton/Wine environments

### Test Results

All tests pass with identical visual output between backends:

```bash
./test_coordinate_parity.sh
```

Output:
- `test_cube_comparison.png`: Shows identical cube orientation in VK and DX
- `test_helmet_comparison.png`: Shows identical helmet orientation in VK and DX

## Impact

- ✅ Vulkan and DirectX now render with identical Y-axis orientation
- ✅ No shader changes required
- ✅ No performance impact
- ✅ Both headless and windowed modes work correctly
- ✅ Camera system handles the transformation transparently

## Related Files

- `src/camera/mod.rs` - Projection matrix calculation with Y-axis fix
- `docs/COORDINATE_SYSTEMS.md` - Updated documentation
- `test_coordinate_parity.sh` - Automated parity testing script

## References

- [Vulkan Coordinate Systems](https://www.khronos.org/registry/vulkan/specs/1.3/html/chap24.html#vertexpostproc-clipping)
- [DirectX Coordinate Systems](https://docs.microsoft.com/en-us/windows/win32/direct3d9/coordinate-systems)
- [Unity's approach to NDC differences](https://docs.unity3d.com/Manual/SL-PlatformDifferences.html)
