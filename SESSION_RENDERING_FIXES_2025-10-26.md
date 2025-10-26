# Rendering Fixes Session - 2025-10-26

## Issues Identified and Fixed

### 1. Backface Culling Configuration
**Problem**: Both Vulkan and DirectX backends were configured to treat clockwise winding order as front-facing, but glTF uses counter-clockwise by default. This caused the renderers to show back faces instead of front faces.

**Fix**:
- **Vulkan**: Changed `front_face` from `CLOCKWISE` to `COUNTER_CLOCKWISE` in pipeline rasterization state
- **DirectX**: Changed `FrontCounterClockwise` from `FALSE` to `TRUE` in rasterizer description

**Files Modified**:
- `src/backends/vulkan/mod.rs` (line 786)
- `src/backends/directx/dx12_impl.rs` (line 821)

### 2. wgpu Backend Status
**Decision**: Removed wgpu backend from active development and documentation as decided earlier. The backend had persistent swapchain issues that were not core to the project goals.

**Current Active Backends**:
- ✅ Vulkan (Linux native)
- ✅ DirectX 12 (Windows via cross-compilation, tested with Proton)

## Testing Results

### Vulkan Backend
- ✅ Builds successfully
- ✅ Runs with forward pipeline
- ✅ Loads glTF scenes
- ✅ Renders textured cube with lighting
- ✅ Backface culling now correct (counter-clockwise)
- ✅ Depth testing enabled

**Test Command**:
```bash
cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --pipeline forward --max-frames 60
```

### DirectX 12 Backend
- ✅ Cross-compiles successfully (x86_64-pc-windows-gnu)
- ✅ Runs via Proton on Linux
- ✅ Loads glTF scenes
- ✅ Renders textured cube with lighting
- ✅ Backface culling now correct (counter-clockwise)
- ✅ Depth testing enabled
- ✅ Drawing 36 vertices per frame

**Test Command**:
```bash
./run_with_proton.sh --scene scenes/gltf_textured.toml --pipeline forward --max-frames 60
```

## Technical Details

### glTF Standard Compliance
Both backends now follow the glTF 2.0 specification which uses:
- Counter-clockwise winding order for front faces
- Right-handed coordinate system
- Backface culling enabled by default

### Rendering Pipeline Configuration

**Vulkan**:
```rust
.cull_mode(vk::CullModeFlags::BACK)
.front_face(vk::FrontFace::COUNTER_CLOCKWISE)
.depth_test_enable(true)
.depth_write_enable(true)
.depth_compare_op(vk::CompareOp::LESS)
```

**DirectX 12**:
```rust
CullMode: D3D12_CULL_MODE_BACK,
FrontCounterClockwise: TRUE,
DepthEnable: TRUE,
DepthWriteMask: D3D12_DEPTH_WRITE_MASK_ALL,
DepthFunc: D3D12_COMPARISON_FUNC_LESS,
```

## Current Rendering Features

Both backends now support:
- ✅ Forward rendering pipeline
- ✅ glTF model loading
- ✅ Texture mapping
- ✅ Per-vertex colors
- ✅ Lighting (directional and point lights)
- ✅ Camera transformations
- ✅ Depth testing
- ✅ Backface culling (correct winding order)
- ✅ Material properties

## Next Steps (As Per User Request)

1. **Backend Parity**: Ensure both Vulkan and DirectX render identically
   - Visual comparison testing
   - Verify texture sampling matches
   - Ensure lighting calculations are identical

2. **CI Rendering**: Enable automated rendering tests in CI/CD
   - Screenshot comparison
   - Headless rendering tests
   - Cross-platform validation

3. **Remove Hardcoded Rendering**: 
   - Move all rendering configuration to scene files
   - Make render passes data-driven
   - Define shader bindings via configuration

4. **Render Graph Architecture**:
   - Make render passes define shaders and bindings declaratively
   - Let RenderGraph handle all resource management
   - Remove any hardcoded rendering paths

## Testing Scripts Created

- `test_rendering_comparison.sh`: Compare Vulkan and DirectX rendering side-by-side

## Verification

To verify the fixes:
1. Run `./test_rendering_comparison.sh` to test both backends
2. Check that cube faces are now correct (not showing back faces)
3. Verify depth testing works (closer faces occlude farther ones)
4. Confirm textures render correctly on both backends

## Known Issues

None identified in this session. Both backends are rendering correctly with the forward pipeline and glTF scenes.
