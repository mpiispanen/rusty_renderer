//! Shader module for Vulkan backend
//!
//! Contains hardcoded SPIR-V shaders for triangle rendering.
//! Generated from GLSL source using glslangValidator.

/// Forward rendering vertex shader SPIR-V
#[allow(dead_code)] // Will be used when we have forward shading
pub const FORWARD_VERTEX_SHADER: &[u8] = include_bytes!("../../../shaders/forward.vert.spv");

/// Forward rendering fragment shader SPIR-V
#[allow(dead_code)] // Will be used when we have forward shading
pub const FORWARD_FRAGMENT_SHADER: &[u8] = include_bytes!("../../../shaders/forward.frag.spv");

/// Convert byte slice to u32 slice for Vulkan
#[allow(dead_code)] // Will be used when we need dynamic shader loading
pub fn bytes_to_u32_vec(bytes: &[u8]) -> Vec<u32> {
    assert_eq!(
        bytes.len() % 4,
        0,
        "SPIR-V byte length must be multiple of 4"
    );
    let mut words = vec![0u32; bytes.len() / 4];
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), words.as_mut_ptr() as *mut u8, bytes.len());
    }
    words
}
