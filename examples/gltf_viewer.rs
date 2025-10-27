//! GLTF Model Viewer
//!
//! A comprehensive example demonstrating:
//! - GLTF model loading from any path
//! - Asset path resolution (relative/absolute)
//! - Material and texture handling
//! - Multi-backend rendering (Vulkan/wgpu/DX12)
//!
//! Usage:
//!   cargo run --example gltf_viewer [BACKEND] <MODEL_PATH>
//!   cargo run --example gltf_viewer vulkan assets/models/textured_cube.gltf
//!   cargo run --example gltf_viewer wgpu scenes/gltf_textured.toml

use anyhow::Result;
use rusty_renderer::{
    backends::{create_backend, BackendType},
    pipelines::{ForwardPipeline, RenderPipeline},
    resources::AssetPathResolver,
    scene::{Scene, SceneLoader},
};
use std::path::Path;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== GLTF Model Viewer ===\n");

    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    let mut backend_type = BackendType::Vulkan;
    let mut model_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "vulkan" => backend_type = BackendType::Vulkan,
            "directx" | "dx12" => backend_type = BackendType::DirectX12,
            "--help" | "-h" => {
                print_usage();
                return Ok(());
            }
            arg => {
                if arg.starts_with('-') {
                    eprintln!("Unknown option: {arg}");
                    print_usage();
                    std::process::exit(1);
                }
                model_path = Some(arg.to_string());
            }
        }
        i += 1;
    }

    let model_path = model_path.unwrap_or_else(|| {
        println!("No model specified, using default textured cube");
        "scenes/gltf_textured.toml".to_string()
    });

    println!("Backend: {backend_type:?}");
    println!("Model: {model_path}\n");

    // Load scene
    let scene = load_model(&model_path)?;

    println!("\n✓ Model loaded successfully!");
    print_scene_info(&scene);

    // Create backend
    println!("\nInitializing {backend_type} backend...");
    let mut backend = create_backend(backend_type, false)?;
    backend.initialize_headless(800, 600)?;
    println!("✓ Backend initialized (800x600)");

    // Create pipeline and build render graph
    println!("\nBuilding render graph...");
    let mut pipeline = ForwardPipeline::new();
    let mut graph = pipeline.build_graph(&scene, &mut *backend)?;
    println!("✓ Render graph built");

    // Compile render graph
    println!("\nCompiling render graph...");
    let compiled = graph.compile()?;
    println!(
        "✓ Render graph compiled: {} passes",
        compiled.execution_order.len()
    );

    // Execute one frame
    println!("\nRendering...");
    backend.begin_frame()?;
    backend.execute_graph(&graph, &compiled)?;
    backend.end_frame()?;
    println!("✓ Frame rendered successfully");

    // Capture and save output
    println!("\nCapturing frame...");
    let (width, height, pixels) = backend.capture_frame()?;
    println!("Captured: {width}x{height}");

    let output_path = generate_output_filename(&model_path, backend_type);
    image::save_buffer(
        &output_path,
        &pixels,
        width,
        height,
        image::ColorType::Rgba8,
    )?;
    println!("✓ Saved to: {output_path}");

    backend.cleanup();

    println!("\n=== Rendering Complete ===");
    println!("Output saved to: {output_path}");

    Ok(())
}

fn load_model(path: &str) -> Result<Scene> {
    let _path_obj = Path::new(path);

    // Check if it's a TOML scene file or a direct GLTF file
    if path.ends_with(".toml") {
        println!("Loading scene from TOML: {path}");
        let loader = SceneLoader::new()?;
        loader.load_from_file(path)
    } else if path.ends_with(".gltf") || path.ends_with(".glb") {
        println!("Loading GLTF model directly: {path}");
        load_gltf_direct(path)
    } else {
        anyhow::bail!("Unsupported file format: {path}. Expected .toml, .gltf, or .glb")
    }
}

fn load_gltf_direct(path: &str) -> Result<Scene> {
    use rusty_renderer::resources::GltfLoader;
    use rusty_renderer::scene::{Camera, Light, Lighting};

    // Resolve path
    let resolver = AssetPathResolver::new()?;
    let model_dir = Path::new(path).parent();
    let resolved_path = resolver.resolve_and_verify(path, model_dir)?;

    println!("Resolved path: {}", resolved_path.display());

    // Load GLTF
    let (objects, materials, metadata) = GltfLoader::load(&resolved_path)?;

    // Create default camera and lighting
    let camera = Camera::Perspective {
        position: [2.0, 2.0, 3.0],
        target: [0.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        fov: 45.0,
        near: 0.1,
        far: 100.0,
    };

    let lighting = Lighting {
        ambient: [0.2, 0.2, 0.2],
        lights: vec![
            Light::Directional {
                direction: [-0.3, -1.0, -0.5],
                color: [1.0, 1.0, 1.0],
                intensity: 0.8,
            },
            Light::Point {
                position: [1.5, 1.0, 2.0],
                color: [1.0, 0.7, 0.3],
                intensity: 1.2,
            },
        ],
    };

    Ok(Scene {
        metadata,
        objects,
        materials,
        camera,
        lighting: Some(lighting),
    })
}

fn print_scene_info(scene: &Scene) {
    println!("\nScene Information:");
    println!("  Name: {}", scene.metadata.name);
    println!("  Description: {}", scene.metadata.description);
    println!("  Objects: {}", scene.objects.len());
    println!("  Materials: {}", scene.materials.len());

    if scene.objects.len() <= 5 {
        println!("\n  Object details:");
        for (i, obj) in scene.objects.iter().enumerate() {
            if let rusty_renderer::scene::SceneObject::Mesh { name, geometry, .. } = obj {
                let vertex_count = match geometry {
                    rusty_renderer::scene::GeometryData::Inline { vertices, .. } => vertices.len(),
                    _ => 0,
                };
                println!("    [{i}] {name} ({vertex_count} vertices)");
            }
        }
    }

    if scene.materials.len() <= 10 {
        println!("\n  Materials:");
        for (i, mat) in scene.materials.iter().enumerate() {
            let has_texture = if mat.diffuse_texture.is_some() {
                "✓"
            } else {
                "✗"
            };
            println!("    [{}] {} [texture: {}]", i, mat.name, has_texture);
        }
    }
}

fn generate_output_filename(input_path: &str, backend: BackendType) -> String {
    let path = Path::new(input_path);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let backend_name = match backend {
        BackendType::Vulkan => "vulkan",
        BackendType::DirectX12 => "dx12",
    };

    format!("{stem}_{backend_name}.png")
}

fn print_usage() {
    println!("GLTF Model Viewer");
    println!();
    println!("Usage:");
    println!("  gltf_viewer [BACKEND] <MODEL_PATH>");
    println!();
    println!("Backends:");
    println!("  vulkan     Use Vulkan backend (default)");
    println!("  wgpu       Use wgpu backend");
    println!("  dx12       Use DirectX 12 backend (Windows only)");
    println!();
    println!("Model Path:");
    println!("  Can be a .toml scene file or a direct .gltf/.glb model file");
    println!();
    println!("Examples:");
    println!("  gltf_viewer vulkan assets/models/textured_cube.gltf");
    println!("  gltf_viewer wgpu scenes/gltf_textured.toml");
    println!("  gltf_viewer assets/models/cube.gltf");
}
