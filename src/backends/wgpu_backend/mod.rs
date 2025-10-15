//! wgpu backend stub
//!
//! This is a stub implementation for issue #14.
//! Real wgpu implementation will be added in M4.

use super::*;

/// wgpu backend stub
pub struct WgpuBackend;

impl WgpuBackend {
    /// Create a new wgpu backend stub
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl GraphicsBackend for WgpuBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Wgpu
    }

    fn initialize(&mut self, _window: &winit::window::Window) -> Result<()> {
        // Stub: will be implemented in M4
        Ok(())
    }

    fn begin_frame(&mut self) -> Result<()> {
        // Stub: will be implemented in M4
        Ok(())
    }

    fn end_frame(&mut self) -> Result<()> {
        // Stub: will be implemented in M4
        Ok(())
    }

    fn resize(&mut self, _width: u32, _height: u32) -> Result<()> {
        // Stub: will be implemented in M4
        Ok(())
    }

    fn cleanup(&mut self) {
        // Stub: will be implemented in M4
    }

    fn device(&self) -> &dyn Device {
        // Stub: will be implemented in M4
        unimplemented!("Device access not yet implemented")
    }

    fn swapchain(&self) -> &dyn Swapchain {
        // Stub: will be implemented in M4
        unimplemented!("Swapchain access not yet implemented")
    }
}
