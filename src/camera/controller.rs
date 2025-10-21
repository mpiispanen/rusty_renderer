//! Camera controller for interactive navigation

use crate::scene::Camera as SceneCamera;
use glam::{Mat4, Vec3};

/// Camera uniforms for GPU (view-projection matrix)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CameraUniforms {
    /// View-projection matrix (column-major for GPU)
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniforms {
    /// Create from view and projection matrices
    pub fn new(view: Mat4, projection: Mat4) -> Self {
        let view_proj = projection * view;
        Self {
            view_proj: view_proj.to_cols_array_2d(),
        }
    }

    /// Get as byte slice for buffer upload
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

// Safety: CameraUniforms is repr(C) with only f32 fields
unsafe impl bytemuck::Pod for CameraUniforms {}
unsafe impl bytemuck::Zeroable for CameraUniforms {}

/// Camera controller
pub struct CameraController {
    /// Current position
    position: Vec3,
    /// Camera type and parameters
    camera_type: CameraType,
    /// Viewport aspect ratio
    aspect_ratio: f32,
}

#[derive(Debug, Clone)]
enum CameraType {
    /// Fixed perspective camera (looks at target)
    Perspective {
        target: Vec3,
        up: Vec3,
        fov: f32,
        near: f32,
        far: f32,
    },
    /// Free-fly camera (yaw/pitch controlled)
    FreeFly {
        yaw: f32,
        pitch: f32,
        fov: f32,
        near: f32,
        far: f32,
    },
}

impl CameraController {
    /// Create from scene camera definition
    pub fn from_scene_camera(camera: &SceneCamera, width: u32, height: u32) -> Self {
        let aspect_ratio = width as f32 / height as f32;

        match camera {
            SceneCamera::Perspective {
                position,
                target,
                up,
                fov,
                near,
                far,
            } => Self {
                position: Vec3::from_array(*position),
                camera_type: CameraType::Perspective {
                    target: Vec3::from_array(*target),
                    up: Vec3::from_array(*up),
                    fov: *fov,
                    near: *near,
                    far: *far,
                },
                aspect_ratio,
            },
            SceneCamera::FreeFly {
                position,
                yaw,
                pitch,
                fov,
            } => Self {
                position: Vec3::from_array(*position),
                camera_type: CameraType::FreeFly {
                    yaw: *yaw,
                    pitch: *pitch,
                    fov: *fov,
                    near: 0.1,
                    far: 1000.0,
                },
                aspect_ratio,
            },
        }
    }

    /// Get view matrix
    pub fn view_matrix(&self) -> Mat4 {
        match &self.camera_type {
            CameraType::Perspective { target, up, .. } => {
                crate::camera::look_at_view(self.position, *target, *up)
            }
            CameraType::FreeFly { yaw, pitch, .. } => {
                crate::camera::free_fly_view(self.position, *yaw, *pitch)
            }
        }
    }

    /// Get projection matrix
    pub fn projection_matrix(&self) -> Mat4 {
        match &self.camera_type {
            CameraType::Perspective { fov, near, far, .. } => {
                crate::camera::perspective_projection(*fov, self.aspect_ratio, *near, *far)
            }
            CameraType::FreeFly { fov, near, far, .. } => {
                crate::camera::perspective_projection(*fov, self.aspect_ratio, *near, *far)
            }
        }
    }

    /// Get camera uniforms for GPU
    pub fn uniforms(&self) -> CameraUniforms {
        CameraUniforms::new(self.view_matrix(), self.projection_matrix())
    }

    /// Update aspect ratio (for window resize)
    pub fn set_aspect_ratio(&mut self, width: u32, height: u32) {
        self.aspect_ratio = width as f32 / height as f32;
    }

    /// Move camera forward/backward (free-fly only)
    pub fn move_forward(&mut self, distance: f32) {
        if let CameraType::FreeFly { yaw, pitch, .. } = &self.camera_type {
            let yaw_rad = yaw.to_radians();
            let pitch_rad = pitch.to_radians();

            let forward = Vec3::new(
                yaw_rad.cos() * pitch_rad.cos(),
                pitch_rad.sin(),
                yaw_rad.sin() * pitch_rad.cos(),
            )
            .normalize();

            self.position += forward * distance;
        }
    }

    /// Move camera right/left (free-fly only)
    pub fn move_right(&mut self, distance: f32) {
        if let CameraType::FreeFly { yaw, .. } = &self.camera_type {
            let yaw_rad = yaw.to_radians();

            let right = Vec3::new(yaw_rad.sin(), 0.0, -yaw_rad.cos()).normalize();

            self.position += right * distance;
        }
    }

    /// Move camera up/down (free-fly only)
    pub fn move_up(&mut self, distance: f32) {
        self.position.y += distance;
    }

    /// Rotate camera (free-fly only)
    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        if let CameraType::FreeFly { yaw, pitch, .. } = &mut self.camera_type {
            *yaw += delta_yaw;
            *pitch += delta_pitch;

            // Clamp pitch to prevent gimbal lock
            *pitch = pitch.clamp(-89.0, 89.0);
        }
    }

    /// Get current position
    pub fn position(&self) -> Vec3 {
        self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Camera;

    #[test]
    fn test_perspective_camera_controller() {
        let scene_camera = Camera::Perspective {
            position: [0.0, 0.0, 5.0],
            target: [0.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            fov: 45.0,
            near: 0.1,
            far: 100.0,
        };

        let controller = CameraController::from_scene_camera(&scene_camera, 800, 600);
        let uniforms = controller.uniforms();

        // Check that matrices aren't NaN
        assert!(!uniforms.view_proj[0][0].is_nan());
    }

    #[test]
    fn test_free_fly_camera_controller() {
        let scene_camera = Camera::FreeFly {
            position: [0.0, 0.0, 5.0],
            yaw: -90.0,
            pitch: 0.0,
            fov: 45.0,
        };

        let mut controller = CameraController::from_scene_camera(&scene_camera, 800, 600);

        // Test movement
        let initial_pos = controller.position();
        controller.move_forward(1.0);
        assert_ne!(initial_pos, controller.position());

        // Test rotation
        controller.rotate(10.0, 5.0);
        let uniforms = controller.uniforms();
        assert!(!uniforms.view_proj[0][0].is_nan());
    }

    #[test]
    fn test_camera_uniforms() {
        let view = Mat4::IDENTITY;
        let proj = Mat4::IDENTITY;
        let _uniforms = CameraUniforms::new(view, proj);

        // Check size matches expected GPU layout
        assert_eq!(std::mem::size_of::<CameraUniforms>(), 64); // 16 floats * 4 bytes
    }
}
