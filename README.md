# Rusty Renderer

[![CI](https://github.com/mpiispanen/rusty_renderer/actions/workflows/ci.yml/badge.svg)](https://github.com/mpiispanen/rusty_renderer/actions/workflows/ci.yml)

A multi-backend 3D renderer in Rust supporting Vulkan, DirectX 12, and wgpu.

This is a cross-platform graphics rendering sandbox developed in Rust to aid learning Rust and AI-driven development.

## Quick Start

```bash
# Install Vulkan runtime (see docs/RUNNING_LOCALLY.md for platform-specific instructions)
# Ubuntu/Debian example:
sudo apt install vulkan-tools libvulkan-dev

# Clone and run the render graph triangle example
git clone https://github.com/mpiispanen/rusty_renderer.git
cd rusty_renderer
cargo run --example render_graph_triangle --release
```

You should see a colorful RGB triangle rendered through the render graph! 🎨

## Features

- **Multi-backend support**: Vulkan (via vulkanalia), DirectX 12 (Windows), and wgpu
- **Modern architecture**: Render graph system for automatic dependency management
- **Cross-platform**: Linux, Windows, and macOS support
- **Well-tested**: Comprehensive unit and integration tests
- **CI/CD**: Automated builds, tests, and GPU validation
- **Validation layers**: Debug mode with validation on all backends (see [docs/VALIDATION_LAYERS.md](docs/VALIDATION_LAYERS.md))

## Building

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs))
- Platform-specific graphics drivers:
  - **Linux**: Vulkan drivers (usually included with GPU drivers)
  - **Windows**: DirectX 12 compatible GPU
  - **All platforms**: wgpu fallback available

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/mpiispanen/rusty_renderer.git
cd rusty_renderer

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run render graph triangle example
cargo run --example render_graph_triangle --release

# Run with detailed logging
RUST_LOG=debug cargo run --example render_graph_triangle

# Try different backends
cargo run --example render_graph_triangle wgpu
cargo run --example render_graph_triangle vulkan

# Run vertex buffer example
cargo run --example vertex_buffer_triangle --release
cargo run -- --backend wgpu
cargo run -- --backend directx  # Windows only
```

For detailed instructions and troubleshooting, see [docs/RUNNING_LOCALLY.md](docs/RUNNING_LOCALLY.md).

## Command-Line Options

```
Usage: rusty_renderer [OPTIONS]

Options:
  -b, --backend <BACKEND>    Graphics backend [default: vulkan] [possible values: vulkan, directx, wgpu]
  -s, --scene <SCENE>        Scene to render [default: triangle]
      --width <WIDTH>        Window width [default: 1280]
      --height <HEIGHT>      Window height [default: 720]
  -d, --debug                Enable debug mode and validation layers
      --vsync                Enable VSync [default: true]
      --log-level <LEVEL>    Log level [default: info] [possible values: off, error, warn, info, debug, trace]
      --max-frames <N>       Maximum frames to render (for testing)
  -h, --help                 Print help
  -V, --version              Print version
```

Note: `directx` backend is only available on Windows.

## Testing

### Unit and Integration Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Visual Regression Tests

Visual tests compare rendering outputs across backends using perceptual metrics:

```bash
# Run all visual tests
cargo test --test visual_tests -- --ignored --nocapture

# Test specific backend comparison
cargo test --test visual_tests test_vulkan_vs_wgpu -- --ignored --nocapture

# FLIP perceptual comparison tests
cargo test --test visual_tests flip -- --ignored --nocapture
```

#### FLIP Integration

The project uses [NVIDIA FLIP](https://research.nvidia.com/publication/2020-07_FLIP) for perceptual image comparison:

```bash
# Install FLIP evaluator
pip install flip-evaluator numpy pillow

# Direct comparison using Python script
python3 scripts/flip_compare.py reference.png test.png --error-map diff.png

# Batch comparison
./scripts/batch_flip_compare.sh reference_dir/ test_dir/ output_dir/
```

See [docs/FLIP_INTEGRATION.md](docs/FLIP_INTEGRATION.md) for detailed documentation.

## Development

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy

# Build documentation
cargo doc --open
```

