//! Vulkan backend stub
//!
//! This is a stub implementation for issue #12.
//! Real Vulkan implementation will be added in M3.

use super::*;

/// Vulkan backend stub
pub struct VulkanBackend;

impl VulkanBackend {
    /// Create a new Vulkan backend stub
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl GraphicsBackend for VulkanBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Vulkan
    }

    fn initialize(&mut self, _window: &winit::window::Window) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn begin_frame(&mut self) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn end_frame(&mut self) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn resize(&mut self, _width: u32, _height: u32) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn cleanup(&mut self) {
        // Stub: will be implemented in M3
    }

    fn device(&self) -> &dyn Device {
        // Stub: will be implemented in M3
        unimplemented!("Device access not yet implemented")
    }

    fn swapchain(&self) -> &dyn Swapchain {
        // Stub: will be implemented in M3
        unimplemented!("Swapchain access not yet implemented")
    }
}
