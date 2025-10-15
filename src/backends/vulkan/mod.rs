//! Vulkan backend implementation
//!
//! This module implements the Vulkan graphics backend using vulkanalia.
//! Supports validation layers in debug mode and provides a complete
//! Vulkan rendering pipeline.

use super::*;
use anyhow::{Context, Result};
use std::ffi::CStr;
use std::os::raw::c_void;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::ExtDebugUtilsExtensionInstanceCommands;
use vulkanalia::window as vk_window;

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

/// Vulkan backend implementation
pub struct VulkanBackend {
    // Core Vulkan objects (wrapped in Option for initialization order)
    entry: Option<Entry>,
    instance: Option<Instance>,
    messenger: Option<vk::DebugUtilsMessengerEXT>,

    // Stub components (will be replaced in future issues)
    device: VulkanDevice,
    swapchain: VulkanSwapchain,
}

impl VulkanBackend {
    /// Create a new Vulkan backend
    pub fn new() -> Result<Self> {
        log::info!("Creating Vulkan backend");

        // Load Vulkan library
        let loader = unsafe { LibloadingLoader::new(LIBRARY)? };
        let entry = unsafe { Entry::new(loader).map_err(|e| anyhow::anyhow!("{e}"))? };

        log::info!("Vulkan library loaded successfully");

        Ok(Self {
            entry: Some(entry),
            instance: None,
            messenger: None,
            device: VulkanDevice::new(),
            swapchain: VulkanSwapchain::new(),
        })
    }

    /// Create Vulkan instance with validation layers
    fn create_instance(&mut self, window: &winit::window::Window) -> Result<()> {
        let entry = self.entry.as_ref().context("Entry not initialized")?;

        // Application info
        let app_info = vk::ApplicationInfo::builder()
            .application_name(b"Rusty Renderer\0")
            .application_version(vk::make_version(0, 1, 0))
            .engine_name(b"No Engine\0")
            .engine_version(vk::make_version(0, 1, 0))
            .api_version(vk::make_version(1, 0, 0));

        // Get required extensions from window
        let mut extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        // Add debug utils extension in debug mode
        if VALIDATION_ENABLED {
            extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
        }

        // Validation layers
        let available_layers = unsafe { entry.enumerate_instance_layer_properties()? }
            .iter()
            .map(|l| l.layer_name)
            .collect::<Vec<_>>();

        let mut layers = Vec::new();

        if VALIDATION_ENABLED {
            if available_layers.contains(&VALIDATION_LAYER) {
                layers.push(VALIDATION_LAYER.as_ptr());
                log::info!("Validation layers enabled");
            } else {
                log::warn!("Validation layers requested but not available");
            }
        }

        // Create instance
        let mut info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);

        // Debug messenger for instance creation/destruction
        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .user_callback(Some(debug_callback));

        if VALIDATION_ENABLED {
            info = info.push_next(&mut debug_info);
        }

        let instance = unsafe { entry.create_instance(&info, None)? };

        log::info!("Vulkan instance created successfully");

        self.instance = Some(instance);

        Ok(())
    }

    /// Create debug messenger
    fn create_debug_messenger(&mut self) -> Result<()> {
        if !VALIDATION_ENABLED {
            return Ok(());
        }

        let instance = self.instance.as_ref().context("Instance not initialized")?;

        let info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .user_callback(Some(debug_callback));

        let messenger = unsafe { instance.create_debug_utils_messenger_ext(&info, None)? };

        self.messenger = Some(messenger);
        log::info!("Debug messenger created");

        Ok(())
    }
}

impl GraphicsBackend for VulkanBackend {
    fn backend_type(&self) -> BackendType {
        BackendType::Vulkan
    }

    fn initialize(&mut self, window: &winit::window::Window) -> Result<()> {
        log::info!("Initializing Vulkan backend");

        // Create instance
        self.create_instance(window)
            .context("Failed to create Vulkan instance")?;

        // Create debug messenger
        self.create_debug_messenger()
            .context("Failed to create debug messenger")?;

        log::info!("Vulkan backend initialized");
        Ok(())
    }

