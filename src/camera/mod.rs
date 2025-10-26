//! Camera system for 3D scene navigation
//!
//! Provides camera controllers with view/projection matrix calculation
//! and interactive controls for scene navigation.

pub mod controller;

pub use controller::{CameraController, CameraUniforms};

use glam::{Mat4, Vec3};

/// Graphics backend type for camera calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraBackend {
    Vulkan,
    DirectX,
}

thread_local! {
    static CAMERA_BACKEND: std::cell::Cell<CameraBackend> = std::cell::Cell::new(CameraBackend::Vulkan);
}

/// Set the active camera backend (must be called before creating cameras)
pub fn set_camera_backend(backend: CameraBackend) {
    CAMERA_BACKEND.with(|b| b.set(backend));
}

/// Get the active camera backend
pub fn get_camera_backend() -> CameraBackend {
    CAMERA_BACKEND.with(|b| b.get())
}

/// Calculate perspective projection matrix (backend-aware)
pub fn perspective_projection(fov_degrees: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4 {
    match get_camera_backend() {
        CameraBackend::Vulkan => {
            // Vulkan: right-handed with reverse Z (0=far, 1=near) for better precision
            Mat4::perspective_rh(fov_degrees.to_radians(), aspect_ratio, near, far)
        }
        CameraBackend::DirectX => {
            // DirectX: left-handed, Y-up, Z depth range [0, 1]  
            Mat4::perspective_lh(fov_degrees.to_radians(), aspect_ratio, near, far)
        }
    }
}

/// Calculate view matrix from position and target
pub fn look_at_view(position: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    match get_camera_backend() {
        CameraBackend::Vulkan => Mat4::look_at_rh(position, target, up),
        CameraBackend::DirectX => Mat4::look_at_lh(position, target, up),
    }
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

    match get_camera_backend() {
        CameraBackend::Vulkan => Mat4::look_at_rh(position, target, up),
        CameraBackend::DirectX => Mat4::look_at_lh(position, target, up),
    }
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
        let view = look_at_view(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);
        assert!(!view.is_nan());
    }

    #[test]
    fn test_free_fly_view() {
        let view = free_fly_view(Vec3::new(0.0, 0.0, 5.0), -90.0, 0.0);
        assert!(!view.is_nan());
    }
}
