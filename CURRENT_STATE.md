# Rusty Renderer - Current State (November 2025)

**Last Updated:** 2025-11-03  
**Version:** 0.8.0  
**Status:** Render Graph Driven Architecture - In Progress

## Current Status

### ✅ Working Features

**Both Vulkan and DirectX backends:**
- Forward lighting via the render-graph driven `ForwardSimplePass`
- Scene-driven geometry (triangle, cube, glTF models) defined in TOML
- Depth testing, backface culling, and screenshot capture
- Headless and windowed execution paths
- Cross-compilation for Windows plus Proton-based DirectX validation on Linux

### ⚠️ Known Issues

**Rendering:**
- Vulkan: Rendering cube geometry correctly
- DirectX: Some synchronization warnings (vkd3d-proton)
- Minor backface culling differences between backends

**Architecture:**
- **Hardcoded shader paths** - Render passes still reference pre-built shader binaries directly
- **Hardcoded pipeline state** - Depth, culling, blending not yet configurable per-scene
- **Hardcoded resource bindings** - Descriptor layouts fixed in backends
- **Legacy render graph gaps** - Index buffers and material binding still manual

**CI/CD:**
- Visual regression tests not yet automated
- Backend parity validation manual

## Current Focus: Render Graph Driven Architecture

We are transitioning from hardcoded rendering to a fully data-driven render graph system where:
1. **Render passes** declare their resource requirements and shader needs
2. **Render graph** manages all resource allocation and lifetime
3. **Pipeline configuration** comes from render pass definitions, not hardcoded
4. **Shaders** are referenced by render passes, not backends

### Immediate Goals

1. **Issue #88 – Remove legacy hardcoded code**
   - Finish cleaning `app.rs` now that the legacy pipeline/application stack is gone
   - Move remaining shader references into render-pass builders
   - Tighten vertex/index buffer handling so the graph stays declarative

2. **Shadow Mapping Prep**
   - Define interfaces for depth-only shadow passes
   - Extend forward pass to consume shadow resources
   - Plan tone mapping/post effects once shadows land

3. **Automation & CI**
   - Stand up headless regression captures for Vulkan + DirectX
   - Track parity deltas automatically
   - Publish lightweight dashboards instead of sprawling status docs

## Next Steps

### Code Cleanup (in progress)
- [ ] Finish shader registration migration into `ForwardSimplePass`
- [ ] Replace manual vertex expansion with index buffer support
- [ ] Remove unused resource helpers leftover from pipeline era

### Shadow Mapping (next milestone)
- [ ] Shadow map render pass & resource descriptors
- [ ] Forward shading with shadow sampling
- [ ] Tone mapping and simple PCF

### CI & Tooling (ongoing)
- [ ] Re-enable automated screenshot comparisons
- [ ] Capture Proton-based DirectX runs in CI
- [ ] Publish consolidated status updates (single source of truth)

## Build Instructions

### Vulkan (Linux/Bazzite)
```bash
cargo run --release -- --backend vulkan --scene cube
```

### DirectX via Proton (Linux/Bazzite)
```bash
./run_with_proton.sh cube
```

### DirectX Native (Windows)
```bash
cargo run --release -- --backend directx --scene cube
```

## Development Workflow

1. **Make changes** to code
2. **Run local checks:**
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   ```
3. **Test rendering:**
   ```bash
   cargo run --release -- --backend vulkan --scene cube
   ./run_with_proton.sh cube  # Test DirectX via Proton
   ```
4. **Commit and push**
5. **Wait for CI to pass** ✅

## Documentation Structure

- **[README.md](README.md)** - Project overview and quick start
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - How to contribute
- **[docs/DESIGN.md](docs/DESIGN.md)** - Architecture and design decisions
- **[docs/WORKFLOW.md](docs/WORKFLOW.md)** - Development workflow
- **[docs/BAZZITE_SETUP.md](docs/BAZZITE_SETUP.md)** - Bazzite-specific setup
- **[docs/TESTING_DIRECTX_ON_LINUX.md](docs/TESTING_DIRECTX_ON_LINUX.md)** - DirectX via Proton
- **[ROADMAP.md](ROADMAP.md)** - Development roadmap
