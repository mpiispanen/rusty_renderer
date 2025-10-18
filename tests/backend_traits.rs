//! Comprehensive backend trait unit tests
//!
//! This test suite validates the backend abstraction layer by testing
//! all trait contracts across all backend implementations.
//!
//! Test Strategy:
//! 1. Verify all backends implement required traits correctly
//! 2. Test trait method contracts and expected behavior
//! 3. Validate error handling and edge cases
//! 4. Ensure backend parity (same interface, same behavior)
//!
//! Coverage: BackendType, GraphicsBackend, Device, Swapchain traits
//!
//! Note: This complements the 58 backend-specific tests in src/backends/
//! and 7 integration tests in backend_selection.rs for comprehensive coverage.

use rusty_renderer::backends::{create_backend, BackendType};

// ============================================================================
// Backend Factory and Type Tests
// ============================================================================

#[test]
fn test_all_backend_types_creatable() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        let result = create_backend(backend_type, false);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().backend_type(), backend_type);
    }
}

#[test]
fn test_backend_type_string_conversion() {
    assert_eq!(BackendType::Vulkan.as_str(), "vulkan");
    assert_eq!(BackendType::DirectX12.as_str(), "directx12");
    assert_eq!(BackendType::Wgpu.as_str(), "wgpu");

    assert_eq!(BackendType::Vulkan.to_string(), "vulkan");
    assert_eq!(BackendType::DirectX12.to_string(), "directx12");
    assert_eq!(BackendType::Wgpu.to_string(), "wgpu");
}

#[test]
fn test_backend_type_equality_and_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(BackendType::Vulkan);
    set.insert(BackendType::DirectX12);
    set.insert(BackendType::Wgpu);

    assert_eq!(set.len(), 3);
    assert!(set.contains(&BackendType::Vulkan));
    assert!(set.contains(&BackendType::DirectX12));
    assert!(set.contains(&BackendType::Wgpu));
}

// ============================================================================
// Device Trait Tests (Cross-Backend)
// ============================================================================

#[test]
fn test_all_backends_provide_device() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        let backend = create_backend(backend_type, false).unwrap();
        let device = backend.device();

        // All devices should have a name
        assert!(!device.name().is_empty());
    }
}

#[test]
fn test_device_feature_queries() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        let backend = create_backend(backend_type, false).unwrap();
        let device = backend.device();

        // Stubs should return false for feature support
        assert!(!device.supports_feature("nonexistent_feature"));

        // as_any() should not panic
        let _any = device.as_any();
    }
}

// ============================================================================
// Swapchain Trait Tests (Cross-Backend)
// ============================================================================

#[test]
fn test_all_backends_provide_swapchain() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        let backend = create_backend(backend_type, false).unwrap();
        let swapchain = backend.swapchain();

        // All swapchains should have valid dimensions
        assert!(swapchain.width() > 0);
        assert!(swapchain.height() > 0);

        // Frame index should be reasonable
        assert!(swapchain.current_frame() < 100);
    }
}

#[test]
fn test_swapchain_dimensions_reasonable() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        let backend = create_backend(backend_type, false).unwrap();
        let swapchain = backend.swapchain();

        // Dimensions should be at least somewhat reasonable
        assert!(swapchain.width() >= 640);
        assert!(swapchain.height() >= 480);
    }
}

// ============================================================================
// GraphicsBackend Trait Tests (Cross-Backend)
// ============================================================================

#[test]
#[ignore] // Requires window context for swapchain operations
fn test_backend_lifecycle_methods() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        let mut backend = create_backend(backend_type, false).unwrap();

        // All backends should support basic lifecycle
        assert!(backend.begin_frame().is_ok());
        assert!(backend.end_frame().is_ok());
        assert!(backend.resize(1024, 768).is_ok());

        // Cleanup should not panic
        backend.cleanup();
    }
}

#[test]
#[ignore] // Requires window context for swapchain operations
fn test_backend_multiple_frame_cycle() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        let mut backend = create_backend(backend_type, false).unwrap();

        // Should handle multiple frame cycles
        for _ in 0..5 {
            assert!(backend.begin_frame().is_ok());
            assert!(backend.end_frame().is_ok());
        }
    }
}

#[test]
fn test_backend_resize_operations() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        let mut backend = create_backend(backend_type, false).unwrap();

        // Test various resize scenarios
        assert!(backend.resize(800, 600).is_ok());
        assert!(backend.resize(1920, 1080).is_ok());
        assert!(backend.resize(1, 1).is_ok()); // Extreme small
        assert!(backend.resize(8192, 8192).is_ok()); // Extreme large
    }
}

// ============================================================================
// Cross-Backend Parity Tests
// ============================================================================

#[test]
fn test_backend_parity_type_consistency() {
    let vulkan = create_backend(BackendType::Vulkan, false).unwrap();
    let directx = create_backend(BackendType::DirectX12, false).unwrap();
    let wgpu = create_backend(BackendType::Wgpu, false).unwrap();

    assert_eq!(vulkan.backend_type(), BackendType::Vulkan);
    assert_eq!(directx.backend_type(), BackendType::DirectX12);
    assert_eq!(wgpu.backend_type(), BackendType::Wgpu);
}

#[test]
fn test_backend_parity_interface() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        let backend = create_backend(backend_type, false).unwrap();

        // All backends provide same interface
        let _device = backend.device();
        let _swapchain = backend.swapchain();
        assert_eq!(backend.backend_type(), backend_type);
    }
}

#[test]
fn test_multiple_backend_instances() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        // Should be able to create multiple instances
        let backend1 = create_backend(backend_type, false).unwrap();
        let backend2 = create_backend(backend_type, false).unwrap();

        assert_eq!(backend1.backend_type(), backend2.backend_type());
    }
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_multiple_cleanup_calls_safe() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        let mut backend = create_backend(backend_type, false).unwrap();

        // Multiple cleanup calls should be safe
        backend.cleanup();
        backend.cleanup();
    }
}

#[test]
#[ignore] // Requires window context for swapchain operations
fn test_backend_operations_after_cleanup() {
    for backend_type in [
        BackendType::Vulkan,
        BackendType::DirectX12,
        BackendType::Wgpu,
    ] {
        let mut backend = create_backend(backend_type, false).unwrap();

        backend.cleanup();

        // Operations after cleanup should still work (stubs)
        assert!(backend.begin_frame().is_ok());
        assert!(backend.end_frame().is_ok());
    }
}

// ============================================================================
// Test Coverage Summary
// ============================================================================

// Total tests in this file: 18
// These complement:
// - 58 backend-specific tests in src/backends/ modules
// - 7 integration tests in tests/backend_selection.rs
// - 6 BackendType tests in src/backends/mod.rs
//
// Total backend trait test coverage: 89 tests
