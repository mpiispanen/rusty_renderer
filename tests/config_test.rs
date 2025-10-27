//! Integration tests for configuration module
//!
//! These tests verify that configuration parsing and validation
//! work correctly in an integrated environment.

mod common;

use rusty_renderer::config::{Backend, Config};

#[test]
fn test_config_validation_valid() {
    common::setup_test_env();

    let config = Config {
        backend: Backend::Vulkan,
        scene: "test".to_string(),
        width: 1920,
        height: 1080,
        debug: false,
        vsync: true,
        log_level: log::LevelFilter::Info,
        max_frames: None,
        headless: false,
        screenshot: None,
        screenshot_interval: 0,
    };

    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validation_zero_width() {
    common::setup_test_env();

    let config = Config {
        backend: Backend::Vulkan,
        scene: "test".to_string(),
        width: 0,
        height: 720,
        debug: false,
        vsync: true,
        log_level: log::LevelFilter::Info,
        max_frames: None,
        headless: false,
        screenshot: None,
        screenshot_interval: 0,
    };

    assert!(config.validate().is_err());
}

#[test]
fn test_config_validation_zero_height() {
    common::setup_test_env();

    let config = Config {
        backend: Backend::Vulkan,
        scene: "test".to_string(),
        width: 1280,
        height: 0,
        debug: false,
        vsync: true,
        log_level: log::LevelFilter::Info,
        max_frames: None,
        headless: false,
        screenshot: None,
        screenshot_interval: 0,
    };

    assert!(config.validate().is_err());
}

#[test]
fn test_config_window_size() {
    common::setup_test_env();
    let config = common::test_config(Backend::Vulkan, 1024, 768);
    assert_eq!(config.window_size(), (1024, 768));
}

#[test]
fn test_backend_display() {
    common::setup_test_env();

    assert_eq!(Backend::Vulkan.to_string(), "Vulkan");
}
