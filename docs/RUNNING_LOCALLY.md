# Running Locally

## Prerequisites

### Vulkan Runtime
You need the Vulkan runtime installed on your system:

- **Linux**: 
  ```bash
  # Ubuntu/Debian
  sudo apt install vulkan-tools libvulkan-dev
  
  # Fedora
  sudo dnf install vulkan-tools vulkan-loader-devel
  
  # Arch
  sudo pacman -S vulkan-tools vulkan-icd-loader
  ```

- **macOS**: 
  ```bash
  # Install Vulkan SDK from LunarG or use MoltenVK
  brew install --cask vulkan-sdk
  ```

- **Windows**: 
  Download and install the Vulkan SDK from [LunarG](https://vulkan.lunarg.com/)

### Verify Vulkan Installation
```bash
vulkaninfo | head -20
```

## Build and Run

### Triangle Example (Recommended)
The simplest way to see the renderer in action:

```bash
cargo run --example triangle --release
```

This will:
- Open an 800x600 window titled "Rusty Renderer"
- Display a colorful triangle (RGB vertices hardcoded in shader)
- Enable Vulkan validation layers in debug mode

### With Detailed Logging
```bash
RUST_LOG=debug cargo run --example triangle
```

### Development Build (Slower, More Validation)
```bash
cargo run --example triangle
```

## Controls

- **ESC** or **Close button**: Exit the application
- The window can be resized, and the renderer will automatically handle swapchain recreation

## Troubleshooting

### "No suitable GPU found"
- Ensure your GPU supports Vulkan 1.0+
- Check that GPU drivers are up to date
- Verify with: `vulkaninfo`

### "Failed to create Vulkan instance"
- Install Vulkan runtime/SDK (see Prerequisites)
- On Linux, ensure `libvulkan.so.1` is in your library path

### Black Screen / No Triangle
- Check console for errors
- Verify shaders are embedded: `cargo build` should show shader compilation
- Try with `RUST_LOG=debug` for detailed logs

### Validation Layer Errors
- Update GPU drivers
- These are warnings about best practices, not critical errors
- Disable with: set `debug: false` in `Config`

## What You Should See

A window displaying a triangle with:
- **Top vertex**: Red
- **Bottom-left vertex**: Green  
- **Bottom-right vertex**: Blue
- **Background**: Black
- Smooth color interpolation across the triangle

## Performance Notes

- **Debug build**: ~60 FPS (vsync enabled)
- **Release build**: ~60 FPS (vsync enabled, can disable for higher FPS)
- Validation layers add overhead in debug mode
- Use `--release` for best performance

## Next Steps

After verifying the triangle renders:
1. Try modifying shader colors in `shaders/triangle.vert`
2. Experiment with vertex positions
3. Add more complex geometry
4. Check out the GPU testing docs in `docs/testing/`
