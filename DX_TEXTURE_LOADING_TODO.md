# DirectX Texture Loading - TODO

## Current Status

✅ **Vertex-colored geometry works perfectly**
- Cube scene renders correctly
- Default white texture binds properly
- No synchronization issues

❌ **Textured models fail during resource allocation**
- Damaged helmet scene fails to load
- Error: 0x80004005 (generic failure) during texture upload
- VKD3D warning: "A command list using this allocator is in the recording state"

## Problem Analysis

### Architecture Issue

**Current flow:**
1. `execute_graph()` is called
2. `allocate_graph_resources()` allocates all resources
3. For textures with initial_data, `create_texture()` is called
4. `create_texture()` calls `upload_to_texture()`
5. `upload_to_texture()` tries to create temp command list/allocator
6. **FAILS**: VKD3D-Proton doesn't support dynamic command allocator creation

**Root cause:** Texture uploads during `execute_graph` resource allocation conflict with the main rendering command list.

### Why Vulkan Works

Vulkan likely uses a different command buffer or has better isolation between upload and rendering command buffers.

### Why Default White Texture Works

The default white texture is created during **initialization** (before any frames), when the command list state is clean and there are no conflicts.

### Why Vertex Colors Work

Vertex-colored geometry doesn't require texture uploads during execute_graph - only buffer uploads, which don't have the same synchronization issues.

## Solutions Considered

### 1. Temporary Command List (ATTEMPTED - FAILED)
**Approach**: Create temporary command allocator/list for each texture upload
```rust
let temp_allocator = device.CreateCommandAllocator(...)?;
let temp_command_list = device.CreateCommandList(..., &temp_allocator, ...)?;
```
**Problem**: VKD3D-Proton reports "allocator in recording state" warning and fails with 0x80004005
**Likely cause**: VKD3D doesn't support on-the-fly command allocator creation during rendering

### 2. Batched Upload Pass (PROPER SOLUTION)
**Approach**:
1. During resource allocation, collect all textures that need data
2. After allocation, do a single upload pass:
   - Reset command list once
   - Record all texture uploads
   - Execute once
   - Wait for completion
3. Then proceed with rendering

**Pros**:
- Single command list used correctly
- Efficient (one execute for all uploads)
- Matches best practices

**Cons**:
- Requires refactoring resource allocation
- Need to track which textures need uploads

### 3. Pre-load All Textures (SIMPLEST)
**Approach**: Load all scene textures during scene loading, before execute_graph
```rust
fn load_scene() {
    // ...
    for material in scene.materials {
        if let Some(texture_path) = material.diffuse_texture {
            let texture = backend.create_texture_from_file(texture_path)?;
            texture_cache.insert(texture_id, texture);
        }
    }
}
```

**Pros**:
- Textures loaded once, cached
- No upload during execute_graph
- Simple implementation

**Cons**:
- Requires texture caching system
- All textures loaded upfront (memory usage)

## Recommended Implementation

### Phase 1: Texture Pre-loading (SHORT TERM)
1. Add texture cache to App struct
2. During `load_scene()`, load all material textures
3. Pass texture ResourceIds to render graph
4. execute_graph just looks up existing textures

### Phase 2: Batched Upload (LONG TERM)
1. Refactor `allocate_graph_resources` to separate allocation from upload
2. Create `upload_initial_data()` function that:
   - Takes list of textures needing data
   - Records all uploads to command list
   - Executes once, waits
3. Call sequence:
   ```
   allocate_resources() -> creates GPU resources
   upload_initial_data() -> uploads texture data
   execute_rendering()   -> actual rendering
   ```

## Workaround for Users

For now, use **vertex-colored scenes**:
- `scenes/cube.toml` - colored cube (WORKS)
- `scenes/triangle.toml` - colored triangle (WORKS)

Textured models will be supported once texture pre-loading is implemented.

## Implementation Checklist

- [ ] Design texture cache system
- [ ] Implement texture pre-loading in load_scene()
- [ ] Update render graph to reference cached textures
- [ ] Test with damaged helmet scene
- [ ] Add texture deduplication (don't reload same file)
- [ ] Handle texture unloading (memory management)
- [ ] Update documentation

## Related Issues

- Issue #87: Resource lifecycle management
- This TODO: Texture loading architecture

## Testing

### Works ✅
```bash
./run_with_proton.sh --scene scenes/cube.toml --headless --max-frames 1
# Exit code: 0, no warnings
```

### Doesn't Work ❌
```bash
./run_with_proton.sh --scene scenes/damaged_helmet.toml --headless --max-frames 1
# Exit code: 1, error: 0x80004005
# VKD3D warning about allocator in recording state
```

## Notes

The DirectX backend is **functionally complete** for vertex-colored rendering. Texture support requires architectural changes to the resource loading system, not fixes to the DirectX backend itself.

Once texture pre-loading is implemented, it will benefit ALL backends (better performance, fewer runtime allocations).
