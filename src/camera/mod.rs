//! Camera system for 3D scene navigation
//!
//! Provides camera controllers with view/projection matrix calculation
//! and interactive controls for scene navigation.

pub mod controller;

pub use controller::{CameraController, CameraUniforms};

use glam::{Mat4, Vec3};

/// Calculate perspective projection matrix
pub fn perspective_projection(fov_degrees: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4 {
    Mat4::perspective_rh(fov_degrees.to_radians(), aspect_ratio, near, far)
}

/// Calculate view matrix from position and target
pub fn look_at_view(position: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    Mat4::look_at_rh(position, target, up)
}

/// Calculate view matrix from position, yaw, and pitch (free-fly camera)
pub fn free_fly_view(position: Vec3, yaw: f32, pitch: f32) -> Mat4 {
    let yaw_rad = yaw.to_radians();
    let pitch_rad = pitch.to_radians();

    // Calculate forward direction from yaw and pitch
    let forward = Vec3::new(
        yaw_rad.cos() * pitch_rad.cos(),
        pitch_rad.sin(),
        yaw_rad.sin() * pitch_rad.cos(),
    )
    .normalize();

    let target = position + forward;
    let up = Vec3::Y;

    Mat4::look_at_rh(position, target, up)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perspective_projection() {
        let proj = perspective_projection(45.0, 16.0 / 9.0, 0.1, 100.0);
        assert!(!proj.is_nan());
    }

    #[test]
    fn test_look_at_view() {
        let view = look_at_view(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
        );
        assert!(!view.is_nan());
    }

    #[test]
    fn test_free_fly_view() {
        let view = free_fly_view(Vec3::new(0.0, 0.0, 5.0), -90.0, 0.0);
        assert!(!view.is_nan());
    }
}
