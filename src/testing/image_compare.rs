//! Image comparison utilities for visual testing

use anyhow::{Context, Result};
use image::{ImageBuffer, Rgba, RgbaImage};
use std::path::Path;

/// Result of an image comparison
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    /// Whether the images match within tolerance
    pub matches: bool,
    /// Number of pixels that differ
    pub diff_pixels: usize,
    /// Total number of pixels
    pub total_pixels: usize,
    /// Percentage of pixels that differ
    pub diff_percentage: f64,
    /// Mean squared error
    pub mse: f64,
    /// Peak signal-to-noise ratio (dB)
    pub psnr: f64,
}

impl ComparisonResult {
    /// Create a new comparison result
    pub fn new(diff_pixels: usize, total_pixels: usize, mse: f64) -> Self {
        let diff_percentage = (diff_pixels as f64 / total_pixels as f64) * 100.0;
        
        // Calculate PSNR (Peak Signal-to-Noise Ratio)
        // PSNR = 10 * log10(MAX^2 / MSE)
        // For 8-bit images, MAX = 255
        let psnr = if mse > 0.0 {
            10.0 * ((255.0 * 255.0) / mse).log10()
        } else {
            f64::INFINITY
        };

        Self {
            matches: diff_pixels == 0,
            diff_pixels,
            total_pixels,
            diff_percentage,
            mse,
            psnr,
        }
    }
}

/// Image comparator with configurable tolerance
pub struct ImageComparator {
    /// Maximum allowed percentage of different pixels
    pub tolerance_percentage: f64,
    /// Per-channel tolerance (0-255)
    pub pixel_tolerance: u8,
}

impl Default for ImageComparator {
    fn default() -> Self {
        Self {
            tolerance_percentage: 0.0, // Exact match by default
            pixel_tolerance: 0,         // No tolerance by default
        }
    }
}

impl ImageComparator {
    /// Create a new comparator with custom tolerance
    pub fn new(tolerance_percentage: f64, pixel_tolerance: u8) -> Self {
        Self {
            tolerance_percentage,
            pixel_tolerance,
        }
    }

    /// Compare two images and return detailed results
    pub fn compare(&self, img1: &RgbaImage, img2: &RgbaImage) -> Result<ComparisonResult> {
        // Check dimensions match
        if img1.dimensions() != img2.dimensions() {
            anyhow::bail!(
                "Image dimensions don't match: {:?} vs {:?}",
                img1.dimensions(),
                img2.dimensions()
            );
        }

        let (width, height) = img1.dimensions();
        let total_pixels = (width * height) as usize;
        let mut diff_pixels = 0;
        let mut mse_sum = 0.0;

        // Compare pixel by pixel
        for y in 0..height {
            for x in 0..width {
                let p1 = img1.get_pixel(x, y);
                let p2 = img2.get_pixel(x, y);

                if !self.pixels_match(p1, p2) {
                    diff_pixels += 1;
                }

                // Calculate squared error for MSE
                for i in 0..4 {
                    let diff = (p1[i] as f64 - p2[i] as f64).abs();
                    mse_sum += diff * diff;
                }
            }
        }

        let mse = mse_sum / (total_pixels * 4) as f64; // 4 channels (RGBA)
        let result = ComparisonResult::new(diff_pixels, total_pixels, mse);

        Ok(result)
    }

    /// Check if comparison result is within tolerance
    pub fn is_within_tolerance(&self, result: &ComparisonResult) -> bool {
        result.diff_percentage <= self.tolerance_percentage
    }

    /// Compare two images from files
    pub fn compare_files<P: AsRef<Path>>(
        &self,
        path1: P,
        path2: P,
    ) -> Result<ComparisonResult> {
        let img1 = image::open(path1.as_ref())
            .with_context(|| format!("Failed to open {}", path1.as_ref().display()))?
            .to_rgba8();

        let img2 = image::open(path2.as_ref())
            .with_context(|| format!("Failed to open {}", path2.as_ref().display()))?
            .to_rgba8();

        self.compare(&img1, &img2)
    }

