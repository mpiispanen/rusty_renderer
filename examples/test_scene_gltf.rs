//! Test scene loader with GLTF
//!
//! Test loading a scene that references a GLTF model.

use anyhow::Result;
use rusty_renderer::scene::SceneLoader;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== Scene Loader GLTF Test ===\n");

    let scene_path = "scenes/gltf_test.toml";
    
    println!("Loading scene: {}", scene_path);
    
    let loader = SceneLoader::new()?;
    let scene = loader.load_from_file(scene_path)?;
    
    println!("\n✓ Scene loaded successfully!");
    println!("\nMetadata:");
    println!("  Name: {}", scene.metadata.name);
    println!("  Description: {}", scene.metadata.description);
    println!("  Author: {}", scene.metadata.author);
    
    println!("\nMaterials: {}", scene.materials.len());
    for (i, mat) in scene.materials.iter().enumerate() {
        println!("  [{}] {}", i, mat.name);
        println!("      Base color: {:?}", mat.base_color);
        println!("      Metallic: {}", mat.metallic);
        println!("      Roughness: {}", mat.roughness);
        println!("      Texture: {:?}", mat.diffuse_texture);
    }
    
    println!("\nObjects: {}", scene.objects.len());
    for obj in &scene.objects {
        if let rusty_renderer::scene::SceneObject::Mesh { name, geometry, material, transform } = obj {
            let vertex_count = match geometry {
                rusty_renderer::scene::GeometryData::Inline { vertices, .. } => vertices.len(),
                _ => 0,
            };
            println!("  Mesh '{}':", name);
            println!("    Vertices: {}", vertex_count);
            println!("    Material: {:?}", material);
            println!("    Position: {:?}", transform.position);
            println!("    Rotation: {:?}", transform.rotation);
            println!("    Scale: {:?}", transform.scale);
        }
    }
    
    println!("\nCamera: {:?}", scene.camera);
    
    if let Some(lighting) = &scene.lighting {
        println!("\nLighting:");
        println!("  Ambient: {:?}", lighting.ambient);
        println!("  Lights: {}", lighting.lights.len());
        for (i, light) in lighting.lights.iter().enumerate() {
            println!("  [{}] {:?}", i, light);
        }
    } else {
        println!("\nNo lighting configured");
    }
    
    println!("\n✓ All checks passed!");
    
    Ok(())
}
