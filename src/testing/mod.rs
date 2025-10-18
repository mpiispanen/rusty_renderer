//! Visual testing and image comparison utilities
//!
//! This module provides tools for automated visual testing, including
//! pixel-by-pixel image comparison, tolerance handling, and diff generation.

pub mod image_compare;

pub use image_compare::{ComparisonResult, ImageComparator};
