# Golden Reference Images - gltf_textured Scene

This directory contains golden reference images for the `gltf_textured` scene.

## Purpose

These images serve as the "known good" baseline for visual regression testing in CI.

## How to Update

Run the update script:
```bash
./scripts/update_golden_references.sh --vulkan
```

Or manually render and copy:
```bash
cargo run --release -- --scene scenes/gltf_textured.toml --backend vulkan --pipeline forward --headless --screenshot temp.png
cp temp.png references/gltf_textured/gltf_textured_vulkan.png
```

Then visually verify and commit if correct.
