# Testing Notes - Coordinate System Fix

## Changes Made
1. Modified `src/camera/mod.rs` to use consistent right-handed coordinates for both backends
2. Added Y-flip in projection matrix for DirectX to match NDC conventions
3. Both `look_at_view()` and `free_fly_view()` now always use `Mat4::look_at_rh`

## Testing Commands

### Vulkan Backend
```bash
cargo run --release -- --scene scenes/gltf_textured.toml --pipeline forward
```

### DirectX Backend (via Proton)
```bash
./run_with_proton.sh
# Or with explicit arguments:
./run_with_proton.sh --scene scenes/gltf_textured.toml --pipeline forward
```

## Expected Results
- Both backends should render the textured cube with the same orientation
- The cube should show the checkerboard texture correctly
- No upside-down or mirrored rendering

## Build Requirements
- Vulkan: Standard Linux build (`cargo build --release`)
- DirectX: Windows target build (`cargo build --release --target x86_64-pc-windows-msvc`)

## Notes
- DirectX shaders already have UV Y-flip at line 58 of `shaders/hlsl/forward.hlsl`
- This is correct and needed because texture coordinates also have different conventions
- The coordinate system fix is independent of the UV coordinate handling
