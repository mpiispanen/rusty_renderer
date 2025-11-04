use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Tell Cargo to rerun if shaders change
    println!("cargo:rerun-if-changed=shaders/hlsl/forward_simple.hlsl");
    println!("cargo:rerun-if-changed=shaders/hlsl/shadow_map.hlsl");
    println!("cargo:rerun-if-changed=shaders/hlsl/triangle.hlsl");

    // Compile unified HLSL shaders for both Vulkan (SPIR-V) and DirectX
    compile_unified_shaders();

    // Legacy triangle shader compilation for compatibility
    compile_legacy_triangle_shaders();
}

/// Compile unified HLSL shaders for both Vulkan (SPIR-V) and DirectX (DXIL)
fn compile_unified_shaders() {
    let shaders = vec![
        (
            "shaders/hlsl/forward_simple.hlsl",
            "forward_simple",
            "VSMain",
            "PSMain",
        ),
        (
            "shaders/hlsl/shadow_map.hlsl",
            "shadow_map",
            "VSMain",
            "PSMain",
        ),
        ("shaders/hlsl/triangle.hlsl", "triangle", "VSMain", "PSMain"),
    ];

    for (hlsl_src, name, vs_entry, ps_entry) in shaders {
        if !Path::new(hlsl_src).exists() {
            println!("cargo:warning=Shader {} not found, skipping", hlsl_src);
            continue;
        }

        // Compile to SPIR-V for Vulkan using DXC
        compile_hlsl_to_spirv(hlsl_src, name, vs_entry, ps_entry);

        // Compile to DXIL for DirectX using DXC (cross-platform)
        compile_hlsl_to_dxil(hlsl_src, name, vs_entry, ps_entry);
    }
}

/// Compile HLSL to SPIR-V for Vulkan using DXC
fn compile_hlsl_to_spirv(hlsl_src: &str, name: &str, vs_entry: &str, ps_entry: &str) {
    let vert_spv = format!("shaders/{}.vert.spv", name);
    let frag_spv = format!("shaders/{}.frag.spv", name);

    // Check if DXC is available
    if Command::new("dxc").arg("--version").output().is_err() {
        println!(
            "cargo:warning=DXC not found, using pre-compiled SPIR-V shaders for {}",
            name
        );
        return;
    }

    println!("cargo:warning=Compiling {} to SPIR-V with DXC", name);

    // Compile vertex shader to SPIR-V
    let vert_result = Command::new("dxc")
        .arg("-spirv") // Generate SPIR-V
        .arg("-T")
        .arg("vs_6_0")
        .arg("-E")
        .arg(vs_entry)
        .arg("-DVULKAN") // Define VULKAN for Vulkan-specific code
        .arg("-fspv-target-env=vulkan1.2")
        .arg("-Fo")
        .arg(&vert_spv)
        .arg(hlsl_src)
        .output();

    match vert_result {
        Ok(output) if output.status.success() => {
            println!("cargo:warning=Compiled {} vertex shader to SPIR-V", name);
        }
        Ok(output) => {
            eprintln!(
                "Warning: Failed to compile {} vertex shader:\n{}",
                name,
                String::from_utf8_lossy(&output.stderr)
            );
            println!(
                "cargo:warning=Using pre-compiled vertex shader for {}",
                name
            );
        }
        Err(e) => {
            eprintln!("Warning: Failed to run DXC for {}: {}", name, e);
            println!(
                "cargo:warning=Using pre-compiled vertex shader for {}",
                name
            );
        }
    }

    // Compile fragment shader to SPIR-V
    let frag_result = Command::new("dxc")
        .arg("-spirv") // Generate SPIR-V
        .arg("-T")
        .arg("ps_6_0")
        .arg("-E")
        .arg(ps_entry)
        .arg("-DVULKAN") // Define VULKAN for Vulkan-specific code
        .arg("-fspv-target-env=vulkan1.2")
        .arg("-Fo")
        .arg(&frag_spv)
        .arg(hlsl_src)
        .output();

    match frag_result {
        Ok(output) if output.status.success() => {
            println!("cargo:warning=Compiled {} fragment shader to SPIR-V", name);
        }
        Ok(output) => {
            eprintln!(
                "Warning: Failed to compile {} fragment shader:\n{}",
                name,
                String::from_utf8_lossy(&output.stderr)
            );
            println!(
                "cargo:warning=Using pre-compiled fragment shader for {}",
                name
            );
        }
        Err(e) => {
            eprintln!("Warning: Failed to run DXC for {}: {}", name, e);
            println!(
                "cargo:warning=Using pre-compiled fragment shader for {}",
                name
            );
        }
    }
}

