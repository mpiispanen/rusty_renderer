//! Visual testing and image comparison utilities
//!
//! This module provides tools for automated visual testing, including
//! pixel-by-pixel image comparison, tolerance handling, and diff generation.
//!
//! It also includes integration with NVIDIA FLIP for industry-standard
//! perceptual image comparison.

pub mod flip;
pub mod image_compare;

pub use flip::{FlipComparator, FlipResult};
pub use image_compare::{ComparisonResult, ImageComparator};
