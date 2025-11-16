# DirectX Brightness Issue Analysis

## Problem Summary
DirectX 12 backend renders significantly darker than Vulkan backend. The issue appears to be related to color space handling (sRGB vs linear).

## Test Results

### Damaged Helmet Scene
- **Pixel Difference**: 681,712 pixels (~74% of image)
- **Brightness**: VK mean=29839, DX mean=22568 (DX is 24% darker)
- **File Size**: VK=316KB, DX=315KB (similar)

### Cube Scene
- **Pixel Difference**: 12,167 pixels (~19% of image)
- **Brightness**: VK mean=32661, DX mean=23035 (DX is 29% darker)
- **File Size**: VK=19.7KB, DX=19.7KB (similar)

## Technical Details

### Texture Formats
Both backends use the same format for offscreen rendering:
- **Vulkan**: `vk::Format::R8G8B8A8_UNORM` (linear)
- **DirectX**: `DXGI_FORMAT_R8G8B8A8_UNORM` (linear)

### Camera Matrices
Y-axis flip is implemented correctly:
- **Vulkan Row 1**: `[0.0, -2.4142134, 0.0, 0.0]` (negative Y)
- **DirectX Row 1**: `[0.0, 1.7320508, 0.0, 0.0]` (positive Y)

Note: Camera positions differ between backends in logs, but this appears to be a logging timing issue.

### Shader Code
- Both backends use the same HLSL shader (`forward_simple.hlsl`)
- No explicit gamma correction in shaders
- Texture sampling uses `.Sample()` which should respect texture format
- Lighting calculations appear identical

### Screenshot Capture
- Both use the same pixel format (RGBA8)
- No apparent differences in capture methodology
- Vertical flip test made the difference WORSE, ruling out simple Y-flip

## Potential Causes

1. **sRGB vs Linear Texture Sampling**
   - Material textures might be loaded as sRGB in Vulkan but linear in DirectX
   - Check texture creation code for format mismatches
   
2. **Render Target Format Mismatch**
   - While offscreen uses UNORM, intermediate buffers might differ
   - Check if render pass attachments use different formats
   
3. **Blending State Differences**
   - DirectX and Vulkan might have different default blend states
   - Check pipeline blending configuration
   
4. **Depth/Stencil State**
   - Different depth testing might affect lighting
   - Check depth attachment formats and states

5. **Texture Loading**
   - glTF material textures might be interpreted differently
   - Check if embedded textures are extracted with correct gamma

## sRGB Format Tests

Attempted fix: Changed material textures to use `Rgba8Srgb` instead of `Rgba8Unorm`.

### Results:
- **VK with UNORM**: mean=29839
- **VK with SRGB**: mean=29309 (2% darker - expected for correct sRGB handling)
- **DX with UNORM (old build)**: mean=22568
- **DX with UNORM (new build)**: mean=17535 (22% darker than old!)
- **DX with SRGB**: mean=17045 (even darker)

### Analysis:
The DirectX backend got DARKER in the new build, even with UNORM format. This suggests:
1. Something changed in the DX rendering path between builds
2. OR there's a configuration issue with how DX initializes textures/render targets
3. The sRGB fix made it worse because of double-conversion

## Current Status

**PAUSED** - Need to investigate why DX rendering changed between builds before proceeding with sRGB fixes.

The core issue appears to be that both backends are NOT at parity, and there's likely a fundamental difference in:
- Render target format
- Blending configuration
- Shader compilation
- Or texture sampling state

## Files Modified

- `src/app.rs` - Attempted to change texture format to sRGB (reverted)
- `src/render_graph/resource.rs` - Added Rgba8Srgb and Bgra8Srgb formats  
- `src/backends/vulkan/mod.rs` - Added sRGB format support
- `src/backends/directx/dx12_impl.rs` - Added sRGB format support (already had DXGI conversion)
