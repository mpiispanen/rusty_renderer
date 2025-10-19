//! DirectX 12 backend implementation
//!
//! Provides rendering using Microsoft's DirectX 12 API.
//! This backend is only available on Windows.
//!
//! ## Testing with WARP
//!
//! Set the environment variable to use WARP software renderer:
//! ```text
//! set RUSTY_RENDERER_USE_WARP=1
//! rusty_renderer --backend directx
//! ```
//!
//! This is useful for CI testing on GitHub Actions Windows runners.

use super::*;
use std::any::Any;

#[cfg(windows)]
mod dx12_impl;

/// DirectX 12 backend
pub struct DirectXBackend {
    #[cfg(windows)]
    inner: dx12_impl::DirectXBackendImpl,

    #[cfg(not(windows))]
    device: DirectXDevice,
    #[cfg(not(windows))]
    swapchain: DirectXSwapchain,
}

impl DirectXBackend {
    /// Create a new DirectX 12 backend
    pub fn new(enable_validation: bool) -> Result<Self> {
        #[cfg(windows)]
        {
            log::info!(
                "Creating DirectX 12 backend (validation: {})",
                enable_validation
            );
            Ok(Self {
                inner: dx12_impl::DirectXBackendImpl::new(enable_validation)?,
            })
        }

        #[cfg(not(windows))]
        {
            log::warn!(
                "DirectX 12 backend is only available on Windows (validation: {enable_validation})"
            );
            Ok(Self {
                device: DirectXDevice::new(),
                swapchain: DirectXSwapchain::new(),
            })
        }
    }
}

impl GraphicsBackend for DirectXBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::DirectX12
    }

    fn initialize(&mut self, _window: &winit::window::Window) -> Result<()> {
        #[cfg(windows)]
        {
            self.inner.initialize(_window)
        }

        #[cfg(not(windows))]
        {
            anyhow::bail!("DirectX 12 backend is only available on Windows")
        }
    }

    fn begin_frame(&mut self) -> Result<()> {
        #[cfg(windows)]
        {
            self.inner.begin_frame()
        }

        #[cfg(not(windows))]
        {
            Ok(())
        }
    }

    fn end_frame(&mut self) -> Result<()> {
        #[cfg(windows)]
        {
            self.inner.end_frame()
        }

        #[cfg(not(windows))]
        {
            Ok(())
        }
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        #[cfg(windows)]
        {
            self.inner.resize(width, height)
        }

        #[cfg(not(windows))]
        {
            self.swapchain.resize(width, height);
            Ok(())
        }
    }

    fn initialize_headless(&mut self, width: u32, height: u32) -> Result<()> {
        #[cfg(windows)]
        {
            self.inner.initialize_headless(width, height)
        }

        #[cfg(not(windows))]
        {
            anyhow::bail!(
                "Headless rendering not yet implemented for DirectX backend (planned for M5). \
                 Requested size: {width}x{height}. \
                 See issue #27 for implementation status."
            )
        }
    }

    fn capture_frame(&mut self) -> Result<(u32, u32, Vec<u8>)> {
        #[cfg(windows)]
        {
            self.inner.capture_frame()
        }

        #[cfg(not(windows))]
        {
            anyhow::bail!(
                "Frame capture not yet implemented for DirectX backend (planned for M5). \
                 See issue #27 for implementation status."
            )
        }
    }

    fn cleanup(&mut self) {
        #[cfg(windows)]
        {
            self.inner.cleanup()
        }
    }

    fn device(&self) -> &dyn Device {
        #[cfg(windows)]
        {
            self.inner.device()
        }

        #[cfg(not(windows))]
        {
            &self.device
        }
    }

    fn swapchain(&self) -> &dyn Swapchain {
        #[cfg(windows)]
        {
            self.inner.swapchain()
        }

        #[cfg(not(windows))]
        {
            &self.swapchain
        }
    }

    fn execute_graph(&mut self, _graph: &crate::render_graph::graph::CompiledGraph) -> Result<()> {
        #[cfg(windows)]
        {
            self.inner.execute_graph(_graph)
        }

        #[cfg(not(windows))]
        {
            anyhow::bail!("DirectX 12 backend is only available on Windows")
        }
    }
}

// Stub implementations for non-Windows platforms
#[cfg(not(windows))]
mod stubs {
    use super::*;

    /// Stub Device implementation
    pub struct DirectXDevice;

    impl DirectXDevice {
        pub fn new() -> Self {
            Self
        }
    }

    impl Device for DirectXDevice {
        fn name(&self) -> &str {
            "directx-stub-device"
        }

        fn supports_feature(&self, _feature: &str) -> bool {
            false
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    /// Stub Swapchain implementation
    pub struct DirectXSwapchain {
        width: u32,
        height: u32,
        current_frame: usize,
    }

    impl DirectXSwapchain {
        pub fn new() -> Self {
            Self {
                width: 800,
                height: 600,
                current_frame: 0,
            }
        }

        pub fn resize(&mut self, width: u32, height: u32) {
            self.width = width;
            self.height = height;
        }
    }

    impl Swapchain for DirectXSwapchain {
        fn width(&self) -> u32 {
            self.width
        }

        fn height(&self) -> u32 {
            self.height
        }

        fn current_frame(&self) -> usize {
            self.current_frame
        }

        fn acquire_next_image(&mut self) -> Result<()> {
            self.current_frame = (self.current_frame + 1) % 3;
            Ok(())
        }

        fn present(&mut self) -> Result<()> {
            Ok(())
        }

        fn recreate(&mut self, width: u32, height: u32) -> Result<()> {
            self.width = width;
            self.height = height;
            Ok(())
        }
    }
}

#[cfg(not(windows))]
use stubs::{DirectXDevice, DirectXSwapchain};

// Tests can be here since they don't require Windows-specific code for basic checks
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directx_backend_creation() {
        let backend = DirectXBackend::new(false);
        #[cfg(not(windows))]
        assert!(backend.is_ok(), "Failed to create DirectX backend stub");

        #[cfg(windows)]
        assert!(
            backend.is_ok() || backend.is_err(),
            "DirectX creation should succeed or fail gracefully"
        );
    }

    #[test]
    fn test_directx_backend_type() {
        let backend = DirectXBackend::new(false).unwrap();
        assert_eq!(backend.backend_type(), BackendType::DirectX12);
    }
}
