# GLTF Texture Rendering Crash Fix

## Issue
The application was crashing when loading and rendering GLTF models with textures (e.g., damaged_helmet.glb). The crash occurred immediately after getting the albedo texture pointer but before actually binding it.

## Root Cause
The Vulkan pipeline descriptor set layout was missing binding 2 for the albedo/base color texture. The hardcoded descriptor layout in `compile_pipeline_from_builder()` only had:
- Binding 0: Lighting uniforms  
- Binding 1: Lighting uniforms (duplicate - should have been different)
- Binding 3: Shadow uniforms
- Binding 4: Shadow map texture

But the forward_simple shader was trying to use:
- Binding 0: Lighting uniforms
- Binding 1: Shadow uniforms
- **Binding 2: Albedo texture** (MISSING!)
- Binding 4: Shadow map texture

## Fix
Updated the descriptor set layout bindings in `src/backends/vulkan/mod.rs` (lines ~1133-1158) to include all required bindings:
- Binding 0: Lighting uniforms
- Binding 1: Shadow uniforms
- **Binding 2: Albedo/Base color texture** (ADDED)
- Binding 3: Material uniforms (reserved for future use)
- Binding 4: Shadow map texture

## Testing
- Application now runs without crashing
- GLTF models with textures load and render
- Camera controls work properly
- Textures bind successfully

## Remaining Issues (Non-critical)
1. **Descriptor set lifecycle** - Validation warns that descriptor sets are being destroyed/updated while bound
2. **Shadow map barriers** - Incorrect layout transitions for shadow map (treating depth image as color)
3. **Unused vertex attributes** - Normals, UVs, and colors not consumed by current vertex shader (performance warning)

These don't cause crashes but should be addressed for optimal performance and correctness.
