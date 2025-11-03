# DirectX Rendering Fixes - November 2, 2025

## Summary

Fixed DirectX 12 backend to render correctly with the new render graph system. DirectX now successfully renders the cube scene via Proton/Wine.

## Issues Fixed

### 1. Missing DXIL Shaders
**Problem**: DirectX was trying to compile HLSL shaders at runtime but failing.
**Solution**: Pre-compiled HLSL shaders to DXIL format using DXC:
```bash
dxc -T vs_6_0 -E VSMain shaders/hlsl/forward_simple.hlsl -Fo shaders/forward_simple.vert.dxil
dxc -T ps_6_0 -E PSMain shaders/hlsl/forward_simple.hlsl -Fo shaders/forward_simple.frag.dxil
dxc -T vs_6_0 -E VSMain shaders/hlsl/triangle.hlsl -Fo shaders/triangle.vert.dxil
dxc -T ps_6_0 -E PSMain shaders/hlsl/triangle.hlsl -Fo shaders/triangle.frag.dxil
```

### 2. Command Allocator Not Reset
**Problem**: `E_INVALIDARG (0x80070057)` when resetting command list - command allocator wasn't being reset before use.
**Solution**: Added `command_allocator.Reset()?` before `command_list.Reset()` in `execute_graph`.
**File**: `src/backends/directx/dx12_impl.rs:1914`

### 3. Incorrect Vertex Input Semantic Mapping
**Problem**: Vertex attributes were mapped to wrong HLSL semantics:
- Location 1 was mapped to COLOR (should be NORMAL)
- Location 2 was mapped to NORMAL (should be TEXCOORD)
- Location 3 was mapped to TEXCOORD (should be COLOR)

**Solution**: Fixed semantic mapping to match vertex layout:
- Location 0 → POSITION ✓
- Location 1 → NORMAL ✓
- Location 2 → TEXCOORD ✓  
- Location 3 → COLOR ✓

**File**: `src/backends/directx/dx12_impl.rs:1694-1700`

### 4. Root Signature Parameter Order Mismatch
**Problem**: Root signature parameters didn't match shader register layout:
- Root signature had constants at b0, but shader expected Camera at b0
- This caused PSO creation to fail with `E_INVALIDARG`

**Solution**: Reordered root signature parameters to match shader:
```
Root Parameter 0: Camera CBV (b0)
Root Parameter 1: Lighting CBV (b1)  
Root Parameter 2: Root Constants (b2) - Push constants
```

**File**: `src/backends/directx/dx12_impl.rs:1608-1645`

## Testing

DirectX rendering now works successfully:
```bash
./run_with_proton.sh --scene cube --headless --screenshot test_cube.png
```

Output: `test_cube_dx.png` (800x600, properly rendered cube)

## Known Issues

1. Vulkan backend appears to be hanging (needs investigation)
2. Need to verify rendering parity between DirectX and Vulkan
3. Build script compiles shaders automatically but may need cleanup

## Next Steps

1. Debug and fix Vulkan rendering
2. Compare DirectX vs Vulkan output for parity
3. Add automated shader compilation to build process
4. Update CI to test both backends
5. Document shader compilation workflow
