# DirectX Backend Parity Status

## Overview
This document tracks the progress of bringing the DirectX 12 backend to feature parity with the Vulkan backend.

## Completed Features ✅

### Core Rendering
- ✅ **Render Graph Execution**: Full render graph support with multi-pass rendering
- ✅ **Resource Layout Types**: Support for ShadowMap, ForwardRendering, and Custom layouts
- ✅ **Per-Pass Pipelines**: Dynamic pipeline compilation based on pass requirements
- ✅ **Root Signature Management**: Layout-specific root signatures for different pass types
- ✅ **Swapchain Blit**: Copy render graph output to swapchain in windowed mode (added today)

### Resource Management
- ✅ **Buffer Creation**: Vertex, index, uniform, and staging buffers
- ✅ **Buffer Upload**: CPU-to-GPU data transfer
- ✅ **Texture Creation**: 2D textures with various formats
- ✅ **Texture Upload**: Pixel data upload via staging buffers
- ✅ **Sampler Creation**: Texture sampling support
- ✅ **Descriptor Heaps**: CBV/SRV/UAV descriptor management

### Shader Support
- ✅ **DXIL Compilation**: Automatic shader compilation via DXC
- ✅ **Forward Rendering Shaders**: Full lighting and texture support
- ✅ **Shadow Map Shaders**: Depth-only rendering for shadow mapping
- ✅ **Shader Registry**: Integration with render graph shader system

### Pipeline Features
- ✅ **Dynamic Pipeline State**: Per-pass PSO creation
- ✅ **Input Layouts**: Configurable vertex attributes
- ✅ **Depth Testing**: Z-buffer support with configurable compare ops
- ✅ **Culling**: Backface culling support
- ✅ **Winding Order**: Counter-clockwise (glTF standard)

