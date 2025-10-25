# Quick Start - Rusty Renderer

**Last Updated**: 2025-10-25

---

## 🚀 Quick Test (5 seconds)

```bash
cd /var/home/matpii01/rusty_renderer

# Vulkan (native Linux)
cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml

# DirectX (via Proton)
./run_with_proton.sh scenes/gltf_textured.toml
```

---

## 🎨 Render a Scene

### Windowed Mode (Interactive)
```bash
cargo run --release -- \
  --backend vulkan \
  --scene scenes/gltf_textured.toml
```

### Headless Mode (Screenshot)
```bash
cargo run --release -- \
  --backend vulkan \
  --scene scenes/gltf_textured.toml \
  --headless \
  --screenshot output.png \
  --max-frames 1
```

---

## 🔨 Build Commands

### Linux (Vulkan)
```bash
cargo build --release
```

### Windows (DirectX) - Cross-compile
```bash
cargo build --release --target x86_64-pc-windows-msvc
```

---

## 🧪 Test DirectX with Proton

### One-time Setup
```bash
# Target already installed, just build
cargo build --release --target x86_64-pc-windows-msvc

# Setup test directory
mkdir -p windows_test_directx
cp target/x86_64-pc-windows-msvc/release/rusty_renderer.exe windows_test_directx/
cp -r assets scenes windows_test_directx/
```

### Run Tests
```bash
# GLTF scene (recommended)
./run_with_proton.sh scenes/gltf_textured.toml

# Triangle
./run_with_proton.sh scenes/triangle.toml

# Textured cube
./run_with_proton.sh scenes/textured_cube.toml

# With debug output
./run_with_proton.sh scenes/gltf_textured.toml info
```

---

## 📁 Available Scenes

| Scene | Description | Best For |
|-------|-------------|----------|
| `scenes/triangle.toml` | Simple triangle | Basic test |
| `scenes/textured_cube.toml` | Lit textured cube | Material testing |
| `scenes/gltf_textured.toml` | GLTF with texture | Full pipeline test |
| `scenes/cube.toml` | Lit cube | Lighting test |

---

## 📊 Current Status

**What Works**:
- ✅ Vulkan backend (Linux)
- ✅ DirectX backend (Windows + Proton)
- ✅ GLTF loading
- ✅ Forward rendering
- ✅ Blinn-Phong lighting
- ✅ Textured materials
- ✅ Multiple lights
- ✅ Windowed + headless modes

**What's Deferred**:
- ⏸️ wgpu backend (bind group issues)

**What's Next**:
- 🎯 Shadow mapping (recommended)
- 📦 Complex GLTF scenes
- ⚡ Performance benchmarks

---

## 🐛 Troubleshooting

### Vulkan Validation Errors
```bash
# Should see: Zero validation errors
cargo run --release -- --backend vulkan --scene scenes/triangle.toml
```

### Proton Not Found
```bash
# Check available versions
ls -1 "$HOME/.local/share/Steam/steamapps/common/" | grep -i proton

# Update PROTON_DIR in run_with_proton.sh
```

### Binary Not Found (DirectX)
```bash
# Rebuild and copy
cargo build --release --target x86_64-pc-windows-msvc
cp target/x86_64-pc-windows-msvc/release/rusty_renderer.exe windows_test_directx/
```

---

## 📚 Documentation

**Status Documents**:
- `PROJECT_STATUS_CURRENT_2025-10-25.md` - Overall status
- `GLTF_IMPLEMENTATION_COMPLETE.md` - GLTF system
- `DIRECTX_PROTON_VERIFIED_2025-10-25.md` - DirectX testing

**How-To Guides**:
- `PROTON_HOWTO.md` - Proton testing guide
- `docs/ASSETS.md` - Asset system

**Session Logs**:
- `SESSION_DIRECTX_PROTON_TESTING_2025-10-25.md` - This session

---

## 🎯 Next Steps

### Start Shadow Mapping (Recommended)
1. Read up on shadow mapping techniques
2. Create shadow pass in render graph
3. Implement depth-only rendering
4. Add shadow texture sampling to forward pass
5. Test with directional light

**Estimated Time**: 10-12 hours  
**Difficulty**: Moderate  
**Impact**: High (dramatic visual improvement)

### Alternative: Test Complex GLTF
1. Download Khronos GLTF sample models
2. Add to `assets/models/gltf_samples/`
3. Create test scenes
4. Identify and fix edge cases

**Estimated Time**: 4-6 hours  
**Difficulty**: Easy to Moderate  
**Impact**: Medium (robustness)

---

## 💡 Tips

**Fast Iteration**:
```bash
# Keep cargo watch running
cargo watch -x 'run --release -- --backend vulkan --scene scenes/triangle.toml'
```

**Compare Backends**:
```bash
# Vulkan
cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml

# DirectX via Proton
./run_with_proton.sh scenes/gltf_textured.toml

# Should produce identical results!
```

**Debug Rendering**:
```bash
# Enable Vulkan validation
cargo run -- --backend vulkan --scene scenes/triangle.toml

# VKD3D debug output
./run_with_proton.sh scenes/triangle.toml info
```

---

## ✅ Quality Checklist

Before committing changes:
- [ ] `cargo test --lib` - All tests pass
- [ ] `cargo clippy` - No warnings
- [ ] `cargo fmt` - Code formatted
- [ ] Test Vulkan rendering - No validation errors
- [ ] Test DirectX via Proton - Exit code 0
- [ ] Update documentation
- [ ] Update PROJECT_STATUS document

---

**Ready to build more features!** 🚀
