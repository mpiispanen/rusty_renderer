use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Tell Cargo to rerun if shaders change
    println!("cargo:rerun-if-changed=shaders/triangle.vert");
    println!("cargo:rerun-if-changed=shaders/triangle.frag");
    println!("cargo:rerun-if-changed=shaders/hlsl/triangle.hlsl");
    println!("cargo:rerun-if-changed=shaders/hlsl/forward.hlsl");

    let out_dir = env::var("OUT_DIR").unwrap();

    // Compile HLSL shaders for DirectX (Windows only)
    #[cfg(target_os = "windows")]
    compile_hlsl_shaders(&out_dir);

    // Compile forward rendering shaders from HLSL to SPIR-V
    // This ensures both Vulkan and DirectX use the same shader source
    compile_forward_shaders();

    // Note: For cross-compilation from Linux, we skip HLSL compilation
    // and embed pre-compiled bytecode instead

    // Try to compile shaders if glslc or glslangValidator is available
    let vertex_src = "shaders/triangle.vert";
    let fragment_src = "shaders/triangle.frag";
    let vertex_spv = Path::new(&out_dir).join("triangle.vert.spv");
    let fragment_spv = Path::new(&out_dir).join("triangle.frag.spv");

    // Try glslc first (from Vulkan SDK)
    let mut compiled = false;
    if let Ok(output) = Command::new("glslc").arg("--version").output() {
        if output.status.success() {
            println!("cargo:warning=Compiling shaders with glslc");

            // Compile vertex shader
            let vert_result = Command::new("glslc")
                .arg(vertex_src)
                .arg("-o")
                .arg(&vertex_spv)
                .output()
                .expect("Failed to run glslc for vertex shader");

            if !vert_result.status.success() {
                panic!(
                    "Vertex shader compilation failed:\n{}",
                    String::from_utf8_lossy(&vert_result.stderr)
                );
            }

            // Compile fragment shader
            let frag_result = Command::new("glslc")
                .arg(fragment_src)
                .arg("-o")
                .arg(&fragment_spv)
                .output()
                .expect("Failed to run glslc for fragment shader");

            if !frag_result.status.success() {
                panic!(
                    "Fragment shader compilation failed:\n{}",
                    String::from_utf8_lossy(&frag_result.stderr)
                );
            }

            compiled = true;
            println!("cargo:warning=Shaders compiled successfully with glslc");
        }
    }

    // Try glslangValidator if glslc not available
    if !compiled {
        if let Ok(output) = Command::new("glslangValidator").arg("--version").output() {
            if output.status.success() {
                println!("cargo:warning=Compiling shaders with glslangValidator");

                // Compile vertex shader
                let vert_result = Command::new("glslangValidator")
                    .arg("-V")
                    .arg(vertex_src)
                    .arg("-o")
                    .arg(&vertex_spv)
                    .output()
                    .expect("Failed to run glslangValidator for vertex shader");

                if !vert_result.status.success() {
                    panic!(
                        "Vertex shader compilation failed:\n{}",
                        String::from_utf8_lossy(&vert_result.stderr)
                    );
                }

                // Compile fragment shader
                let frag_result = Command::new("glslangValidator")
                    .arg("-V")
                    .arg(fragment_src)
                    .arg("-o")
                    .arg(&fragment_spv)
                    .output()
                    .expect("Failed to run glslangValidator for fragment shader");

                if !frag_result.status.success() {
                    panic!(
                        "Fragment shader compilation failed:\n{}",
                        String::from_utf8_lossy(&frag_result.stderr)
                    );
                }

                compiled = true;
                println!("cargo:warning=Shaders compiled successfully with glslangValidator");
            }
        }
    }

    // Validate compiled shaders if spirv-val is available
    if compiled && Command::new("spirv-val").arg("--version").output().is_ok() {
        // Validate vertex shader
        let vert_val = Command::new("spirv-val")
            .arg(&vertex_spv)
            .output()
            .expect("Failed to run spirv-val");

        if !vert_val.status.success() {
            panic!(
                "Vertex shader validation failed:\n{}",
                String::from_utf8_lossy(&vert_val.stderr)
            );
        }

        // Validate fragment shader
        let frag_val = Command::new("spirv-val")
            .arg(&fragment_spv)
            .output()
            .expect("Failed to run spirv-val");

        if !frag_val.status.success() {
            panic!(
                "Fragment shader validation failed:\n{}",
                String::from_utf8_lossy(&frag_val.stderr)
            );
        }

        println!("cargo:warning=Shaders validated successfully with spirv-val");
    }

    // If compilation failed, try to use pre-compiled shaders
    if !compiled {
        let precompiled_vert = "shaders/triangle.vert.spv";
        let precompiled_frag = "shaders/triangle.frag.spv";

        if Path::new(precompiled_vert).exists() && Path::new(precompiled_frag).exists() {
            fs::copy(precompiled_vert, &vertex_spv)
                .expect("Failed to copy pre-compiled vertex shader");
            fs::copy(precompiled_frag, &fragment_spv)
                .expect("Failed to copy pre-compiled fragment shader");
            println!("cargo:warning=Using pre-compiled shaders");
        } else {
            panic!(
                "No shader compiler found (glslc or glslangValidator) and no pre-compiled shaders available.\n\
                 Install Vulkan SDK or compile shaders manually."
            );
        }
    }
}

