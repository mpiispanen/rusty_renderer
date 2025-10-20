//! Test scene loading
//!
//! Simple example to test loading scenes from TOML files

use anyhow::Result;
use rusty_renderer::scene::SceneLoader;

fn main() -> Result<()> {
    println!("=== Scene Loading Test ===\n");

    // Load triangle scene
    let scene = SceneLoader::load_from_file("scenes/triangle.toml")?;
    println!("✓ Loaded scene: {}", scene.metadata.name);
    println!("  Description: {}", scene.metadata.description);
    println!("  Objects: {}", scene.objects.len());
    println!("  Camera: {:?}", scene.camera);
    println!();

    // Load quad scene
    let scene = SceneLoader::load_from_file("scenes/quad.toml")?;
    println!("✓ Loaded scene: {}", scene.metadata.name);
    println!("  Description: {}", scene.metadata.description);
    println!("  Objects: {}", scene.objects.len());
    println!();

    // List all scenes
    let scenes = SceneLoader::list_scenes("scenes")?;
    println!("✓ Available scenes:");
    for scene_name in scenes {
        println!("  - {scene_name}");
    }

    println!("\n=== Scene loading works! ===");
    Ok(())
}
