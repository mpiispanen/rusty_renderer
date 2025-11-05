# Camera System Testing

This document describes how to test the camera implementation in rusty_renderer.

## Overview

The camera system supports two modes:
- **Perspective Camera**: Fixed look-at camera with position, target, and up vector
- **FreeFly Camera**: Interactive first-person camera with yaw/pitch control

## Automated Tests

### Quick Test
```bash
./test_camera.sh
```

This runs automated tests for both camera modes and generates screenshots:
- `camera_test_freefly_vulkan.png` - FreeFly camera view
- `camera_test_perspective_vulkan.png` - Perspective camera view

## Interactive Testing

### Launch Interactive Mode
```bash
./test_camera_interactive.sh
```

### Controls
- **W/S** - Move forward/backward
- **A/D** - Strafe left/right
- **Q/E** - Move down/up
- **Mouse** - Look around (camera rotation)
- **Shift** - Move faster (hold while moving)
- **ESC** - Exit application

### What to Test

1. **Movement**
   - Move forward/backward and verify the scene responds correctly
   - Strafe left/right
   - Move up/down

2. **Camera Rotation**
   - Move mouse to rotate camera
   - Verify pitch is clamped (can't flip upside down)
   - Check yaw rotates smoothly

3. **Scene Rendering**
   - Verify the ground plane renders correctly
   - Check the cube is visible and properly lit
   - Shadows should be visible on the ground

## Camera Implementation Details

### Key Features
- **Backend-Aware**: Automatically handles Vulkan vs DirectX coordinate system differences
- **Dynamic Matrices**: View and projection matrices calculated per-frame
- **Push Constants**: Camera uniforms sent via push constants for efficiency
- **Thread-Local State**: Camera uniforms accessible globally within render passes

### Files
- `src/camera/mod.rs` - Core camera math and backend handling
- `src/camera/controller.rs` - CameraController for interactive control
- `scenes/camera_test.toml` - Test scene with FreeFly camera
- `scenes/shadow_test.toml` - Test scene with Perspective camera

### Adding Camera to New Scene

Example FreeFly camera:
```toml
[camera]
type = "free_fly"
position = [0.0, 2.0, 5.0]
yaw = -90.0
pitch = -15.0
fov = 60.0
```

Example Perspective camera:
```toml
[camera]
type = "perspective"
position = [3.0, 3.0, 5.0]
target = [0.0, 0.0, 0.0]
fov = 60.0
near = 0.1
far = 100.0
```

## Expected Results

### FreeFly Camera Test
- Should show a green ground plane and red cube from an elevated position
- Camera looking down at ~15 degree angle
- Cube should be rotated 45 degrees

### Perspective Camera Test
- Should show the shadow test scene from an angled view
- Cube hovering above gray ground plane
- Shadow visible on ground

## Troubleshooting

### Scene appears upside down
- Check backend is set correctly (Vulkan vs DirectX)
- Verify `camera::set_camera_backend()` is called before creating camera

### Camera movement not working
- Ensure window has focus
- Check that input events are being processed
- Verify CameraController is being updated each frame

### Matrices appear incorrect
- Check debug output with `RUST_LOG=info`
- Look for "CAMERA MATRIX DEBUG" messages
- Verify view-projection matrix is not NaN

## Performance Notes

- Camera matrix calculation is O(1) per frame
- Push constants avoid buffer updates
- Thread-local storage minimizes overhead

## Next Steps

- [ ] Add camera animation/paths
- [ ] Implement orbit camera mode
- [ ] Add camera FOV adjustment controls
- [ ] Support multiple cameras per scene
