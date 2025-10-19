//! NVIDIA FLIP integration for perceptual image comparison
//!
//! This module provides a wrapper around the FLIP Python tool for
//! accurate perceptual image comparison using the industry-standard metric.
//!
//! Two methods are supported:
//! 1. Command-line `flip` tool (original implementation)
//! 2. Python script wrapper for direct API access (JSON output)

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Result of a FLIP comparison
#[derive(Debug, Clone)]
pub struct FlipResult {
    /// Mean FLIP error (0.0 to 1.0, lower is better)
    pub mean: f64,
    /// Median FLIP error
    pub median: f64,
    /// 1st quartile
    pub q1: f64,
    /// 3rd quartile
    pub q3: f64,
    /// Minimum error value
    pub min: f64,
    /// Maximum error value
    pub max: f64,
    /// Path to generated error map
    pub error_map_path: Option<String>,
    /// Pixels per degree used
    pub ppd: f64,
}

impl FlipResult {
    /// Check if the comparison passes a given threshold
    /// Recommended thresholds:
    /// - < 0.05: Excellent match
    /// - < 0.10: Good match
    /// - < 0.15: Acceptable match
    /// - > 0.15: Significant differences
    pub fn passes(&self, threshold: f64) -> bool {
        self.mean < threshold
    }

    /// Get the weighted median (for compatibility)
    pub fn weighted_median(&self) -> f64 {
        self.median
    }
}

/// FLIP comparison tool
pub struct FlipComparator {
    /// Pixels per degree (default: 67 for 0.7m viewing distance on 4K display)
    pub pixels_per_degree: Option<f64>,
    /// Verbosity level (0-2)
    pub verbosity: u8,
    /// Use Python script API (JSON output) instead of CLI (default: false)
    pub use_python_api: bool,
}

impl Default for FlipComparator {
    fn default() -> Self {
        Self {
            pixels_per_degree: None, // Use FLIP's default (67)
            verbosity: 2,
            use_python_api: false,
        }
    }
}

impl FlipComparator {
    /// Create a new FLIP comparator with custom settings
    pub fn new(pixels_per_degree: Option<f64>, verbosity: u8) -> Self {
        Self {
            pixels_per_degree,
            verbosity,
            use_python_api: false,
        }
    }

    /// Create a new FLIP comparator using the Python API
    pub fn with_python_api(pixels_per_degree: Option<f64>, verbosity: u8) -> Self {
        Self {
            pixels_per_degree,
            verbosity,
            use_python_api: true,
        }
    }

    /// Compare two images using FLIP
    pub fn compare<P: AsRef<Path>>(&self, reference: P, test: P) -> Result<FlipResult> {
        if self.use_python_api {
            self.compare_with_python_api(reference, test)
        } else {
            self.compare_with_cli(reference, test)
        }
    }

