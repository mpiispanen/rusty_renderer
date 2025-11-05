# Camera System - Quick Usage Guide

## Running with Different Camera Modes

### FreeFly Camera (Interactive)
```bash
cargo run --release -- --scene camera_test
```
Controls: WASD (move), QE (up/down), Mouse (look), Shift (faster)

### Perspective Camera (Fixed View)
```bash
cargo run --release -- --scene shadow_test
```

## Headless Testing (Screenshots)
```bash
cargo run --release -- --scene camera_test --headless --screenshot output.png
```

## Creating a Scene with Camera

### FreeFly Camera Scene
```toml
[camera]
type = "free_fly"
position = [0.0, 2.0, 5.0]  # Starting position
yaw = -90.0                  # Initial horizontal rotation (degrees)
pitch = -15.0                # Initial vertical rotation (degrees)
fov = 60.0                   # Field of view (degrees)
```

### Perspective Camera Scene
```toml
[camera]
type = "perspective"
position = [3.0, 3.0, 5.0]  # Camera position
target = [0.0, 0.0, 0.0]    # Look-at point
fov = 60.0                   # Field of view (degrees)
near = 0.1                   # Near clip plane
far = 100.0                  # Far clip plane
```

## Interactive Controls

| Key | Action |
|-----|--------|
| W | Move forward |
| S | Move backward |
| A | Move left |
| D | Move right |
| Q | Move down |
| E | Move up |
| Shift | Move faster (hold) |
| Mouse | Look around |
| ESC | Exit |

## Test Scripts

### Automated Tests
```bash
./test_camera.sh           # Run both camera mode tests
```

### Interactive Test
```bash
./test_camera_interactive.sh   # Launch interactive test
```

## Camera Implementation Details

- **Backend Support**: Vulkan and DirectX 12
- **Coordinate System**: Right-handed for both backends
- **Matrix Updates**: Per-frame via push constants
- **Input Handling**: Integrated with winit event loop
- **Movement Speed**: 5 units/second (10 with Shift)

## Verifying Camera Works

After running a test, check that:
1. Scene renders correctly from camera position
2. Movement controls respond smoothly
3. Mouse look rotates view appropriately
4. Screenshots capture the correct view

## Troubleshooting

**Scene appears incorrect**: Check camera position and target/yaw values
**Controls don't work**: Ensure window has focus, not in headless mode
**Upside down rendering**: Verify backend is set correctly

## Documentation

- Full implementation details: `docs/CAMERA_IMPLEMENTATION.md`
- Testing guide: `docs/CAMERA_TESTING.md`
- Code: `src/camera/mod.rs` and `src/camera/controller.rs`