### Multi-Pass Architecture
- ✅ **Per-Pass Framebuffers**: Dynamic RTV/DSV creation per pass (Issue #94)
- ✅ **Resource Transitions**: Automatic state tracking and barriers
- ✅ **Shadow Map Pass**: Dedicated depth-only pass for shadows
- ✅ **Forward Pass**: Full lighting with shadow map sampling

### Platform Support
- ✅ **Headless Rendering**: Offscreen rendering for testing/CI
- ✅ **Windowed Rendering**: Full swapchain management
- ✅ **WARP Device**: Software rasterizer support for CI

## Remaining Work 🚧

### High Priority

1. **Texture Binding in Execute Context** (TODO line 3352)
   - Dynamic texture/sampler descriptor tables in bind groups
   - Currently: Only CBV bindings work fully
   - Impact: Textures must be bound via root signature static samplers (works but less flexible)
   - Required for: Advanced material systems with many textures

2. **Dynamic Descriptor Creation** (TODO line 3434)
   - Actually populate descriptor heaps with SRV/CBV/UAV descriptors
   - Currently: Placeholder - descriptors tracked but not created
   - Impact: Bind groups partially functional
   - Required for: Complex multi-texture materials

### Medium Priority

3. **Swap Chain Recreation** (TODO line 1132)
   - Handle window resize gracefully
   - Currently: May have issues on resize
   - Impact: Window resizing may not work properly

4. **Per-Resource RTV Creation** (TODO line 2089)
   - Create render target views for arbitrary textures
   - Currently: Uses swapchain RTV by default
   - Impact: Render-to-texture scenarios may be limited

5. **Per-Resource DSV Creation** (TODO line 2120)
   - Create depth stencil views for arbitrary textures
   - Currently: Uses main depth buffer
   - Impact: Multiple depth targets may not work

6. **Resource Lifecycle Management** (TODO line 2151, Issue #87)
   - Smart cleanup and deduplication
   - Currently: Resources created but may leak
   - Impact: Memory usage over time

### Low Priority

7. **Configurable Vertex Stride** (TODO lines 3466, 3932)
   - Per-buffer stride configuration
   - Currently: Assumes packed GPU vertex layout
   - Impact: Non-standard vertex formats unsupported

### Testing

8. **Expand Test Coverage**
   - Vulkan has 15 tests, DirectX has only 2
   - Need tests for:
     - Texture upload/binding
     - Multi-pass rendering
     - Shadow mapping
     - Resource transitions
     - Descriptor management

## Feature Parity Comparison

| Feature | Vulkan | DirectX | Notes |
|---------|--------|---------|-------|
| Basic rendering | ✅ | ✅ | |
| Render graph execution | ✅ | ✅ | |
| Multi-pass rendering | ✅ | ✅ | Issue #94 resolved |
| Shadow mapping | ✅ | ✅ | Layout-based root signatures |
| Texture loading | ✅ | ✅ | |
| Texture binding | ✅ | ⚠️ | Works via static samplers |
| Dynamic descriptors | ✅ | ⚠️ | Tracked but not created |
| Window resize | ✅ | ⚠️ | TODO |
| Render-to-texture | ✅ | ⚠️ | Limited |
| Swapchain blit | ✅ | ✅ | Added 2025-11-08 |
| Headless mode | ✅ | ✅ | |
| WARP support | N/A | ✅ | DirectX-specific |
| Coordinate system parity | ✅ | ✅ | Fixed 2025-11-16 |

Legend:
- ✅ Fully implemented
- ⚠️ Partially implemented or has limitations
- ❌ Not implemented
- N/A Not applicable

## Recent Changes

### Session: 2025-11-16 - Coordinate System Fix

#### Coordinate System Parity ✅
- **Fixed**: Y-axis orientation mismatch between Vulkan and DirectX
- **Problem**: Both backends using same projection matrix despite different NDC conventions
- **Solution**: Negate Y-axis in projection matrix for Vulkan only
- **Impact**: Both backends now render with identical orientation
- **Testing**: Verified with cube and damaged helmet scenes
- **Documentation**: Added `COORDINATE_SYSTEM_FIX.md` with detailed explanation

### Session: 2025-11-08

### 1. Resource Layout Types
- Added `ResourceLayoutType` enum to `RenderPass`
- Supports: `ShadowMap`, `ForwardRendering`, `Custom`
- DirectX now uses layout type instead of pass name heuristics
- Root signatures created based on explicit layout requirements

### 2. Root Signature Improvements
- Shadow map pass: Only light uniforms (b0)
- Forward pass: Lighting (b0), shadows (b1), textures (t0, t1), samplers (s1, s2)
- Proper descriptor ranges for multiple textures
- Static samplers for base color and shadow comparison

### 3. Swapchain Blit (New)
- Added logic to copy render graph final output to actual swapchain
- Matches Vulkan's blit implementation
- Only applies in windowed mode
- Uses DirectX `CopyResource` with proper state transitions

## Next Steps

### Immediate (to complete parity)
1. Implement dynamic texture descriptor tables (TODO 3352)
2. Add descriptor heap population (TODO 3434)
3. Test with complex textured scenes

### Short-term
1. Fix swap chain recreation (TODO 1132)
2. Implement per-resource RTV/DSV creation (TODOs 2089, 2120)
3. Add more unit tests to match Vulkan coverage

### Long-term
1. Implement resource lifecycle tracking (Issue #87)
2. Support non-standard vertex layouts (TODOs 3466, 3932)
3. Optimize descriptor management and caching

## Known Limitations

1. **~~Dark Rendering Output~~** ✅ **RESOLVED** (2025-11-16)
   - ~~DirectX renders but output is almost black~~
   - **Root cause**: Vulkan and DirectX have different NDC Y-axis conventions
     - Vulkan: Y goes from -1 (top) to +1 (bottom) - INVERTED
     - DirectX: Y goes from -1 (bottom) to +1 (top) - STANDARD
   - **Fix**: Applied Y-axis negation in projection matrix for Vulkan only
   - **Location**: `src/camera/mod.rs::perspective_projection()`
   - **Result**: Both backends now render with identical orientation
   - **Documentation**: See `COORDINATE_SYSTEM_FIX.md` for details

2. **Texture Binding**: Currently relies on static samplers in root signature
   - Works for fixed number of textures
   - Less flexible than dynamic descriptor tables
   - Not a blocker for most use cases

3. **Descriptor Heap Management**: Simplified allocation
   - No deduplication or recycling
   - May waste GPU memory with many resources
   - Acceptable for current use cases

4. **Testing**: Limited DirectX-specific test coverage
   - Most testing done via Vulkan backend
   - DirectX behavior assumed similar
   - Need more platform-specific validation

## Performance Notes

- DirectX backend performance should be comparable to Vulkan
- Both use explicit GPU synchronization
- DirectX has some overhead for root signature switching
- WARP device significantly slower (software rasterization)

## Platform-Specific Features

### DirectX-Only
- WARP device support (software rasterizer)
- Native Windows integration
- DirectX 12 debug layer
- PIX integration (for debugging)

### Vulkan-Only
- Cross-platform (Linux, Windows, macOS via MoltenVK)
- Validation layers
- RenderDoc integration

## References

- Issue #94: Per-pass render passes and framebuffers (RESOLVED)
- Issue #87: Resource lifecycle management (PENDING)
- `SHADOW_MAP_TODO.md`: Shadow mapping architecture notes
- `RENDERPASS_ARCHITECTURE_FIX.md`: Multi-pass architecture documentation
