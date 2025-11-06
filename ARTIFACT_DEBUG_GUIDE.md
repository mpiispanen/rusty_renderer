# Rendering Artifact Investigation

## Current Status

The renderer successfully loads and displays glTF models (like DamagedHelmet) with textures and shadow mapping. However, there's a reported visual artifact: a "model-shaped" region that shows the clear screen color and stays in the same position when moving the camera.

## What We Know

### Rendering Pipeline
1. **Shadow Map Pass** (PassId 0): Draws geometry from light's perspective into depth texture
2. **Forward Pass** (PassId 1): Draws geometry with lighting and shadows to color buffer

### Current Behavior
- Both passes draw the same geometry (46,356 indices for DamagedHelmet)
- Textures load and display correctly
- Camera movement works correctly
- ESC exits the application
- F12 captures screenshots to `screenshots/screenshot_<timestamp>.png`

### Debug Output
Run with debug logging to see detailed information:
```bash
RUST_LOG=debug cargo run --release
```

Key log messages:
- `=== DRAWING INDEXED GEOMETRY ===` - Shows draw call parameters
- Transform information (position, rotation, scale)
- Vertex/index counts
- Buffer binding confirmations

## Possible Causes

### 1. Depth Testing Issue
The artifact might be caused by depth buffer precision or configuration issues:
- Check if depth testing is enabled correctly
- Verify depth buffer clear value
- Check depth compare operation

### 2. Shadow Map Artifact
The shadow pass outputs might be bleeding through:
- Shadow map might not be fully cleared
- Shadow map viewport/scissor might be wrong
- Check shadow map attachment load/store operations

### 3. Multiple Draw Calls
If geometry is drawn multiple times with different parameters:
- Check render pass load operations (LOAD vs CLEAR)
- Verify only one draw call per pass
- Check if shadow pass is writing to color buffer

### 4. Backface Culling
Some triangles might be culled incorrectly:
- Verify winding order matches culling mode
- Check if normals are oriented correctly
- Ensure transform matrices don't flip geometry

## How to Debug

### 1. Capture Screenshots
Press F12 during runtime to capture the current frame:
```bash
cargo run --release --scene damaged_helmet
# Press F12 to capture
# Check screenshots/screenshot_<timestamp>.png
```

### 2. Disable Shadow Mapping
Test without shadows to isolate the issue:
```bash
cargo run --release --scene gltf_textured  # Scene without directional light
```

### 3. Simplify Geometry
Test with simpler geometry:
```bash
cargo run --release --scene cube  # Simple cube scene
```

### 4. Check Render Pass Configuration
Look at the forward pass setup in `src/passes/forward_simple.rs`:
- Color attachment load operation (should be CLEAR)
- Depth attachment load operation (should be CLEAR)
- Clear color value
- Depth clear value

### 5. Add Visual Debugging
Modify the fragment shader to output specific values:
- Output world position as color
- Output normals as color
- Output depth as grayscale
- Output a solid color to rule out lighting issues

## Quick Tests

### Test 1: Solid Color Output
Edit `shaders/hlsl/forward_simple.hlsl` fragment shader:
```hlsl
float4 PSMain(PSInput input) : SV_TARGET {
    return float4(1.0, 0.0, 1.0, 1.0);  // Solid magenta
}
```

If the artifact still appears with solid color, it's not a lighting/texturing issue.

### Test 2: Depth Visualization
```hlsl
float4 PSMain(PSInput input) : SV_TARGET {
    float depth = input.position.z;
    return float4(depth, depth, depth, 1.0);
}
```

This helps identify depth buffer issues.

### Test 3: Normal Visualization
```hlsl
float4 PSMain(PSInput input) : SV_TARGET {
    float3 n = normalize(input.normal) * 0.5 + 0.5;
    return float4(n, 1.0);
}
```

Helps identify if normals are correct.

## Next Steps

1. **Run interactive session** and capture screenshots using F12
2. **Compare screenshots** from different camera positions
3. **Disable shadows** and see if artifact persists
4. **Add shader debugging** to output specific values
5. **Check render pass clear operations** in backend implementations

## Files to Check

- `src/passes/forward_simple.rs` - Forward pass setup and execution
- `src/passes/shadow_map.rs` - Shadow map pass
- `src/backends/vulkan/mod.rs` - Vulkan backend render pass begin/end
- `src/backends/directx/dx12_impl.rs` - DirectX backend render pass begin/end
- `shaders/hlsl/forward_simple.hlsl` - Fragment shader
- `src/app.rs` - Main render loop

## Screenshot Controls

- **F12**: Capture screenshot (windowed mode)
- **ESC**: Exit application
- **Mouse**: Click to capture mouse, move to look around
- **WASD**: Move camera
- **QE**: Move up/down
- **Shift**: Move faster

---

Last updated: 2025-11-06
