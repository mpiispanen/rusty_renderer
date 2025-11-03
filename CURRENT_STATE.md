# Rusty Renderer - Current State (November 2025)

**Last Updated:** 2025-11-03  
**Version:** 0.8.0  
**Status:** Render Graph Driven Architecture - In Progress

## Current Status

### ✅ Working Features

**Both Vulkan and DirectX backends:**
- Basic forward rendering with Blinn-Phong lighting
- Vertex color rendering (triangle pass)
- Scene loading from TOML files
- GLTF model loading with textures
- Depth testing and backface culling
- Headless and windowed modes
- Screenshot capture
- Cross-compilation for Windows
- DirectX testing via Proton on Linux

### ⚠️ Known Issues

**Rendering:**
- Vulkan: Rendering cube geometry correctly
- DirectX: Some synchronization warnings (vkd3d-proton)
- Minor backface culling differences between backends

**Architecture:**
- **Hardcoded shader paths** - Shaders referenced by fixed file paths in backend code
- **Hardcoded pipeline state** - Depth, culling, blending not configurable
- **Hardcoded resource bindings** - Descriptor layouts fixed in backends
- **Legacy code** - Some unused paths and variables remain from refactoring
- **Lights hardcoded in app** - Should come from scene files

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

1. **Remove legacy/hardcoded code**
   - Clean up unused variables and paths
   - Remove hardcoded vertex data in app.rs
   - Move shader registration to render passes
   - Extract lights from app to scene files

2. **Complete render graph resource management**
   - Automatic resource allocation
   - Proper resource upload and initialization
   - Pipeline compilation from pass definitions

3. **Add shadow mapping**
   - Shadow map render pass
   - Forward rendering with shadows
   - Tone mapping post-process

4. **Make pipeline configurable**
   - Runtime pass enable/disable
   - Debug UI for pipeline configuration

## Next Steps

### Phase 1: Code Cleanup (THIS WEEK)
- [ ] Remove all hardcoded vertices in app.rs
- [ ] Move shader registration to render passes
- [ ] Extract lights to scene files
- [ ] Remove unused/legacy code paths
- [ ] Update documentation to reflect current state

### Phase 2: Shadow Mapping (THIS WEEK)
- [ ] Implement shadow map render pass
- [ ] Update forward pass to use shadow maps
- [ ] Add tone mapping post-process pass

### Phase 3: CI and Testing (ONGOING)
- [ ] Fix CI rendering tests
- [ ] Automated backend parity validation
- [ ] Reference image management

### Phase 4: Debug UI (NEXT SPRINT)
- [ ] Pipeline configuration UI
- [ ] Pass enable/disable
- [ ] Real-time shader reload

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

