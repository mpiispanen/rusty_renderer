//! Backend selection integration tests
//!
//! Tests for backend selection and initialization via CLI arguments.

use rusty_renderer::backends::{create_backend, BackendType};

#[test]
fn test_create_vulkan_backend() {
    let backend = create_backend(BackendType::Vulkan, false);
    assert!(backend.is_ok(), "Failed to create Vulkan backend");
    let backend = backend.unwrap();
    assert_eq!(backend.backend_type(), BackendType::Vulkan);
}

#[test]
fn test_create_directx_backend() {
    let backend = create_backend(BackendType::DirectX12, false);
    assert!(backend.is_ok(), "Failed to create DirectX backend");
    let backend = backend.unwrap();
    assert_eq!(backend.backend_type(), BackendType::DirectX12);
}

#[test]
#[ignore] // Wgpu backend removed
fn test_create_wgpu_backend() {
    // Removed - wgpu backend no longer supported
}

#[test]
fn test_backend_factory_returns_correct_types() {
    let vulkan = create_backend(BackendType::Vulkan, false).unwrap();
    let directx = create_backend(BackendType::DirectX12, false).unwrap();

    // Verify each backend has correct type
    assert_eq!(vulkan.backend_type(), BackendType::Vulkan);
    assert_eq!(directx.backend_type(), BackendType::DirectX12);

    // Verify they're different instances
    assert_ne!(vulkan.backend_type(), directx.backend_type());
}

#[test]
fn test_backend_device_access() {
    let backend = create_backend(BackendType::Vulkan, false).unwrap();
    let device = backend.device();

    // Should have a valid name
    assert!(!device.name().is_empty());
    assert!(device.name().contains("stub") || device.name().contains("Vulkan"));
}

#[test]
fn test_backend_swapchain_access() {
    let backend = create_backend(BackendType::Vulkan, false).unwrap();
    let swapchain = backend.swapchain();

    // Should have valid dimensions
    assert!(swapchain.width() > 0);
    assert!(swapchain.height() > 0);
}

#[test]
fn test_multiple_backend_instances() {
    // Should be able to create multiple backends
    let backend1 = create_backend(BackendType::Vulkan, false).unwrap();
    let backend2 = create_backend(BackendType::Vulkan, false).unwrap();

    assert_eq!(backend1.backend_type(), backend2.backend_type());
}
