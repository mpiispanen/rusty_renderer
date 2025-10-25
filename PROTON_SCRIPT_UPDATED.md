# Proton Script Updated - 2025-10-25

## Summary

Updated `run_with_proton.sh` to accept the same arguments as the main rusty_renderer application, making it more flexible and easier to use.

## Changes Made

### 1. Updated `run_with_proton.sh`

**Previous Behavior:**
- Accepted positional arguments: scene file and VKD3D debug level
- Limited flexibility
- Different interface from main application

**New Behavior:**
- Accepts all rusty_renderer command-line arguments
- Forwards them directly to the Windows binary
- Adds `--vkd3d-debug` as script-specific option
- Defaults to windowed mode with `scenes/gltf_textured.toml`
- Automatically adds `--backend directx` (since we're using Proton)

### 2. Usage Examples

```bash
# Default: windowed mode, GLTF textured cube
./run_with_proton.sh

# Specific scene
./run_with_proton.sh --scene scenes/textured_cube.toml

# Custom window size
./run_with_proton.sh --width 1920 --height 1080

# Limited frame count for testing
./run_with_proton.sh --max-frames 1

# Combine arguments
./run_with_proton.sh --scene scenes/cube.toml --width 1280 --height 720 --max-frames 100

# VKD3D debug output
./run_with_proton.sh --vkd3d-debug debug

# Complex combination
./run_with_proton.sh --scene scenes/triangle.toml --width 1024 --height 768 --max-frames 10 --vkd3d-debug info
```

### 3. Supported Arguments

The script now forwards ALL rusty_renderer arguments:
- `--scene <FILE>` - Scene file to load
- `--width <WIDTH>` - Window width
- `--height <HEIGHT>` - Window height
- `--max-frames <N>` - Maximum frames to render
- `--list-scenes` - List available scenes
- `--list-pipelines` - List available pipelines
- Plus any future arguments added to rusty_renderer

Script-specific options:
- `--vkd3d-debug <LEVEL>` - VKD3D debug level (warn, info, debug)

## Testing

Tested successfully with:
1. Default arguments (no args) ✅
2. Custom scene (`--scene scenes/textured_cube.toml`) ✅
3. Custom dimensions (`--width 1280 --height 720`) ✅
4. Frame limiting (`--max-frames 1`) ✅
5. VKD3D debug levels (`--vkd3d-debug info`) ✅
6. Complex combinations ✅

All tests exit with code 0 and run without errors.

## Benefits

1. **Consistency** - Same interface as native Linux builds
2. **Flexibility** - Can use any rusty_renderer argument
3. **Future-proof** - Automatically supports new arguments
4. **Ease of use** - Smart defaults for common testing scenarios
5. **Documentation** - Clear usage in script header comments

## Documentation Updates

Updated `PROTON_HOWTO.md` with:
- New usage examples
- Complete list of supported arguments
- Better organization of command-line options

## Next Steps

This script is now ready for:
1. Testing DirectX backend with various scenes
2. Comparing DX vs Vulkan output
3. CI integration for cross-platform testing
4. Performance benchmarking

The script provides a clean, consistent interface for testing the DirectX backend on Linux via Proton.