### CI/CD

The project uses GitHub Actions for continuous integration. All commits are:
- Built in release mode
- Tested
- Linted with clippy
- Format-checked
- Documentation is verified

See `.github/workflows/ci.yml` for details.

For self-hosted runner setup (needed for GPU testing), see [docs/SELF_HOSTED_RUNNER.md](docs/SELF_HOSTED_RUNNER.md).

## Project Structure

```
rusty_renderer/
├── src/
│   ├── main.rs              # Application entry point
│   ├── app.rs               # Application framework and event loop
│   ├── config.rs            # Configuration and CLI parsing
│   ├── backends/            # Graphics backend abstractions
│   ├── render_graph/        # Render graph system
│   ├── scene/               # Scene management
│   ├── shaders/             # Shader management
│   ├── ui/                  # User interface
│   └── profiling/           # Performance profiling
├── tests/                   # Integration tests
├── shaders/                 # Shader source files
├── assets/                  # Test assets
└── docs/                    # Documentation
```

## Project Documentation

- **[CONTRIBUTING.md](CONTRIBUTING.md)** - How to contribute
- **[docs/WORKFLOW.md](docs/WORKFLOW.md)** - Development workflow and CI requirements
- [docs/DESIGN.md](docs/DESIGN.md) - Architecture and technical decisions
- [docs/MILESTONES.md](docs/MILESTONES.md) - Development roadmap
- [docs/SELF_HOSTED_RUNNER.md](docs/SELF_HOSTED_RUNNER.md) - CI/CD runner setup

## Milestones

- [x] **M1: Project Foundation** - Basic structure, CLI, CI/CD
- [ ] **M2: Backend Abstraction** - Stub implementations for all backends
- [ ] **M3: Vulkan Triangle** - First graphics output
- [ ] **M4: Multi-Backend Triangle** - DirectX and wgpu support
- [ ] **M5: Render Graph** - Advanced rendering pipeline

See [docs/MILESTONES.md](docs/MILESTONES.md) for detailed milestone plans.

## Acknowledgments

- [x] **M1: Project Foundation** - Basic structure, CLI, CI/CD
- [ ] **M2: Backend Abstraction** - Stub implementations for all backends
- [ ] **M3: Vulkan Triangle** - First graphics output
- [ ] **M4: Multi-Backend Triangle** - DirectX and wgpu support
- [ ] **M5: Render Graph** - Advanced rendering pipeline

See [docs/MILESTONES.md](docs/MILESTONES.md) for detailed milestone plans.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

This is primarily a learning project, but suggestions and feedback are welcome!

**Please read [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/WORKFLOW.md](docs/WORKFLOW.md) before contributing.**

Key points:
1. Run local checks before pushing (`cargo test`, `cargo clippy`, `cargo fmt`)
2. **Wait for CI to pass** before closing issues
3. Follow the workflow in [docs/WORKFLOW.md](docs/WORKFLOW.md)

Quick start:
1. Check existing issues or create a new one
2. Fork the repository
3. Make your changes following [CONTRIBUTING.md](CONTRIBUTING.md)
4. Run tests and formatting locally
5. Submit changes and **wait for CI** ✅
6. Address any CI failures

## Project Documentation

- **[CONTRIBUTING.md](CONTRIBUTING.md)** - How to contribute
- **[docs/WORKFLOW.md](docs/WORKFLOW.md)** - Development workflow and CI requirements
- [docs/DESIGN.md](docs/DESIGN.md) - Architecture and technical decisions
- [docs/MILESTONES.md](docs/MILESTONES.md) - Development roadmap
- [docs/SELF_HOSTED_RUNNER.md](docs/SELF_HOSTED_RUNNER.md) - CI/CD runner setup

- Built as a learning project for Rust and modern graphics programming
- Inspired by production renderers like Frostbite, Unreal Engine, and Unity
- Developed with AI assistance for accelerated learning
