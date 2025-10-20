# CI Test Plan for M8.3

## What We're Testing

### Linux (Vulkan + wgpu)
- ✅ Bind group layout creation
- ✅ Bind group creation  
- ✅ Triangle renders with vertex buffers
- ✅ Descriptor pool management

### Windows (DirectX 12 + WARP)
- 🧪 Root signature creation from BindGroupLayout
- 🧪 Descriptor heap initialization
- 🧪 Bind group methods don't crash
- 🧪 Triangle still renders (doesn't use bind groups yet)

## Expected Results

### Should PASS:
- All Linux tests (Vulkan primary)
- Windows build
- Windows triangle example (WARP)

### May FAIL (acceptable):
- DirectX bind group usage (not connected to rendering yet)
- Texture/sampler binding (M8.4 dependency)

## How to Interpret Results

✅ **Green CI** = Vulkan + wgpu working, DirectX compiles
⚠️ **Yellow CI** = Some DirectX issues but Vulkan works
❌ **Red CI** = Compilation errors or Vulkan broken

## Next Steps After CI

1. If CI passes: Move to M8.4 (Texture Loading)
2. If DirectX fails: Add more defensive checks
3. If Vulkan fails: Fix immediately

---
Generated: 2025-10-20
