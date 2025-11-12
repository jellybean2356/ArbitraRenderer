use cgmath::{Matrix4, Vector3, Deg};

/// Represents a 3D transformation (position, rotation, scale)
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],  // Euler angles in degrees (x, y, z)
    pub scale: [f32; 3],
}

impl Transform {
    pub fn new() -> Self {
        Transform {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }

    #[allow(dead_code)]
    pub fn with_position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = [x, y, z];
        self
    }

    #[allow(dead_code)]
    pub fn with_rotation(mut self, x: f32, y: f32, z: f32) -> Self {
        self.rotation = [x, y, z];
        self
    }

    #[allow(dead_code)]
    pub fn with_scale(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = [x, y, z];
        self
    }

    /// Convert transform to a 4x4 model matrix
    pub fn to_matrix(&self) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(Vector3::new(
            self.position[0],
            self.position[1],
            self.position[2],
        ));

        // Apply rotations in order: X -> Y -> Z (Euler angles)
        let rotation_x = Matrix4::from_angle_x(Deg(self.rotation[0]));
        let rotation_y = Matrix4::from_angle_y(Deg(self.rotation[1]));
        let rotation_z = Matrix4::from_angle_z(Deg(self.rotation[2]));
        let rotation = rotation_z * rotation_y * rotation_x;

        let scale = Matrix4::from_nonuniform_scale(
            self.scale[0],
            self.scale[1],
            self.scale[2],
        );

        // Combine: translate * rotate * scale
        translation * rotation * scale
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}
