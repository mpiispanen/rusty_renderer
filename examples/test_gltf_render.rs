//! Render a GLTF model using the forward pipeline
//!
//! This demonstrates end-to-end GLTF loading and rendering.

use anyhow::Result;
use rusty_renderer::{
    backends::{create_backend, BackendType},
    pipelines::{ForwardPipeline, RenderPipeline},
    scene::SceneLoader,
};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("=== GLTF Forward Rendering Test ===\n");

    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    let mut backend_type = BackendType::Vulkan;
    let mut scene_file = "scenes/gltf_test.toml";

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "vulkan" => backend_type = BackendType::Vulkan,
            "wgpu" => backend_type = BackendType::Wgpu,
            "directx" | "dx12" => backend_type = BackendType::DirectX12,
            arg if !arg.starts_with('-') => scene_file = arg,
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    println!("Backend: {:?}", backend_type);
    println!("Scene: {}\n", scene_file);

    // Load scene
    println!("Loading scene...");
    let loader = SceneLoader::new()?;
    let scene = loader.load_from_file(scene_file)?;
    println!("✓ Scene loaded: {} objects", scene.objects.len());

    // Create backend
    println!("\nInitializing {} backend...", backend_type);
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
    println!("✓ Render graph compiled: {} passes", compiled.execution_order.len());

    // Execute one frame
    println!("\nExecuting render frame...");
    backend.begin_frame()?;
    backend.execute_graph(&graph, &compiled)?;
    backend.end_frame()?;
    println!("✓ Frame rendered successfully");

    // Capture output
    println!("\nCapturing frame...");
    let (width, height, pixels) = backend.capture_frame()?;
    println!("Captured frame: {}x{}", width, height);

    let output_path = "gltf_test.png";
    image::save_buffer(output_path, &pixels, width, height, image::ColorType::Rgba8)?;
    println!("Saved to: {}", output_path);

    backend.cleanup();

    println!("\n=== GLTF Rendering Test Complete ===");
    println!("✓ All tests passed!");

    Ok(())
}
