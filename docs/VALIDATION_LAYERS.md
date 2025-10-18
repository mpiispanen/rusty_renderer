# Validation Layers

Rusty Renderer supports validation/debug layers on all graphics backends for enhanced debugging and error detection during development.

## Overview

Validation layers provide:
- **API usage validation** - Detects incorrect API usage patterns
- **Memory tracking** - Identifies memory leaks and invalid memory access
- **Performance warnings** - Highlights suboptimal usage patterns
- **Thread safety checks** - Validates multi-threaded usage

## Enabling Validation

Validation is controlled via the `--debug` command-line flag:

```bash
# Enable validation layers
cargo run -- --debug

# Enable validation with specific backend
cargo run -- --backend vulkan --debug
cargo run -- --backend wgpu --debug
cargo run -- --backend directx --debug  # Windows only
```

## Backend-Specific Details

### Vulkan

Vulkan validation uses the Khronos validation layer (`VK_LAYER_KHRONOS_validation`).

**Requirements:**
- Vulkan SDK or validation layer package installed
- On Linux: `vulkan-validation-layers` package
- On Windows: Vulkan SDK with validation layers

**Features:**
- Comprehensive API validation
- Best practices warnings
- Shader validation
- Synchronization validation

**Example output:**
```
[INFO] Validation layers enabled
[INFO] Vulkan [VALIDATION]: vkCreateInstance(): Khronos Validation Layer Active
```

### wgpu

wgpu provides validation through its `VALIDATION` and `DEBUG` instance flags, which enable backend-specific validation:
- **Vulkan**: Uses Vulkan validation layers
- **DirectX 12**: Uses D3D12 debug layer
- **Metal**: Uses Metal validation

**Features:**
- Cross-platform validation abstraction
- Automatic backend selection
- Detailed error messages

**Example output:**
```
[INFO] wgpu validation and debug enabled
[INFO] wgpu_hal::vulkan::instance] GENERAL [Loader Message]
```

### DirectX 12 (Windows only)

DirectX 12 validation uses the D3D12 debug layer and DXGI debug factory.

**Requirements:**
- Windows 10/11 with Graphics Tools feature installed
- Install via: Settings → Apps → Optional Features → Graphics Tools

**Features:**
- D3D12 API validation
- Resource state tracking
- GPU-based validation (when available)
- Live object tracking

**Example output:**
```
[INFO] DirectX 12 debug layer enabled
[INFO] Creating DXGI factory (debug mode)
```

## Performance Impact

**Warning:** Validation layers have significant performance overhead:
- **Vulkan**: 20-50% slower frame times
- **wgpu**: 15-40% slower (varies by backend)
- **DirectX 12**: 30-60% slower with GPU validation

**Best Practices:**
- Only enable validation during development and debugging
- Disable for performance testing and release builds
- Use `--max-frames` for faster iteration during validation testing

## Disabling Validation

Validation is disabled by default. Simply omit the `--debug` flag:

```bash
cargo run                      # No validation
cargo run -- --backend vulkan  # No validation
```

## CI Integration

For CI testing, validation layers can be selectively enabled:

```yaml
# Enable validation for test runs
- run: cargo run -- --debug --max-frames 10

# Performance testing without validation
- run: cargo run -- --max-frames 100
```

## Troubleshooting

### Vulkan: "Validation layers requested but not available"

**Solution:** Install validation layers:
```bash
# Fedora/RHEL
sudo dnf install vulkan-validation-layers

# Ubuntu/Debian
sudo apt install vulkan-validationlayers

# Arch
sudo pacman -S vulkan-validation-layers
```

### DirectX: "Debug layer not available"

**Solution:** Install Graphics Tools:
1. Open Settings
2. Navigate to Apps → Optional Features
3. Click "Add a feature"
4. Search for "Graphics Tools"
5. Install and reboot

### wgpu: No validation output

**Cause:** Backend validation layers not available

**Solution:** Install backend-specific validation:
- On Linux: Install Vulkan validation layers (see above)
- On Windows: Install Graphics Tools (see DirectX section)
- On macOS: Metal validation is built-in

## Related Documentation

- [DESIGN.md](DESIGN.md) - Overall architecture
- [TESTING_DIRECTX_ON_LINUX.md](TESTING_DIRECTX_ON_LINUX.md) - DirectX testing via Proton
- [COORDINATE_SYSTEMS.md](COORDINATE_SYSTEMS.md) - Backend coordinate differences
