# Running DirectX Backend with Proton

This guide explains how to run the DirectX 12 backend on Linux using Proton.

## Prerequisites

1. **Proton**: Install via Steam
   - Open Steam
   - Install any game that uses Proton (or install Proton directly)
   - Default location: `~/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)/`

2. **Windows Cross-Compilation Toolchain**:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

3. **Required Directories**: The script will automatically sync these:
   - `shaders/` - Shader files (HLSL/SPIR-V)
   - `assets/` - Textures, models, etc.
   - `scenes/` - Scene definition files

## Building

Build the Windows executable:
```bash
cargo build --release --target x86_64-pc-windows-gnu
```

## Running

### Quick Start
```bash
./run_with_proton.sh
```

This will:
1. Automatically sync the binary from `target/x86_64-pc-windows-gnu/release/`
2. Copy all required assets, scenes, and shaders to `windows_test_directx/`
3. Run with the default scene using the DirectX backend

### Common Options

```bash
# Run with a specific scene
./run_with_proton.sh --scene scenes/cube.toml

# Run with frame limit (useful for testing)
./run_with_proton.sh --max-frames 10

# Run with debug output
./run_with_proton.sh --vkd3d-debug debug

# Combine options
./run_with_proton.sh --scene scenes/gltf_textured.toml --max-frames 5 --vkd3d-debug info
```

### Environment Variables

The script sets these automatically, but you can override them:

- `STEAM_COMPAT_CLIENT_INSTALL_PATH`: Steam installation path
- `STEAM_COMPAT_DATA_PATH`: Proton prefix (Wine prefix equivalent)
- `VKD3D_DEBUG`: VKD3D-Proton debug level (none/warn/info/debug)
- `RUST_LOG`: Rust logging level (error/warn/info/debug/trace)
- `RUST_BACKTRACE`: Enable Rust backtraces (0/1/full)

## Configuration

Edit the script to change defaults:

```bash
# In run_with_proton.sh
PROTON_DIR="$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)"
COMPAT_DATA="$HOME/.proton_rusty_renderer"  # Wine prefix location
TEST_DIR="windows_test_directx"              # Working directory
```

## Known Issues

### Argument Parsing - FIXED
The DirectX backend value must be `directx` (lowercase, no hyphen), not `direct-x`.  
The clap ValueEnum derives names from the Rust enum variant name using kebab-case conversion.  
Fixed by adding `#[value(name = "directx")]` attribute to the DirectX enum variant.

### Runtime Error (CURRENT ISSUE)
The application starts but fails during execution with error code `0x80004005` (E_FAIL).
This is a generic DirectX/COM error that needs further investigation.

**Status**: Under investigation  
**Last seen**: Buffers create successfully, but application fails during render loop
**Debug log location**: `windows_test_directx/rusty_renderer_debug.log`

## Troubleshooting

### "Proton not found"
- Install Proton via Steam
- Or update `PROTON_DIR` in the script to point to your Proton installation

### "Binary not found"
- Build the Windows binary first:
  ```bash
  cargo build --release --target x86_64-pc-windows-gnu
  ```

### "No such file or directory" for assets/scenes
- The script automatically syncs these directories
- Make sure they exist in the project root
- Check that the script completed the sync step successfully

### Shader compilation errors
- Ensure shaders are compiled for DirectX (HLSL or SPIR-V with DXIL)
- Check that `shaders/` directory contains the required shader files

### Graphics/rendering issues
- Try different VKD3D debug levels: `--vkd3d-debug debug`
- Check Vulkan drivers are up to date (VKD3D-Proton translates D3D12 to Vulkan)
- Review logs in the console output

## How It Works

1. **Proton** is Valve's Wine fork with gaming enhancements
2. **VKD3D-Proton** translates DirectX 12 calls to Vulkan
3. The script:
   - Builds/syncs the Windows `.exe` and all assets
   - Sets up a Wine prefix at `~/.proton_rusty_renderer`
   - Runs the `.exe` through Proton with the DirectX backend
   - Proton translates D3D12 → Vulkan → Your GPU

## Quick Reference

```bash
# Standard workflow:
cargo build --release --target x86_64-pc-windows-gnu  # Build
./run_with_proton.sh --scene scenes/cube.toml          # Run

# For testing/debugging:
./run_with_proton.sh --max-frames 10 --vkd3d-debug debug

# Compare with Vulkan backend:
cargo run --release -- --backend vulkan --scene scenes/cube.toml
./run_with_proton.sh --scene scenes/cube.toml
```

## Notes

- The backend is **automatically set to DirectX** in the script (`--backend directx` is added for you)
- **Do NOT manually add `--backend directx`** - the script does this automatically
- All paths in scene files should be relative to the working directory
- The test directory (`windows_test_directx/`) is the working directory for the Windows binary
- Assets are copied, not symlinked, so changes require re-running the script
- To run in headless mode, add `--headless` to the script arguments
- The script passes all unknown arguments directly to rusty_renderer.exe
