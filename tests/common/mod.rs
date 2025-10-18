//! Common test utilities and helpers
//!
//! This module provides shared functionality for integration tests.

/// Initialize test environment with logging
pub fn setup_test_env() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}

/// Create a test configuration with custom parameters
pub fn test_config(
    backend: rusty_renderer::config::Backend,
    width: u32,
    height: u32,
) -> rusty_renderer::config::Config {
    rusty_renderer::config::Config {
        backend,
        scene: "test".to_string(),
        width,
        height,
        debug: true,
        vsync: false,
        log_level: log::LevelFilter::Debug,
        max_frames: None,
    }
}

/// Create a default test configuration
pub fn default_test_config() -> rusty_renderer::config::Config {
    test_config(rusty_renderer::config::Backend::Vulkan, 800, 600)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_test_env() {
        // Should not panic
        setup_test_env();
    }

    #[test]
    fn test_config_creation() {
        let config = test_config(rusty_renderer::config::Backend::Wgpu, 1024, 768);
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert_eq!(config.backend, rusty_renderer::config::Backend::Wgpu);
        assert!(config.debug);
        assert!(!config.vsync);
    }

    #[test]
    fn test_default_config() {
        let config = default_test_config();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.backend, rusty_renderer::config::Backend::Vulkan);
    }
}
