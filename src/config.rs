//! Configuration management
//!
//! This module handles command-line argument parsing and application configuration.

use clap::Parser;

/// Graphics backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Backend {
    /// Vulkan backend (using vulkanalia)
    Vulkan,
    /// DirectX 12 backend (Windows only)
    #[cfg(target_os = "windows")]
    DirectX,
    /// wgpu backend (cross-platform)
    Wgpu,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Backend::Vulkan => write!(f, "Vulkan"),
            #[cfg(target_os = "windows")]
            Backend::DirectX => write!(f, "DirectX 12"),
            Backend::Wgpu => write!(f, "wgpu"),
        }
    }
}

/// Rusty Renderer - A multi-backend 3D renderer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    /// Graphics backend to use
    #[arg(short, long, value_enum, default_value = "vulkan")]
    pub backend: Backend,
    
    /// Scene to render (triangle, cube, etc.)
    #[arg(short, long, default_value = "triangle")]
    pub scene: String,

    /// Window width
    #[arg(long, default_value = "1280")]
    pub width: u32,

    /// Window height
    #[arg(long, default_value = "720")]
    pub height: u32,

    /// Enable debug mode and validation layers
    #[arg(short, long, default_value = "false")]
    pub debug: bool,

    /// Enable VSync
    #[arg(long, default_value = "true")]
    pub vsync: bool,

    /// Log level (off, error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    pub log_level: log::LevelFilter,

    /// Maximum number of frames to render (for testing)
    #[arg(long)]
    pub max_frames: Option<u64>,
}

impl Config {
    /// Parse command-line arguments
    pub fn parse_args() -> Self {
        Config::parse()
    }

    /// Validate configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.width == 0 || self.height == 0 {
            anyhow::bail!("Window dimensions must be greater than 0");
        }

        if self.width > 7680 || self.height > 4320 {
            log::warn!(
                "Large window size requested: {}x{} (may cause performance issues)",
                self.width,
                self.height
            );
        }

        // Warn about platform-specific backend issues
        #[cfg(not(target_os = "windows"))]
        {
            if matches!(self.backend, Backend::Vulkan | Backend::Wgpu) {
                // Valid backends on non-Windows platforms
            }
        }

        Ok(())
    }

    /// Get the window size as a tuple
    pub fn window_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let mut config = Config {
            backend: Backend::Vulkan,
            scene: "triangle".to_string(),
            width: 1280,
            height: 720,
            debug: false,
            vsync: true,
            log_level: log::LevelFilter::Info,
            max_frames: None,
        };

        assert!(config.validate().is_ok());

        // Test invalid dimensions
        config.width = 0;
        assert!(config.validate().is_err());

        config.width = 1280;
        config.height = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_window_size() {
        let config = Config {
            backend: Backend::Vulkan,
            scene: "triangle".to_string(),
            width: 1920,
            height: 1080,
            debug: false,
            vsync: true,
            log_level: log::LevelFilter::Info,
            max_frames: None,
        };

        assert_eq!(config.window_size(), (1920, 1080));
    }
}
