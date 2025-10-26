# wgpu Backend Removal - Complete

## Date
2025-10-26

## Reason
After extensive investigation, the wgpu backend has fundamental limitations for interactive rendering:

1. **Platform limitation**: wgpu's Vulkan backend on AMD+Linux doesn't properly implement present synchronization
2. **API limitation**: No way to wait for GPU without blocking event loop (needed for winit)
3. **Limited benefit**: Only useful for WebGPU target, which is not a priority
4. **Maintenance burden**: Requires completely different architecture (async render pipeline)

## What Was Removed

### Code
- `src/backends/wgpu_backend/` - Entire backend implementation (2500+ lines)
- Backend enum values for `Wgpu`/`wgpu`
- wgpu-specific code paths in passes
- Import statements and module declarations

### Dependencies
- `wgpu = "23.0"`
- `pollster = "0.3"`

### Documentation Updates
- README.md - Removed wgpu references
- Cargo.toml - Updated description and keywords
- All backend enums simplified

## What Still Works

✅ **Vulkan backend** - Primary Linux target, fully functional
✅ **DirectX 12 backend** - Windows target, fully functional  
✅ **All existing scenes and pipelines**
✅ **Headless rendering**
✅ **CI/CD pipeline**

## Impact

- ✅ Simpler codebase (2500+ lines removed)
- ✅ Fewer dependencies (wgpu ecosystem removed)
- ✅ Clearer architecture (no dual sync/async paths)
- ✅ Faster builds (less to compile)
- ❌ No WebGPU target (but wasn't working anyway)

## Future Considerations

If WebGPU becomes critical:
1. Consider async architecture redesign (3-4 weeks)
2. Or use separate project with wgpu-first design
3. Or wait for wgpu present synchronization fixes

For now, Vulkan + DirectX covers all production use cases.

## Related Documents

- `WGPU_SOLUTION_FINAL.md` - Why it didn't work
- `WGPU_ARCHITECTURE_ANALYSIS.md` - What it would take to fix

## Commit

```
git commit -m "Remove wgpu backend - fundamental platform limitations

After extensive debugging, wgpu's Vulkan backend has fundamental
synchronization issues on AMD+Linux that prevent interactive rendering
beyond 4-5 frames. The API doesn't expose the primitives needed to
properly wait for GPU without blocking the event loop.

To fully support wgpu would require:
- Async render pipeline architecture
- Frame scheduler system  
- Event loop restructuring
- Estimated 3-4 weeks work
- 50%+ maintenance increase

Not worth the complexity for a WebGPU target that isn't a priority.

Removed:
- src/backends/wgpu_backend/ (2500+ lines)
- wgpu dependency and pollster
- Backend enum Wgpu variants
- wgpu-specific code paths

Updated:
- README.md - Removed wgpu references
- Cargo.toml - Simplified description
- All backend selection code

Result: Cleaner, simpler codebase focused on Vulkan + DirectX.
```
