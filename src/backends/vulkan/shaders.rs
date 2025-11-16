//! Shader module for Vulkan backend
//!
//! Shaders are now loaded dynamically through the render graph system.

/// Convert byte slice to u32 slice for Vulkan SPIR-V shaders
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
