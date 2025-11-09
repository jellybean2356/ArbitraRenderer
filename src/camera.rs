use cgmath;
use bytemuck;

use crate::input::Input;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub yaw: f32,
    pub pitch: f32,
}

pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into()
    }
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            sensitivity: 0.002,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, input: &mut Input) {
        use cgmath::InnerSpace;

        let (raw_mouse_x, raw_mouse_y) = input.take_mouse_delta();
        let mouse_yaw = raw_mouse_x * self.sensitivity;
        let mouse_pitch = raw_mouse_y * self.sensitivity;
        
        if mouse_yaw != 0.0 || mouse_pitch != 0.0 {
            camera.yaw += mouse_yaw;
            camera.pitch -= mouse_pitch;
            camera.pitch = camera.pitch.clamp(-1.5, 1.5);
        }

        let (yaw_sin, yaw_cos) = camera.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = camera.pitch.sin_cos();
        
        let forward = cgmath::Vector3::new(
            yaw_cos * pitch_cos,
            pitch_sin,
            yaw_sin * pitch_cos,
        ).normalize();
        
        let right = forward.cross(camera.up).normalize();
        let up = camera.up.normalize();

        camera.target = camera.eye + forward;

        if input.is_forward_pressed {
            camera.eye += forward * self.speed;
            camera.target += forward * self.speed;
        }
        if input.is_backward_pressed {
            camera.eye -= forward * self.speed;
            camera.target -= forward * self.speed;
        }
        if input.is_right_pressed {
            camera.eye += right * self.speed;
            camera.target += right * self.speed;
        }
        if input.is_left_pressed {
            camera.eye -= right * self.speed;
            camera.target -= right * self.speed;
        }
        if input.is_space_pressed {
            camera.eye += up * self.speed;
            camera.target += up * self.speed;
        }
        if input.is_shift_pressed {
            camera.eye -= up * self.speed;
            camera.target -= up * self.speed;
        }
    }
}