    fn begin_frame(&mut self) -> Result<()> {
        // Will be implemented in issue #25
        Ok(())
    }

    fn end_frame(&mut self) -> Result<()> {
        // Will be implemented in issue #25
        Ok(())
    }

    fn resize(&mut self, _width: u32, _height: u32) -> Result<()> {
        // Will be implemented in issue #22 (swapchain)
        Ok(())
    }

    fn cleanup(&mut self) {
        log::info!("Cleaning up Vulkan backend");

        unsafe {
            if let Some(instance) = &self.instance {
                if let Some(messenger) = self.messenger {
                    instance.destroy_debug_utils_messenger_ext(messenger, None);
                }
                instance.destroy_instance(None);
            }
        }

        log::info!("Vulkan backend cleaned up");
    }

    fn device(&self) -> &dyn super::Device {
        &self.device
    }

    fn swapchain(&self) -> &dyn Swapchain {
        &self.swapchain
    }
}

/// Debug callback for Vulkan validation layers
unsafe extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = *data;
    let message = CStr::from_ptr(data.message).to_string_lossy();

    match severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("Vulkan [{type_:?}]: {message}");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("Vulkan [{type_:?}]: {message}");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("Vulkan [{type_:?}]: {message}");
        }
        _ => {
            log::debug!("Vulkan [{type_:?}]: {message}");
        }
    }

    vk::FALSE
}

/// Vulkan device stub
#[derive(Default)]
pub struct VulkanDevice;

impl VulkanDevice {
    /// Create a new Vulkan device stub
    pub fn new() -> Self {
        Self
    }
}

impl super::Device for VulkanDevice {
    fn name(&self) -> &str {
        "Vulkan Stub Device"
    }

