use anyhow::{Context, Result};
use std::path::Path;

/// Loaded image data in RGBA8 format
#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA8 format (4 bytes per pixel)
}

/// Utility for loading texture images from files
pub struct TextureLoader;

impl TextureLoader {
    /// Load an image from a file path
    ///
    /// Supports PNG, JPEG, and other formats supported by the `image` crate.
    /// The image is automatically converted to RGBA8 format regardless of the source format.
    ///
    /// # Arguments
    /// * `path` - Path to the image file
    ///
    /// # Returns
    /// * `Ok(LoadedImage)` - Successfully loaded image data
    /// * `Err` - If the file cannot be read or decoded
    ///
    /// # Example
    /// ```no_run
    /// use rusty_renderer::resources::TextureLoader;
    /// use std::path::Path;
    ///
    /// let image = TextureLoader::load_from_file(Path::new("texture.png")).unwrap();
    /// println!("Loaded {}x{} texture", image.width, image.height);
    /// ```
    pub fn load_from_file(path: &Path) -> Result<LoadedImage> {
        // Load image using the image crate
        let img =
            image::open(path).with_context(|| format!("Failed to load image from {path:?}"))?;

        // Convert to RGBA8 format
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let data = rgba.into_raw();

        // Validate data size
        let expected_size = (width * height * 4) as usize;
        if data.len() != expected_size {
            anyhow::bail!(
                "Invalid image data size: expected {} bytes, got {}",
                expected_size,
                data.len()
            );
        }

        Ok(LoadedImage {
            width,
            height,
            data,
        })
    }

    /// Create a test checkerboard pattern
    ///
    /// Useful for testing and debugging texture functionality.
    ///
    /// # Arguments
    /// * `size` - Width and height of the square checkerboard
    /// * `checker_size` - Size of each checker square in pixels
    ///
    /// # Returns
    /// A LoadedImage containing an RGBA8 checkerboard pattern
    pub fn create_checkerboard(size: u32, checker_size: u32) -> LoadedImage {
        let mut data = Vec::with_capacity((size * size * 4) as usize);

        for y in 0..size {
            for x in 0..size {
                let checker_x = (x / checker_size) % 2;
                let checker_y = (y / checker_size) % 2;
                let is_white = (checker_x + checker_y).is_multiple_of(2);

                let color = if is_white {
                    [255, 255, 255, 255] // White
                } else {
                    [0, 0, 0, 255] // Black
                };

                data.extend_from_slice(&color);
            }
        }

        LoadedImage {
            width: size,
            height: size,
            data,
        }
    }

    /// Create a test gradient pattern
    ///
    /// Creates a horizontal gradient from black to white.
    ///
    /// # Arguments
    /// * `width` - Width of the gradient image
    /// * `height` - Height of the gradient image
    ///
    /// # Returns
    /// A LoadedImage containing an RGBA8 gradient pattern
    pub fn create_gradient(width: u32, height: u32) -> LoadedImage {
        let mut data = Vec::with_capacity((width * height * 4) as usize);

        for _y in 0..height {
            for x in 0..width {
                let intensity = (x as f32 / width as f32 * 255.0) as u8;
                data.extend_from_slice(&[intensity, intensity, intensity, 255]);
            }
        }

        LoadedImage {
            width,
            height,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkerboard_creation() {
        let image = TextureLoader::create_checkerboard(256, 32);

        assert_eq!(image.width, 256);
        assert_eq!(image.height, 256);
        assert_eq!(image.data.len(), 256 * 256 * 4);

        // Verify first pixel is white (top-left checker)
        assert_eq!(&image.data[0..4], &[255, 255, 255, 255]);

        // Verify pixel at (32, 0) is black (second checker in first row)
        let offset = (32 * 4) as usize;
        assert_eq!(&image.data[offset..offset + 4], &[0, 0, 0, 255]);
    }

    #[test]
    fn test_gradient_creation() {
        let image = TextureLoader::create_gradient(256, 128);

        assert_eq!(image.width, 256);
        assert_eq!(image.height, 128);
        assert_eq!(image.data.len(), 256 * 128 * 4);

        // Verify first pixel is black
        assert_eq!(&image.data[0..4], &[0, 0, 0, 255]);

        // Verify last pixel in first row is near white (rounding may not be exact 255)
        let offset = (255 * 4) as usize;
        let last_pixel = &image.data[offset..offset + 4];
        assert!(last_pixel[0] >= 254 && last_pixel[1] >= 254 && last_pixel[2] >= 254);
        assert_eq!(last_pixel[3], 255); // Alpha should be 255
    }

    #[test]
    fn test_invalid_file() {
        let result = TextureLoader::load_from_file(Path::new("nonexistent.png"));
        assert!(result.is_err());
    }

    #[test]
    fn test_data_size_consistency() {
        let image = TextureLoader::create_checkerboard(64, 8);
        let expected_size = (image.width * image.height * 4) as usize;
        assert_eq!(image.data.len(), expected_size);
    }
}
