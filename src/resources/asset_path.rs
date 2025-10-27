//! Asset path resolution
//!
//! Handles resolving asset paths relative to the project root or scene directory.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Asset path resolver
///
/// Resolves asset paths from scene files and handles relative/absolute paths correctly.
pub struct AssetPathResolver {
    /// Project root directory (where Cargo.toml is located)
    project_root: PathBuf,
}

impl AssetPathResolver {
    /// Create a new asset path resolver
    pub fn new() -> Result<Self> {
        // Get the project root by looking for Cargo.toml
        let project_root = Self::find_project_root().context("Failed to find project root")?;

        Ok(Self { project_root })
    }

    /// Create with explicit project root
    pub fn with_root<P: Into<PathBuf>>(root: P) -> Self {
        Self {
            project_root: root.into(),
        }
    }

    /// Find the project root by searching for Cargo.toml
    fn find_project_root() -> Result<PathBuf> {
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;

        let mut path = current_dir.as_path();

        loop {
            let cargo_toml = path.join("Cargo.toml");
            if cargo_toml.exists() {
                return Ok(path.to_path_buf());
            }

            match path.parent() {
                Some(parent) => path = parent,
                None => anyhow::bail!("Could not find project root (no Cargo.toml found)"),
            }
        }
    }

    /// Resolve an asset path
    ///
    /// Handles several path formats:
    /// - Absolute paths: used as-is
    /// - Paths starting with "assets/": resolved relative to project root
    /// - Paths starting with "./": resolved relative to scene directory
    /// - Other relative paths: resolved relative to project root
    pub fn resolve(&self, path: &str, scene_dir: Option<&Path>) -> PathBuf {
        let path_obj = Path::new(path);

        // If absolute, use as-is
        if path_obj.is_absolute() {
            return path_obj.to_path_buf();
        }

        // If starts with "./", resolve relative to scene directory
        if path.starts_with("./") || path.starts_with(".\\") {
            if let Some(scene_dir) = scene_dir {
                return scene_dir.join(&path[2..]);
            }
        }

        // Otherwise, resolve relative to project root
        self.project_root.join(path)
    }

    /// Resolve an asset path and verify it exists
    pub fn resolve_and_verify(&self, path: &str, scene_dir: Option<&Path>) -> Result<PathBuf> {
        let resolved = self.resolve(path, scene_dir);

        if !resolved.exists() {
            anyhow::bail!(
                "Asset not found: '{}' (resolved to '{}')",
                path,
                resolved.display()
            );
        }

        Ok(resolved)
    }

    /// Get the project root path
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Get the assets directory path
    pub fn assets_dir(&self) -> PathBuf {
        self.project_root.join("assets")
    }
}

impl Default for AssetPathResolver {
    fn default() -> Self {
        Self::new().expect("Failed to create default AssetPathResolver")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_project_root() {
        let root = AssetPathResolver::find_project_root();
        assert!(root.is_ok());

        let root = root.unwrap();
        assert!(root.join("Cargo.toml").exists());
    }

    #[test]
    fn test_resolve_absolute() {
        let resolver = AssetPathResolver::with_root("/project");
        let path = "/absolute/path/to/asset.png";

        let resolved = resolver.resolve(path, None);
        assert_eq!(resolved, PathBuf::from(path));
    }

    #[test]
    fn test_resolve_assets() {
        let resolver = AssetPathResolver::with_root("/project");
        let path = "assets/textures/test.png";

        let resolved = resolver.resolve(path, None);
        assert_eq!(resolved, PathBuf::from("/project/assets/textures/test.png"));
    }

    #[test]
    fn test_resolve_relative_to_scene() {
        let resolver = AssetPathResolver::with_root("/project");
        let path = "./texture.png";
        let scene_dir = Path::new("/project/scenes/my_scene");

        let resolved = resolver.resolve(path, Some(scene_dir));
        assert_eq!(
            resolved,
            PathBuf::from("/project/scenes/my_scene/texture.png")
        );
    }
}