    /// Generate a diff image highlighting differences
    pub fn generate_diff(&self, img1: &RgbaImage, img2: &RgbaImage) -> Result<RgbaImage> {
        // Check dimensions match
        if img1.dimensions() != img2.dimensions() {
            anyhow::bail!(
                "Image dimensions don't match: {:?} vs {:?}",
                img1.dimensions(),
                img2.dimensions()
            );
        }

        let (width, height) = img1.dimensions();
        let mut diff_img = ImageBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let p1 = img1.get_pixel(x, y);
                let p2 = img2.get_pixel(x, y);

                let pixel = if self.pixels_match(p1, p2) {
                    // Matching pixels: show as grayscale
                    let gray = ((p1[0] as u16 + p1[1] as u16 + p1[2] as u16) / 3) as u8;
                    Rgba([gray, gray, gray, 255])
                } else {
                    // Different pixels: highlight in red
                    Rgba([255, 0, 0, 255])
                };

                diff_img.put_pixel(x, y, pixel);
            }
        }

        Ok(diff_img)
    }

    /// Generate and save a diff image to file
    pub fn generate_diff_file<P: AsRef<Path>>(
        &self,
        img1: &RgbaImage,
        img2: &RgbaImage,
        output_path: P,
    ) -> Result<()> {
        let diff = self.generate_diff(img1, img2)?;
        diff.save(output_path.as_ref())
            .with_context(|| format!("Failed to save diff to {}", output_path.as_ref().display()))?;
        Ok(())
    }

    /// Check if two pixels match within tolerance
    fn pixels_match(&self, p1: &Rgba<u8>, p2: &Rgba<u8>) -> bool {
        for i in 0..4 {
            let diff = (p1[i] as i16 - p2[i] as i16).unsigned_abs() as u8;
            if diff > self.pixel_tolerance {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_images() {
        let img1 = RgbaImage::from_pixel(10, 10, Rgba([255, 0, 0, 255]));
        let img2 = RgbaImage::from_pixel(10, 10, Rgba([255, 0, 0, 255]));

        let comparator = ImageComparator::default();
        let result = comparator.compare(&img1, &img2).unwrap();

        assert!(result.matches);
        assert_eq!(result.diff_pixels, 0);
        assert_eq!(result.diff_percentage, 0.0);
        assert_eq!(result.mse, 0.0);
        assert_eq!(result.psnr, f64::INFINITY);
    }

    #[test]
    fn test_different_images() {
        let img1 = RgbaImage::from_pixel(10, 10, Rgba([255, 0, 0, 255]));
        let img2 = RgbaImage::from_pixel(10, 10, Rgba([0, 255, 0, 255]));

        let comparator = ImageComparator::default();
        let result = comparator.compare(&img1, &img2).unwrap();

        assert!(!result.matches);
        assert_eq!(result.diff_pixels, 100);
        assert_eq!(result.diff_percentage, 100.0);
        assert!(result.mse > 0.0);
    }

    #[test]
    fn test_tolerance() {
        let img1 = RgbaImage::from_pixel(10, 10, Rgba([255, 0, 0, 255]));
        let img2 = RgbaImage::from_pixel(10, 10, Rgba([250, 0, 0, 255])); // 5 unit difference

        // No tolerance - should fail
        let comparator = ImageComparator::default();
        let result = comparator.compare(&img1, &img2).unwrap();
        assert!(!result.matches);

        // With tolerance - should pass
        let comparator = ImageComparator::new(100.0, 10);
        let result = comparator.compare(&img1, &img2).unwrap();
        assert!(comparator.is_within_tolerance(&result));
    }

    #[test]
    fn test_dimension_mismatch() {
        let img1 = RgbaImage::from_pixel(10, 10, Rgba([255, 0, 0, 255]));
        let img2 = RgbaImage::from_pixel(5, 5, Rgba([255, 0, 0, 255]));

        let comparator = ImageComparator::default();
        let result = comparator.compare(&img1, &img2);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("dimensions"));
    }
}
