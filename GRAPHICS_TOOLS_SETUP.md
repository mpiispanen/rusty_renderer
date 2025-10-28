# Graphics Development Tools Setup

## Required for Unified Shader Compilation

### DXC (DirectX Shader Compiler)
**Purpose:** Compile HLSL to SPIR-V and DXIL from a single source
**Install:**
```bash
rpm-ostree install DirectXShaderCompiler
# or if not available:
rpm-ostree install spirv-tools vulkan-tools mesa-vulkan-drivers
```

**Alternative (if DXC not in repos):**
Download from: https://github.com/microsoft/DirectXShaderCompiler/releases
- Get the Linux x64 build
- Extract to `/usr/local/bin` or `~/.local/bin`

## Graphics Tools Ecosystem

### Essential Tools (Install All)

```bash
rpm-ostree install \
    vulkan-tools \
    vulkan-loader \
    vulkan-validation-layers \
    mesa-vulkan-drivers \
    spirv-tools \
    glslang \
    shaderc \
    renderdoc \
    apitrace \
    gdb \
    valgrind \
    imagemagick \
    python3-pillow \
    python3-numpy
```

### Tool Descriptions

#### Shader Compilation
- **DXC**: HLSL → SPIR-V, DXIL (Microsoft official)
- **glslang**: GLSL/HLSL → SPIR-V (already have)
- **shaderc**: Google's GLSL → SPIR-V wrapper
- **spirv-tools**: SPIR-V optimizer, validator, disassembler (already have)

#### Debugging & Profiling
- **RenderDoc**: GPU frame debugger (essential!)
- **apitrace**: API call tracing (Vulkan/OpenGL)
- **gdb**: General debugging
- **valgrind**: Memory leak detection

#### Vulkan Development
- **vulkan-tools**: vkvia, vulkaninfo
- **vulkan-loader**: Vulkan runtime
- **vulkan-validation-layers**: Debug validation (already have)
- **mesa-vulkan-drivers**: Software/hardware drivers

#### Image Analysis
- **ImageMagick**: Image comparison, manipulation
- **python3-pillow**: Python image processing
- **python3-numpy**: Numerical analysis

## Python Packages for Visual Testing

```bash
pip install --user \
    flip-evaluator \
    numpy \
    pillow \
    matplotlib
```

## Usage

### Single-Source HLSL Compilation

**With DXC (recommended):**
```bash
# For Vulkan (SPIR-V)
dxc -spirv -T vs_6_0 -E VSMain forward.hlsl -Fo forward.vert.spv

# For DirectX (DXIL)
dxc -T vs_6_0 -E VSMain forward.hlsl -Fo forward_vs.dxil
```

**With glslang (current):**
```bash
# HLSL → SPIR-V for Vulkan
glslangValidator -V -D -e VSMain --hlsl-iomap -S vert forward.hlsl -o forward.vert.spv
```

### Offline vs Online Compilation

**Offline (build time):**
- Compile during `cargo build`
- Commit SPIR-V/DXIL to repo
- Faster runtime, larger repo

**Online (runtime):**
- DirectX: Use D3DCompile API
- Vulkan: Load pre-compiled SPIR-V
- Slower startup, smaller repo

**Hybrid (recommended):**
- Pre-compile during build
- Embed bytecode as fallback
- Support runtime reload for development

## Recommended Installation Command

```bash
# Single command to install everything
rpm-ostree install \
    vulkan-tools vulkan-loader vulkan-validation-layers \
    mesa-vulkan-drivers spirv-tools glslang shaderc \
    renderdoc apitrace imagemagick \
    python3-pillow python3-numpy

# Then reboot
systemctl reboot

# After reboot, install Python packages
pip install --user flip-evaluator matplotlib
```

## Verify Installation

```bash
# Check shader compilers
dxc --version
glslangValidator --version
spirv-val --version

# Check Vulkan
vulkaninfo | head -20

# Check debugging tools
renderdoc --version
apitrace --help

# Check image tools
convert --version
python3 -c "import flip_evaluator; print('FLIP OK')"
```

## Why These Tools?

### DXC
- Official Microsoft compiler
- Best HLSL support (SM 6.x)
- Single source for Vulkan + DirectX
- Supports Vulkan extensions via attributes

### RenderDoc
- Industry standard GPU debugger
- Frame capture and analysis
- Shader debugging
- Performance profiling

### apitrace
- Records API calls
- Replay captures
- Compare renders
- Find API misuse

### ImageMagick
- Automated image comparison
- Generate diff images
- Format conversion
- Batch processing

## Next Steps After Installation

1. Test DXC: `dxc --version`
2. Update build.rs to use DXC
3. Write single forward.hlsl for both backends
4. Remove forward_vk.hlsl (no longer needed)
5. Test rendering parity