    fn supports_feature(&self, _feature: &str) -> bool {
        // Stub: return false for all features
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Vulkan command buffer stub
#[derive(Default)]
pub struct VulkanCommandBuffer;

impl VulkanCommandBuffer {
    /// Create a new Vulkan command buffer stub
    pub fn new() -> Self {
        Self
    }
}

impl CommandBuffer for VulkanCommandBuffer {
    fn begin(&mut self) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn end(&mut self) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn clear_color(&mut self, _r: f32, _g: f32, _b: f32, _a: f32) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn bind_pipeline(&mut self, _pipeline: &dyn Pipeline) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn draw(&mut self, _vertex_count: u32, _instance_count: u32) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }
}

/// Vulkan pipeline stub
#[derive(Default)]
pub struct VulkanPipeline;

impl VulkanPipeline {
    /// Create a new Vulkan pipeline stub
    pub fn new() -> Self {
        Self
    }
}

impl Pipeline for VulkanPipeline {
    fn name(&self) -> &str {
        "Vulkan Stub Pipeline"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Vulkan resource stub
#[derive(Default)]
pub struct VulkanResource;

impl VulkanResource {
    /// Create a new Vulkan resource stub
    pub fn new() -> Self {
        Self
    }
}

impl Resource for VulkanResource {
    fn size(&self) -> usize {
        // Stub: return 0
        0
    }

    fn resource_type(&self) -> &str {
        "Vulkan Stub Resource"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Vulkan swapchain stub
pub struct VulkanSwapchain {
    width: u32,
    height: u32,
    current_frame: usize,
}

impl Default for VulkanSwapchain {
    fn default() -> Self {
        Self::new()
    }
}

impl VulkanSwapchain {
    /// Create a new Vulkan swapchain stub
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            current_frame: 0,
        }
    }
}

impl Swapchain for VulkanSwapchain {
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
        // Stub: will be implemented in M3
        Ok(())
    }

    fn present(&mut self) -> Result<()> {
        // Stub: will be implemented in M3
        Ok(())
    }

    fn recreate(&mut self, width: u32, height: u32) -> Result<()> {
        // Stub: update dimensions
        self.width = width;
        self.height = height;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulkan_backend_creation() {
        let backend = VulkanBackend::new();
        assert!(backend.is_ok(), "Failed to create Vulkan backend");
    }

    #[test]
    fn test_vulkan_backend_type() {
        let backend = VulkanBackend::new().unwrap();
        assert_eq!(backend.backend_type(), BackendType::Vulkan);
    }

    #[test]
    fn test_vulkan_device_access() {
        let backend = VulkanBackend::new().unwrap();
        let device = backend.device();
        assert_eq!(device.name(), "Vulkan Stub Device");
    }

    #[test]
    fn test_vulkan_device_features() {
        let backend = VulkanBackend::new().unwrap();
        let device = backend.device();
        assert!(!device.supports_feature("any_feature"));
    }

    #[test]
    fn test_vulkan_swapchain_access() {
        let backend = VulkanBackend::new().unwrap();
        let swapchain = backend.swapchain();
        assert_eq!(swapchain.width(), 800);
        assert_eq!(swapchain.height(), 600);
        assert_eq!(swapchain.current_frame(), 0);
    }

    #[test]
    fn test_vulkan_swapchain_recreate() {
        let mut backend = VulkanBackend::new().unwrap();
        let swapchain = backend.swapchain();
        assert_eq!(swapchain.width(), 800);
        assert_eq!(swapchain.height(), 600);

        // Access the mutable swapchain field directly for testing
        let result = backend.swapchain.recreate(1920, 1080);
        assert!(result.is_ok());

        let swapchain = backend.swapchain();
        assert_eq!(swapchain.width(), 1920);
        assert_eq!(swapchain.height(), 1080);
    }

    #[test]
    fn test_vulkan_command_buffer_creation() {
        let _cmd_buffer = VulkanCommandBuffer::new();
        // Just verify it doesn't panic
    }

    #[test]
    fn test_vulkan_command_buffer_operations() {
        let mut cmd_buffer = VulkanCommandBuffer::new();
        assert!(cmd_buffer.begin().is_ok());
        assert!(cmd_buffer.clear_color(0.0, 0.0, 0.0, 1.0).is_ok());
        assert!(cmd_buffer.draw(3, 1).is_ok());
        assert!(cmd_buffer.end().is_ok());
    }

    #[test]
    fn test_vulkan_pipeline_creation() {
        let pipeline = VulkanPipeline::new();
        assert_eq!(pipeline.name(), "Vulkan Stub Pipeline");
    }

    #[test]
    fn test_vulkan_pipeline_bind() {
        let mut cmd_buffer = VulkanCommandBuffer::new();
        let pipeline = VulkanPipeline::new();
        assert!(cmd_buffer.bind_pipeline(&pipeline).is_ok());
    }

    #[test]
    fn test_vulkan_resource_creation() {
        let resource = VulkanResource::new();
        assert_eq!(resource.size(), 0);
        assert_eq!(resource.resource_type(), "Vulkan Stub Resource");
    }

    #[test]
    fn test_vulkan_backend_frame_operations() {
        let mut backend = VulkanBackend::new().unwrap();
        assert!(backend.begin_frame().is_ok());
        assert!(backend.end_frame().is_ok());
    }

    #[test]
    fn test_vulkan_backend_resize() {
        let mut backend = VulkanBackend::new().unwrap();
        assert!(backend.resize(1024, 768).is_ok());
    }

    #[test]
    fn test_vulkan_backend_cleanup() {
        let mut backend = VulkanBackend::new().unwrap();
        // Should not panic
        backend.cleanup();
    }

    #[test]
    fn test_vulkan_swapchain_operations() {
        let mut backend = VulkanBackend::new().unwrap();
        assert!(backend.swapchain.acquire_next_image().is_ok());
        assert!(backend.swapchain.present().is_ok());
    }
}
