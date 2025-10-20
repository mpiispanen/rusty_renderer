//! Unified application framework
//!
//! This module provides the main application that integrates:
//! - Scene loading
//! - Pipeline selection
//! - Backend initialization
//! - Event loop management

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

pub mod runner;

pub use runner::ApplicationRunner;

/// Command-line arguments for the unified application
#[derive(Parser, Debug)]
#[command(name = "rusty_renderer")]
#[command(about = "A multi-backend 3D renderer", long_about = None)]
pub struct ApplicationArgs {
    /// Scene file to load
    #[arg(short, long, value_name = "FILE")]
    pub scene: Option<PathBuf>,

    /// Pipeline to use (simple, forward, deferred, etc.)
    #[arg(short, long, default_value = "simple")]
    pub pipeline: String,

    /// Graphics backend to use
    #[arg(short, long, value_name = "BACKEND")]
    pub backend: Option<String>,

    /// Run in headless mode (no window)
    #[arg(long)]
    pub headless: bool,

    /// List available scenes
    #[arg(long)]
    pub list_scenes: bool,

    /// List available pipelines
    #[arg(long)]
    pub list_pipelines: bool,

    /// Window width
    #[arg(long, default_value = "800")]
    pub width: u32,

    /// Window height
    #[arg(long, default_value = "600")]
    pub height: u32,

    /// Maximum frames to render (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub max_frames: u32,

    /// Screenshot output path (for headless mode)
    #[arg(long, value_name = "FILE")]
    pub screenshot: Option<PathBuf>,
}

impl ApplicationArgs {
    /// Parse arguments from command line
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Validate arguments
    pub fn validate(&self) -> Result<()> {
        // If scene is specified, it must exist
        if let Some(scene_path) = &self.scene {
            if !scene_path.exists() {
                anyhow::bail!("Scene file not found: {}", scene_path.display());
            }
        }

        // If headless, we need a scene
        if self.headless && self.scene.is_none() && !self.list_scenes && !self.list_pipelines {
            anyhow::bail!("Headless mode requires --scene argument");
        }

        Ok(())
    }

    /// Get the scene directory
    pub fn scene_directory(&self) -> PathBuf {
        PathBuf::from("scenes")
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_default_args() {
        // Can't easily test clap parsing without actual args
        // This would need integration tests
    }
}