/// Compile HLSL to DXIL for DirectX using DXC
fn compile_hlsl_to_dxil(hlsl_src: &str, name: &str, vs_entry: &str, ps_entry: &str) {
    let vs_out = format!("shaders/{}.vert.dxil", name);
    let ps_out = format!("shaders/{}.frag.dxil", name);

    // Check if DXC is available
    if Command::new("dxc").arg("--version").output().is_err() {
        println!(
            "cargo:warning=DXC not found, using embedded DirectX shaders for {}",
            name
        );
        return;
    }

    println!("cargo:warning=Compiling {} to DXIL with DXC", name);

    // Compile vertex shader to DXIL
    let vert_result = Command::new("dxc")
        .arg("-T")
        .arg("vs_6_0")
        .arg("-E")
        .arg(vs_entry)
        .arg("-validator-version")
        .arg("0.0") // Use internal validator (better VKD3D compatibility)
        .arg("-Fo")
        .arg(&vs_out)
        .arg(hlsl_src)
        .output();

    match vert_result {
        Ok(output) if output.status.success() => {
            println!("cargo:warning=Compiled {} vertex shader to DXIL", name);
        }
        Ok(output) => {
            eprintln!(
                "Warning: Failed to compile {} vertex shader to DXIL:\n{}",
                name,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            eprintln!("Warning: Failed to run DXC for {}: {}", name, e);
        }
    }

    // Compile fragment shader to DXIL
    let frag_result = Command::new("dxc")
        .arg("-T")
        .arg("ps_6_0")
        .arg("-E")
        .arg(ps_entry)
        .arg("-validator-version")
        .arg("0.0") // Use internal validator (better VKD3D compatibility)
        .arg("-Fo")
        .arg(&ps_out)
        .arg(hlsl_src)
        .output();

    match frag_result {
        Ok(output) if output.status.success() => {
            println!("cargo:warning=Compiled {} pixel shader to DXIL", name);
        }
        Ok(output) => {
            eprintln!(
                "Warning: Failed to compile {} pixel shader to DXIL:\n{}",
                name,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            eprintln!("Warning: Failed to run DXC for {}: {}", name, e);
        }
    }
}

fn compile_legacy_triangle_shaders() {
    // This function maintains compatibility with legacy GLSL shaders
    // Can be removed once all shaders are unified HLSL
    let out_dir = env::var("OUT_DIR").unwrap();
    let vertex_src = "shaders/triangle.vert";
    let fragment_src = "shaders/triangle.frag";
    let vertex_spv = Path::new(&out_dir).join("triangle_glsl.vert.spv");
    let fragment_spv = Path::new(&out_dir).join("triangle_glsl.frag.spv");

    if !Path::new(vertex_src).exists() || !Path::new(fragment_src).exists() {
        return;
    }

    // Try glslc first
    if let Ok(output) = Command::new("glslc").arg("--version").output() {
        if output.status.success() {
            let _ = Command::new("glslc")
                .arg(vertex_src)
                .arg("-o")
                .arg(&vertex_spv)
                .output();
            let _ = Command::new("glslc")
                .arg(fragment_src)
                .arg("-o")
                .arg(&fragment_spv)
                .output();
        }
    }
}
