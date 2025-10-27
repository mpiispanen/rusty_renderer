# DirectX Shader Compilation Fix - 2025-10-26

## Problem
DirectX backend was failing to render with a shader compilation error:
```
Variable "color" was already declared in this scope.
```

## Root Cause
The HLSL shader in `shaders/hlsl/forward.hlsl` had a variable naming conflict:
- Line 65: Declared local variable `float3 color`
- Line 74: Referenced `input.color` (struct member)

HLSL compiler was treating these as conflicting declarations in the same scope.

## Solution
Renamed the local variable from `color` to `albedo` to avoid the conflict:
- `float3 color = baseColor.rgb;` → `float3 albedo = baseColor.rgb;`
- Updated all references throughout the pixel shader

## Files Changed
- `shaders/hlsl/forward.hlsl` - Fixed variable naming conflict
- `windows_test_directx/shaders/hlsl/forward.hlsl` - Synced shader copy

## Testing
- DirectX backend now compiles shaders successfully
- Renders 3 frames and exits cleanly (exit code 0)
- Screenshot saved to `gltf_textured_dx12.png`

## Rendering Configuration
Both Vulkan and DirectX backends now have matching configuration:
- **Culling**: Back-face culling enabled
- **Front Face**: Counter-clockwise winding order  
- **Depth Testing**: Enabled with LESS comparison
- **Depth Write**: Enabled

## Next Steps
1. Compare DX and Vulkan screenshot outputs
2. Verify texture sampling is working correctly
3. Check that lighting calculations match between backends
4. Enable CI rendering tests for both backends
