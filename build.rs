use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell Cargo to rerun if shaders change
    println!("cargo:rerun-if-changed=shaders/triangle.vert");
    println!("cargo:rerun-if-changed=shaders/triangle.frag");

    let out_dir = env::var("OUT_DIR").unwrap();

    // Try to compile shaders if glslc is available
    let vertex_src = "shaders/triangle.vert";
    let fragment_src = "shaders/triangle.frag";
    let vertex_spv = Path::new(&out_dir).join("triangle.vert.spv");
    let fragment_spv = Path::new(&out_dir).join("triangle.frag.spv");

    // Check if glslc exists
    if let Ok(output) = std::process::Command::new("glslc")
        .arg("--version")
        .output()
    {
        if output.status.success() {
            // Compile vertex shader
            std::process::Command::new("glslc")
                .arg(vertex_src)
                .arg("-o")
                .arg(&vertex_spv)
                .status()
                .expect("Failed to compile vertex shader");

            // Compile fragment shader
            std::process::Command::new("glslc")
                .arg(fragment_src)
                .arg("-o")
                .arg(&fragment_spv)
                .status()
                .expect("Failed to compile fragment shader");

            println!("cargo:warning=Shaders compiled successfully");
        }
    } else {
        // If glslc is not available, copy pre-compiled shaders if they exist
        let precompiled_vert = "shaders/triangle.vert.spv";
        let precompiled_frag = "shaders/triangle.frag.spv";

        if Path::new(precompiled_vert).exists() && Path::new(precompiled_frag).exists() {
            fs::copy(precompiled_vert, &vertex_spv).ok();
            fs::copy(precompiled_frag, &fragment_spv).ok();
            println!("cargo:warning=Using pre-compiled shaders");
        } else {
            println!(
                "cargo:warning=No shader compiler found and no pre-compiled shaders available"
            );
            println!("cargo:warning=Shaders will need to be embedded manually");
        }
    }
}