    /// Compare using the Python API script (returns JSON)
    fn compare_with_python_api<P: AsRef<Path>>(&self, reference: P, test: P) -> Result<FlipResult> {
        let reference_path = reference.as_ref();
        let test_path = test.as_ref();

        // Check that files exist
        if !reference_path.exists() {
            anyhow::bail!("Reference image not found: {}", reference_path.display());
        }
        if !test_path.exists() {
            anyhow::bail!("Test image not found: {}", test_path.display());
        }

        // Find the Python script
        let script_path = std::env::current_dir()?
            .join("scripts")
            .join("flip_compare.py");

        if !script_path.exists() {
            anyhow::bail!(
                "Python FLIP script not found at: {}. Use CLI method instead.",
                script_path.display()
            );
        }

        // Build command
        let mut cmd = Command::new("python3");
        cmd.arg(&script_path)
            .arg(reference_path)
            .arg(test_path)
            .arg("--verbosity")
            .arg(self.verbosity.to_string());

        if let Some(ppd) = self.pixels_per_degree {
            cmd.arg("--ppd").arg(ppd.to_string());
        }

        // Generate error map
        let error_map_path = test_path.with_file_name(format!(
            "flip_error_{}.png",
            test_path.file_stem().unwrap().to_string_lossy()
        ));
        cmd.arg("--error-map").arg(&error_map_path);

        // Execute
        let output = cmd
            .output()
            .context("Failed to execute Python FLIP script. Is python3 available?")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("FLIP Python script failed: {stderr}");
        }

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_json_output(&stdout, Some(error_map_path.to_string_lossy().to_string()))
    }

    /// Compare using the CLI tool (original implementation)
    fn compare_with_cli<P: AsRef<Path>>(&self, reference: P, test: P) -> Result<FlipResult> {
        let reference_path = reference.as_ref();
        let test_path = test.as_ref();

        // Check that files exist
        if !reference_path.exists() {
            anyhow::bail!("Reference image not found: {}", reference_path.display());
        }
        if !test_path.exists() {
            anyhow::bail!("Test image not found: {}", test_path.display());
        }

        // Build FLIP command
        let mut cmd = Command::new("flip");
        cmd.arg("-r")
            .arg(reference_path)
            .arg("-t")
            .arg(test_path)
            .arg("-v")
            .arg(self.verbosity.to_string());

        if let Some(ppd) = self.pixels_per_degree {
            cmd.arg("-ppd").arg(ppd.to_string());
        }

        // Execute FLIP
        let output = cmd.output().context(
            "Failed to execute FLIP. Is 'flip-evaluator' installed? (pip install flip-evaluator)",
        )?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("FLIP execution failed: {stderr}");
        }

        // Parse output
        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_cli_output(&stdout)
    }

    /// Parse JSON output from Python script
    fn parse_json_output(
        &self,
        output: &str,
        error_map_path: Option<String>,
    ) -> Result<FlipResult> {
        // Find JSON in output (skip stderr messages)
        let json_start = output.find('{').context("No JSON found in output")?;
        let json_str = &output[json_start..];

        let json: serde_json::Value =
            serde_json::from_str(json_str).context("Failed to parse JSON output")?;

        Ok(FlipResult {
            mean: json["mean"].as_f64().context("Missing mean")?,
            median: json["median"].as_f64().context("Missing median")?,
            q1: json["q1"].as_f64().context("Missing q1")?,
            q3: json["q3"].as_f64().context("Missing q3")?,
            min: json["min"].as_f64().context("Missing min")?,
            max: json["max"].as_f64().context("Missing max")?,
            ppd: json["ppd"].as_f64().context("Missing ppd")?,
            error_map_path: error_map_path.or_else(|| json["error_map"].as_str().map(String::from)),
        })
    }

    /// Parse CLI output to extract metrics
    fn parse_cli_output(&self, output: &str) -> Result<FlipResult> {
        let mut mean = None;
        let mut weighted_median = None;
        let mut q1 = None;
        let mut q3 = None;
        let mut min = None;
        let mut max = None;
        let mut ppd = 67.0; // default
        let mut error_map_path = None;

        for line in output.lines() {
            let line = line.trim();

            if line.contains("Mean:") {
                mean = line
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse::<f64>().ok());
            } else if line.contains("Weighted median:") {
                weighted_median = line
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse::<f64>().ok());
            } else if line.contains("1st weighted quartile:") {
                q1 = line
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse::<f64>().ok());
            } else if line.contains("3rd weighted quartile:") {
                q3 = line
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse::<f64>().ok());
            } else if line.contains("Min:") {
                min = line
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse::<f64>().ok());
            } else if line.contains("Max:") {
                max = line
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse::<f64>().ok());
            } else if line.contains("Pixels per degree:") {
                ppd = line
                    .split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .unwrap_or(67.0);
            } else if line.contains("FLIP error map location:") {
                error_map_path = line.split(':').nth(1).map(|s| s.trim().to_string());
            }
        }

        // Validate we got all required metrics
        let mean = mean.context("Failed to parse mean FLIP error")?;
        let median = weighted_median.context("Failed to parse weighted median FLIP error")?;
        let q1 = q1.context("Failed to parse 1st quartile FLIP error")?;
        let q3 = q3.context("Failed to parse 3rd quartile FLIP error")?;
        let min = min.context("Failed to parse min FLIP error")?;
        let max = max.context("Failed to parse max FLIP error")?;

        Ok(FlipResult {
            mean,
            median,
            q1,
            q3,
            min,
            max,
            error_map_path,
            ppd,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    use std::path::PathBuf;

    fn test_output_dir() -> PathBuf {
        let dir = PathBuf::from("target/flip_tests");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    #[ignore] // Requires FLIP to be installed
    fn test_identical_images() {
        let output_dir = test_output_dir();
        let img1_path = output_dir.join("identical1.png");
        let img2_path = output_dir.join("identical2.png");

        // Create identical images
        let img = RgbaImage::from_pixel(100, 100, Rgba([128, 128, 128, 255]));
        img.save(&img1_path).unwrap();
        img.save(&img2_path).unwrap();

        let comparator = FlipComparator::default();
        let result = comparator.compare(&img1_path, &img2_path).unwrap();

        // Identical images should have very low FLIP error
        assert!(result.mean < 0.001, "Mean error: {}", result.mean);
        assert!(result.passes(0.01));
    }

    #[test]
    #[ignore] // Requires FLIP to be installed
    fn test_different_images() {
        let output_dir = test_output_dir();
        let img1_path = output_dir.join("different1.png");
        let img2_path = output_dir.join("different2.png");

        // Create different images
        let img1 = RgbaImage::from_pixel(100, 100, Rgba([255, 0, 0, 255]));
        let img2 = RgbaImage::from_pixel(100, 100, Rgba([0, 255, 0, 255]));
        img1.save(&img1_path).unwrap();
        img2.save(&img2_path).unwrap();

        let comparator = FlipComparator::default();
        let result = comparator.compare(&img1_path, &img2_path).unwrap();

        // Different images should have high FLIP error
        assert!(result.mean > 0.1, "Mean error: {}", result.mean);
        assert!(!result.passes(0.05));
    }
}
