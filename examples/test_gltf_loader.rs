//! Test GLTF loader
//!
//! Simple test to verify GLTF loading works correctly.

use anyhow::Result;
use rusty_renderer::resources::GltfLoader;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== GLTF Loader Test ===\n");

    let args: Vec<String> = std::env::args().collect();
    let gltf_path = if args.len() > 1 {
        &args[1]
    } else {
        "assets/models/cube.gltf"
    };
    
    println!("Loading: {}", gltf_path);
    
    let (objects, materials, metadata) = GltfLoader::load(gltf_path)?;
    
    println!("\n✓ GLTF loaded successfully!");
    println!("\nMetadata:");
    println!("  Name: {}", metadata.name);
    println!("  Description: {}", metadata.description);
    println!("  Author: {}", metadata.author);
    
    println!("\nMaterials: {}", materials.len());
    for (i, mat) in materials.iter().enumerate() {
        println!("  [{}] {}", i, mat.name);
        println!("      Base color: {:?}", mat.base_color);
        println!("      Metallic: {}", mat.metallic);
        println!("      Roughness: {}", mat.roughness);
        println!("      Texture: {:?}", mat.diffuse_texture);
    }
    
    println!("\nObjects: {}", objects.len());
    for obj in &objects {
        if let rusty_renderer::scene::SceneObject::Mesh { name, geometry, material, .. } = obj {
            let vertex_count = match geometry {
                rusty_renderer::scene::GeometryData::Inline { vertices, .. } => vertices.len(),
                _ => 0,
            };
            println!("  Mesh '{}': {} vertices, material index: {:?}", 
                name, vertex_count, material);
        }
    }
    
    println!("\n✓ All checks passed!");
    
    Ok(())
}
