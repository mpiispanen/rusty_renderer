# Resource Management Migration Status

## Overview
We are migrating from hardcoded resource paths to render graph-managed resources.

## Completed (2025-11-02)

### Shader Registration Centralization
- ✅ Created centralized `App::register_shaders()` function
- ✅ All shaders registered before building passes
- ✅ Passes no longer register shaders themselves
- ✅ Forward simple GLSL shaders added with SPIR-V compilation
- ✅ Both Vulkan and DirectX use unified shader sources (HLSL via DXC)

### Vulkan Backend
- ✅ Uses PipelineBuilder for pipeline compilation
- ✅ Compiles shaders from shader registry via `compile_to_spirv()`
- ✅ Shader module caching implemented
- ✅ All resources allocated through render graph

### Script Improvements
- ✅ Fixed `run_with_proton.sh` to handle scene name shortcuts

## In Progress

### DirectX Backend
- ⚠️ Still uses hardcoded pipeline creation in `execute_graph()`
- ⚠️ Loads shaders with hardcoded paths in `load_shader_source()`
- ⚠️ Does not use PipelineBuilder yet
- ⚠️ Needs to compile shaders from shader registry

## TODO

### High Priority
1. **DirectX Pipeline Compilation**
   - Implement `compile_pipeline_from_builder()` for DirectX
   - Use shader registry instead of `load_shader_source()`
   - Remove hardcoded shader paths

2. **Resource Management**
   - Remove any remaining hardcoded buffer/texture paths
   - Ensure all resources go through render graph

3. **Validation**
   - Test both backends render identical output
   - No validation errors in either backend
   - CI tests pass with backend parity checks

### Medium Priority
4. **Pipeline State Caching**
   - Cache compiled pipelines for DirectX (like Vulkan)
   - Avoid recompilation on every frame

5. **Shader Compilation**
   - Consider pre-compiling shaders at build time
   - Add shader hot-reloading for development

### Low Priority
6. **Documentation**
   - Document shader registration workflow
   - Add examples of adding new shaders
   - Document resource lifecycle

## Architecture Notes

### Current Shader Flow
1. `App::register_shaders()` registers all shaders with render graph
2. Passes declare which shaders they use via handles
3. During pipeline compilation:
   - Vulkan: Uses `PipelineBuilder` and shader registry
   - DirectX: Still uses hardcoded approach (needs migration)

### Target Shader Flow
1. All shaders registered centrally
2. All backends use `PipelineBuilder`
3. Backends compile from shader registry
4. No hardcoded paths anywhere

## Testing Status
- ✅ Vulkan renders successfully
- ✅ DirectX renders successfully (via Proton)
- ✅ All library tests pass
- ✅ Clippy passes
- ✅ Code formatting passes

## Next Steps
1. Implement `compile_pipeline_from_builder()` for DirectX
2. Remove `load_shader_source()` and use shader registry
3. Test backend parity
4. Update CI to validate rendering output
