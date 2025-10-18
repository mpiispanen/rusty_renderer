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
    /// Structural Similarity Index (SSIM)
    pub ssim: f64,
    /// Perceptual error (inspired by FLIP)
    pub perceptual_error: f64,
}

impl ComparisonResult {
    /// Create a new comparison result
    pub fn new(diff_pixels: usize, total_pixels: usize, mse: f64, ssim: f64, perceptual_error: f64) -> Self {
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
            ssim,
            perceptual_error,
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
        let mut perceptual_sum = 0.0;

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
                
                // Calculate perceptual error (simplified FLIP-inspired metric)
                // Weight luminance changes more heavily than color
                let perceptual_diff = self.perceptual_difference(p1, p2);
                perceptual_sum += perceptual_diff;
            }
        }

        let mse = mse_sum / (total_pixels * 4) as f64; // 4 channels (RGBA)
        let perceptual_error = perceptual_sum / total_pixels as f64;
        
        // Calculate SSIM in windows
        let ssim = self.calculate_ssim(img1, img2);
        
        let result = ComparisonResult::new(diff_pixels, total_pixels, mse, ssim, perceptual_error);

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
    
    /// Calculate perceptual difference between two pixels (inspired by FLIP)
    /// Uses luminance-weighted difference with human visual sensitivity
    fn perceptual_difference(&self, p1: &Rgba<u8>, p2: &Rgba<u8>) -> f64 {
        // Convert to linear RGB and then to luminance
        // Using ITU-R BT.709 coefficients for RGB to luminance conversion
        let to_linear = |v: u8| {
            let v = v as f64 / 255.0;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        
        let r1 = to_linear(p1[0]);
        let g1 = to_linear(p1[1]);
        let b1 = to_linear(p1[2]);
        let lum1 = 0.2126 * r1 + 0.7152 * g1 + 0.0722 * b1;
        
        let r2 = to_linear(p2[0]);
        let g2 = to_linear(p2[1]);
        let b2 = to_linear(p2[2]);
        let lum2 = 0.2126 * r2 + 0.7152 * g2 + 0.0722 * b2;
        
        // Luminance difference (weighted heavily as humans are sensitive to it)
        let lum_diff = (lum1 - lum2).abs();
        
        // Color difference (chromatic aberration)
        let color_diff = ((r1 - r2).powi(2) + (g1 - g2).powi(2) + (b1 - b2).powi(2)).sqrt();
        
        // Combine with weights (luminance is more important)
        3.0 * lum_diff + color_diff
    }
    
    /// Calculate SSIM (Structural Similarity Index) between two images
    /// Uses a simplified window-based approach
    fn calculate_ssim(&self, img1: &RgbaImage, img2: &RgbaImage) -> f64 {
        let (width, height) = img1.dimensions();
        let window_size = 11u32;
        let half_window = window_size / 2;
        
        // Constants for SSIM calculation
        let c1 = (0.01_f64 * 255.0).powi(2);
        let c2 = (0.03_f64 * 255.0).powi(2);
        
        let mut ssim_sum = 0.0;
        let mut count = 0;
        
        // Slide window across image
        for y in half_window..(height - half_window) {
            for x in half_window..(width - half_window) {
                let (mean1, var1) = self.window_stats(img1, x, y, window_size);
                let (mean2, var2) = self.window_stats(img2, x, y, window_size);
                let covar = self.window_covariance(img1, img2, x, y, window_size, mean1, mean2);
                
                // SSIM formula
                let numerator = (2.0 * mean1 * mean2 + c1) * (2.0 * covar + c2);
                let denominator = (mean1.powi(2) + mean2.powi(2) + c1) * (var1 + var2 + c2);
                
                ssim_sum += numerator / denominator;
                count += 1;
            }
        }
        
        if count > 0 {
            ssim_sum / count as f64
        } else {
            1.0
        }
    }
    
    /// Calculate mean and variance for a window
    fn window_stats(&self, img: &RgbaImage, cx: u32, cy: u32, window_size: u32) -> (f64, f64) {
        let half = window_size / 2;
        let mut sum = 0.0;
        let mut sq_sum = 0.0;
        let mut count = 0;
        
        for y in cy.saturating_sub(half)..=(cy + half).min(img.height() - 1) {
            for x in cx.saturating_sub(half)..=(cx + half).min(img.width() - 1) {
                let pixel = img.get_pixel(x, y);
                let gray = (pixel[0] as f64 + pixel[1] as f64 + pixel[2] as f64) / 3.0;
                sum += gray;
                sq_sum += gray * gray;
                count += 1;
            }
        }
        
        let mean = sum / count as f64;
        let variance = sq_sum / count as f64 - mean * mean;
        (mean, variance)
    }
    
    /// Calculate covariance between two windows
    fn window_covariance(&self, img1: &RgbaImage, img2: &RgbaImage, cx: u32, cy: u32, 
                        window_size: u32, mean1: f64, mean2: f64) -> f64 {
        let half = window_size / 2;
        let mut covar = 0.0;
        let mut count = 0;
        
        for y in cy.saturating_sub(half)..=(cy + half).min(img1.height() - 1) {
            for x in cx.saturating_sub(half)..=(cx + half).min(img1.width() - 1) {
                let p1 = img1.get_pixel(x, y);
                let p2 = img2.get_pixel(x, y);
                let gray1 = (p1[0] as f64 + p1[1] as f64 + p1[2] as f64) / 3.0;
                let gray2 = (p2[0] as f64 + p2[1] as f64 + p2[2] as f64) / 3.0;
                covar += (gray1 - mean1) * (gray2 - mean2);
                count += 1;
            }
        }
        
        covar / count as f64
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
        assert_eq!(result.perceptual_error, 0.0);
        assert!((result.ssim - 1.0).abs() < 0.01); // SSIM should be ~1.0 for identical images
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
        assert!(result.perceptual_error > 0.0);
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
    
    #[test]
    fn test_perceptual_metrics() {
        // Create images with subtle differences
        let img1 = RgbaImage::from_pixel(20, 20, Rgba([128, 128, 128, 255]));
        let mut img2 = img1.clone();
        
        // Modify a few pixels slightly
        for y in 5..15 {
            for x in 5..15 {
                img2.put_pixel(x, y, Rgba([130, 126, 129, 255]));
            }
        }

        let comparator = ImageComparator::new(100.0, 5);
        let result = comparator.compare(&img1, &img2).unwrap();

        // Should detect difference but with reasonable metrics
        assert!(result.perceptual_error < 1.0); // Small perceptual difference
        assert!(result.ssim > 0.9); // High structural similarity
        assert!(result.psnr > 30.0); // Good PSNR
    }
}
