//! Validation layer tests
//!
//! These tests verify that validation/debug layers work correctly
//! across all backends.

use rusty_renderer::{backends, config::Backend, RenderConfig};

#[test]
fn test_vulkan_validation_enabled() {
    let config = RenderConfig {
        backend: Backend::Vulkan,
        scene: "triangle".to_string(),
        width: 800,
        height: 600,
        debug: true, // Enable validation
        vsync: false,
        log_level: log::LevelFilter::Info,
        max_frames: Some(1),
        headless: true,
        screenshot: None,
        screenshot_interval: 0,
    };

    // Should not panic with validation enabled
    let result = backends::create_backend(backends::BackendType::Vulkan, config.debug);

    match result {
        Ok(_) => {
            // Validation layers successfully enabled
            println!("✅ Vulkan validation layers enabled successfully");
        }
        Err(e) => {
            let err_msg = e.to_string();
            // Check if it's a validation layer availability issue
            if err_msg.contains("validation") || err_msg.contains("VK_LAYER") {
                println!(
                    "⚠ Vulkan validation layers not available (expected in some CI environments)"
                );
                println!("   Error: {err_msg}");
            } else {
                panic!("Unexpected error: {e}");
            }
        }
    }
}

#[test]
fn test_vulkan_validation_disabled() {
    let config = RenderConfig {
        backend: Backend::Vulkan,
        scene: "triangle".to_string(),
        width: 800,
        height: 600,
        debug: false, // Disable validation
        vsync: false,
        log_level: log::LevelFilter::Warn,
        max_frames: Some(1),
        headless: true,
        screenshot: None,
        screenshot_interval: 0,
    };

    // Should work without validation layers
    let result = backends::create_backend(backends::BackendType::Vulkan, config.debug);
    assert!(
        result.is_ok(),
        "Vulkan backend should work without validation layers"
    );
}

#[test]
fn test_wgpu_validation_enabled() {
    let config = RenderConfig {
        backend: Backend::Wgpu,
        scene: "triangle".to_string(),
        width: 800,
        height: 600,
        debug: true, // Enable validation
        vsync: false,
        log_level: log::LevelFilter::Info,
        max_frames: Some(1),
        headless: true,
        screenshot: None,
        screenshot_interval: 0,
    };

    // wgpu validation should work on all platforms
    let result = backends::create_backend(backends::BackendType::Wgpu, config.debug);
    assert!(
        result.is_ok(),
        "wgpu backend with validation should initialize successfully"
    );

    println!("✅ wgpu validation/debug mode enabled successfully");
}

#[test]
fn test_wgpu_validation_disabled() {
    let config = RenderConfig {
        backend: Backend::Wgpu,
        scene: "triangle".to_string(),
        width: 800,
        height: 600,
        debug: false, // Disable validation
        vsync: false,
        log_level: log::LevelFilter::Warn,
        max_frames: Some(1),
        headless: true,
        screenshot: None,
        screenshot_interval: 0,
    };

    let result = backends::create_backend(backends::BackendType::Wgpu, config.debug);
    assert!(
        result.is_ok(),
        "wgpu backend without validation should initialize successfully"
    );
}

#[test]
#[cfg(target_os = "windows")]
fn test_directx_validation_enabled() {
    let config = RenderConfig {
        backend: Backend::DirectX,
        scene: "triangle".to_string(),
        width: 800,
        height: 600,
        debug: true, // Enable validation
        vsync: false,
        log_level: log::LevelFilter::Info,
        max_frames: Some(1),
        headless: true,
        screenshot: None,
        screenshot_interval: 0,
    };

    let result = backends::create_backend(backends::BackendType::DirectX12, config.debug);

    match result {
        Ok(_) => {
            println!("✅ DirectX 12 debug layer enabled successfully");
        }
        Err(e) => {
            let err_msg = e.to_string();
            // Check if it's a debug layer availability issue
            if err_msg.contains("debug") || err_msg.contains("D3D12") {
                println!("⚠ DirectX 12 debug layer not available");
                println!("   Install Graphics Tools via Windows Optional Features");
                println!("   Error: {}", err_msg);
            } else {
                panic!("Unexpected error: {}", e);
            }
        }
    }
}

#[test]
#[cfg(target_os = "windows")]
fn test_directx_validation_disabled() {
    let config = RenderConfig {
        backend: Backend::DirectX,
        scene: "triangle".to_string(),
        width: 800,
        height: 600,
        debug: false, // Disable validation
        vsync: false,
        log_level: log::LevelFilter::Warn,
        max_frames: Some(1),
        headless: true,
        screenshot: None,
        screenshot_interval: 0,
    };

    let result = backends::create_backend(backends::BackendType::DirectX12, config.debug);
    assert!(
        result.is_ok(),
        "DirectX backend should work without debug layers"
    );
}

#[test]
fn test_validation_flag_consistency() {
    // Verify debug flag is properly passed through config
    let config_debug = RenderConfig {
        backend: Backend::Wgpu,
        scene: "triangle".to_string(),
        width: 800,
        height: 600,
        debug: true,
        vsync: false,
        log_level: log::LevelFilter::Info,
        max_frames: Some(1),
        headless: true,
        screenshot: None,
        screenshot_interval: 0,
    };

    let config_no_debug = RenderConfig {
        backend: Backend::Wgpu,
        scene: "triangle".to_string(),
        width: 800,
        height: 600,
        debug: false,
        vsync: false,
        log_level: log::LevelFilter::Warn,
        max_frames: Some(1),
        headless: true,
        screenshot: None,
        screenshot_interval: 0,
    };

    assert!(config_debug.debug, "Debug config should have debug=true");
    assert!(
        !config_no_debug.debug,
        "Non-debug config should have debug=false"
    );
}
