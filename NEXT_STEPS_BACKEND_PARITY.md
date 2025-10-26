# Next Steps - 2025-10-26

## Immediate Priority: Backend Parity

### Visual Verification Needed
Both backends are now running, but we need visual confirmation that they render identically:

1. **Run both backends and capture screenshots**
   - Vulkan: `cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --pipeline forward`
   - DirectX: `./run_with_proton.sh --scene scenes/gltf_textured.toml --pipeline forward`

2. **Compare rendering output**
   - Check cube face visibility (should show front faces only)
   - Verify texture mapping matches
   - Confirm lighting calculations are identical
   - Ensure depth testing works (occlusion correct)

3. **Issues to check**:
   - Are colors identical?
   - Is texture orientation the same?
   - Do both handle transparency the same way?
   - Are lighting calculations producing the same results?

### Once Parity is Confirmed

1. **Enable CI Rendering Tests**
   - Headless rendering mode
   - Screenshot capture
   - Automated comparison
   - Run on both backends

2. **Remove Hardcoded Rendering**
   - Create pipeline template system (TOML files)
   - Move shader selection to configuration
   - Define render passes in scene files
   - Remove all embedded vertex/shader data

3. **Architecture Improvements**
   - Make RenderGraph fully data-driven
   - Render passes define requirements declaratively
   - Auto-generate resource bindings from shaders
   - Pipeline state from template files

## Current Working State

### ✅ What Works
- Both Vulkan and DirectX backends compile and run
- glTF loading with embedded textures
- Forward rendering with lighting
- Depth testing enabled
- Backface culling correctly configured (CCW winding)
- Multi-light support (directional + point lights)
- Material properties
- Camera transformations

### 🔧 What Needs Testing
- Visual parity between backends
- Texture sampling consistency
- Color output matching
- Lighting calculation accuracy

### 📝 What Needs Implementation
- Headless rendering mode for CI
- Screenshot comparison tools
- Pipeline template system
- Data-driven render pass configuration
- Automated resource binding

## Test Commands

### Quick Backend Test
```bash
# Vulkan
cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --pipeline forward --max-frames 60

# DirectX (via Proton)
./run_with_proton.sh --scene scenes/gltf_textured.toml --pipeline forward --max-frames 60
```

### Comparison Test
```bash
./test_rendering_comparison.sh
```

## Documentation Updates Needed

1. Update README.md with current backend status
2. Update design documents to remove wgpu references
3. Create architecture document for data-driven rendering
4. Document pipeline template format
5. Create guide for adding new render passes

## GitHub Issues to Create/Update

1. **Backend Parity Tracking**
   - Visual comparison results
   - Identified differences
   - Fix priority

2. **CI Rendering Tests**
   - Headless mode implementation
   - Screenshot capture
   - Comparison tooling

3. **Architecture Refactor**
   - Pipeline templates
   - Data-driven render passes
   - Resource binding automation

4. **Remove Hardcoding**
   - Embedded shader removal
   - Vertex data cleanup
   - Default configuration removal