#[cfg(target_os = "windows")]
fn compile_hlsl_shaders(out_dir: &str) {
    use std::process::Command;

    let hlsl_src = "shaders/hlsl/triangle.hlsl";
    let vs_out = Path::new(out_dir).join("triangle_vs.cso");
    let ps_out = Path::new(out_dir).join("triangle_ps.cso");

    // Try to find dxc (DirectX Shader Compiler)
    let dxc_result = Command::new("dxc")
        .arg("/T")
        .arg("vs_6_0") // Vertex shader model 6.0
        .arg("/E")
        .arg("VSMain") // Entry point
        .arg("/Fo")
        .arg(&vs_out) // Output file
        .arg(hlsl_src)
        .output();

    match dxc_result {
        Ok(output) if output.status.success() => {
            println!("cargo:warning=Compiled vertex shader with dxc");

            // Compile pixel shader
            let ps_result = Command::new("dxc")
                .arg("/T")
                .arg("ps_6_0") // Pixel shader model 6.0
                .arg("/E")
                .arg("PSMain") // Entry point
                .arg("/Fo")
                .arg(&ps_out) // Output file
                .arg(hlsl_src)
                .output()
                .expect("Failed to compile pixel shader");

            if !ps_result.status.success() {
                panic!(
                    "Pixel shader compilation failed:\n{}",
                    String::from_utf8_lossy(&ps_result.stderr)
                );
            }

            println!("cargo:warning=HLSL shaders compiled successfully");
        }
        _ => {
            // dxc not available, try fxc
            let fxc_result = Command::new("fxc")
                .arg("/T")
                .arg("vs_5_0") // Vertex shader model 5.0
                .arg("/E")
                .arg("VSMain") // Entry point
                .arg("/Fo")
                .arg(&vs_out) // Output file
                .arg(hlsl_src)
                .output();

            match fxc_result {
                Ok(output) if output.status.success() => {
                    println!("cargo:warning=Compiled vertex shader with fxc");

                    // Compile pixel shader
                    let ps_result = Command::new("fxc")
                        .arg("/T")
                        .arg("ps_5_0") // Pixel shader model 5.0
                        .arg("/E")
                        .arg("PSMain") // Entry point
                        .arg("/Fo")
                        .arg(&ps_out) // Output file
                        .arg(hlsl_src)
                        .output()
                        .expect("Failed to compile pixel shader");

                    if !ps_result.status.success() {
                        panic!(
                            "Pixel shader compilation failed:\n{}",
                            String::from_utf8_lossy(&ps_result.stderr)
                        );
                    }

                    println!("cargo:warning=HLSL shaders compiled successfully with fxc");
                }
                _ => {
                    println!("cargo:warning=No HLSL compiler found (dxc or fxc), will use embedded bytecode");
                }
            }
        }
    }
}

fn compile_forward_shaders() {
    let hlsl_src = "shaders/hlsl/forward.hlsl";
    let vert_spv = "shaders/forward.vert.spv";
    let frag_spv = "shaders/forward.frag.spv";

    // Check if shaders already exist
    if Path::new(vert_spv).exists() && Path::new(frag_spv).exists() {
        println!("cargo:warning=Using pre-compiled forward shaders");
        return;
    }

    // Check if glslangValidator is available
    if Command::new("glslangValidator")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("Warning: glslangValidator not found");
        eprintln!("Pre-compiled shaders should exist at:");
        eprintln!("  - {}", vert_spv);
        eprintln!("  - {}", frag_spv);

        if !Path::new(vert_spv).exists() || !Path::new(frag_spv).exists() {
            panic!("glslangValidator not found and no pre-compiled shaders available!");
        }
        return;
    }

    println!("cargo:warning=Compiling forward shaders from HLSL to SPIR-V");

    // Compile vertex shader with VULKAN define
    let vert_result = Command::new("glslangValidator")
        .arg("-V") // Generate SPIR-V (automatically defines VULKAN)
        .arg("-D") // Input is HLSL
        .arg("-e")
        .arg("VSMain") // Entry point
        .arg("--hlsl-iomap") // Use HLSL I/O mapping
        .arg("-S")
        .arg("vert") // Shader stage
        .arg(hlsl_src)
        .arg("-o")
        .arg(vert_spv)
        .output()
        .expect("Failed to run glslangValidator for forward vertex shader");

    if !vert_result.status.success() {
        panic!(
            "Forward vertex shader compilation failed:\n{}",
            String::from_utf8_lossy(&vert_result.stderr)
        );
    }

    // Compile fragment shader with VULKAN define
    let frag_result = Command::new("glslangValidator")
        .arg("-V") // Generate SPIR-V (automatically defines VULKAN)
        .arg("-D") // Input is HLSL
        .arg("-e")
        .arg("PSMain") // Entry point
        .arg("--hlsl-iomap") // Use HLSL I/O mapping
        .arg("-S")
        .arg("frag") // Shader stage
        .arg(hlsl_src)
        .arg("-o")
        .arg(frag_spv)
        .output()
        .expect("Failed to run glslangValidator for forward fragment shader");

    if !frag_result.status.success() {
        panic!(
            "Forward fragment shader compilation failed:\n{}",
            String::from_utf8_lossy(&frag_result.stderr)
        );
    }

    println!("cargo:warning=Forward shaders compiled successfully with glslangValidator");

    // Validate shaders
    if Command::new("spirv-val").arg("--version").output().is_ok() {
        let vert_val = Command::new("spirv-val")
            .arg(vert_spv)
            .output()
            .expect("Failed to run spirv-val");

        if !vert_val.status.success() {
            panic!(
                "Forward vertex shader validation failed:\n{}",
                String::from_utf8_lossy(&vert_val.stderr)
            );
        }

        let frag_val = Command::new("spirv-val")
            .arg(frag_spv)
            .output()
            .expect("Failed to run spirv-val");

        if !frag_val.status.success() {
            panic!(
                "Forward fragment shader validation failed:\n{}",
                String::from_utf8_lossy(&frag_val.stderr)
            );
        }

        println!("cargo:warning=Forward shaders validated successfully with spirv-val");
    }
}
