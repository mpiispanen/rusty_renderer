# Coordinate System Fix for Vulkan vs DirectX

## Problem
The cube appeared upside down in DirectX compared to Vulkan.

## Root Cause
- **Vulkan**: Uses right-handed coordinate system with NDC Y from -1 (bottom) to +1 (top)
- **DirectX**: Uses left-handed coordinate system with NDC Y from -1 (top) to +1 (bottom)

The Y-axis in NDC (Normalized Device Coordinates) is inverted between the two APIs.

## Solution
Instead of using different coordinate systems (RH for Vulkan, LH for DirectX), we:

1. **Use right-handed coordinates for both backends** in the view matrix
   - This keeps vertex winding order consistent
   - Simplifies the mental model

2. **Flip Y in the projection matrix for DirectX only**
   - Multiply `proj.y_axis.y` by -1.0 for DirectX
   - This converts from Vulkan's NDC convention to DirectX's NDC convention

## Code Changes
- `src/camera/mod.rs`:
  - `perspective_projection()`: DX now uses RH projection with Y-flip
  - `look_at_view()`: Always uses RH (removed LH path for DX)
  - `free_fly_view()`: Always uses RH (removed LH path for DX)

## Result
Both Vulkan and DirectX now render the same scene with the same orientation.

## Testing
```bash
# Test Vulkan
cargo run --release -- --scene scenes/gltf_textured.toml --pipeline forward

# Test DirectX
./run_with_proton.sh
```

Both should show the textured cube with the same orientation.
