# DirectX Backend Parity Issues - 2025-11-16

## Current State

Both Vulkan and DirectX backends are rendering the damaged helmet model successfully!

## Remaining Issues

### 1. Clear Color Mismatch

**Observation:** DirectX shows black background, Vulkan shows dark blue background

**Expected Behavior:** Both should show the same clear color defined in the render pass

**Code Analysis:**
- Vulkan clear: `[0.1, 0.1, 0.2, 1.0]` at vulkan/mod.rs:1377
- DirectX clear: `[0.1, 0.1, 0.2, 1.0]` at dx12_impl.rs:2127
- Both use identical clear color values

**Hypothesis:** The clear may not be applied correctly, or there's a second clear happening somewhere

### 2. Orientation Mismatch (Y-Axis Flip)

**Observation:** DirectX helmet is upside down compared to Vulkan

**Expected Behavior:** Both backends should render the model with the same orientation

**Coordinate System Analysis:**

**Vulkan:**
- NDC Y-axis: -1 (top) to +1 (bottom) 
- Projection matrix: Y-axis flipped in camera/mod.rs:58 (`proj.y_axis *= -1.0`)
- Front face: Counter-clockwise

**DirectX:**
- NDC Y-axis: -1 (bottom) to +1 (top)
- Projection matrix: No flip (camera/mod.rs:64)
- Front face: Counter-clockwise (dx12_impl.rs:1838)

**Root Cause:** The coordinate system handling appears inconsistent. Vulkan flips Y in projection to account for its inverted NDC, but DirectX doesn't compensate for anything, resulting in opposite orientations.

**Possible Solutions:**
1. Add Y-flip to DirectX projection matrix as well
2. Remove Y-flip from Vulkan and handle it differently
3. Flip the viewport in one backend
4. Check if the issue is in texture coordinates or geometry loading

## Test Results

### Latest Screenshots
- `backend_comparison_fixed.png`: Side-by-side comparison showing both issues
  - Left (Vulkan): Dark blue background, helmet facing up
  - Right (DirectX): Black background, helmet facing down

### Test Commands
```bash
# Vulkan
./target/release/rusty_renderer --backend vulkan --headless --max-frames 1 --scene damaged_helmet

# DirectX (via Proton)
./run_with_proton.sh --headless --max-frames 1 --scene damaged_helmet
```

## Action Items

1. **Fix Clear Color:**
   - Add logging to confirm ClearRenderTargetView is called with correct values
   - Verify no second clear is happening
   - Check if render target is properly bound when clearing

2. **Fix Orientation:**
   - Test if adding Y-flip to DirectX projection fixes the issue
   - OR test if removing Y-flip from Vulkan works
   - Verify camera backend is set before any projection matrices are calculated
   - Add debug logging to print projection matrix values for comparison

3. **Move to Render Pass Configuration:**
   - Clear color should come from render pass definition, not hardcoded
   - Coordinate system handling should be transparent to the application

