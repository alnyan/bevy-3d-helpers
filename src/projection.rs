use bevy::math::{Mat4, Vec2};

pub trait Projection {
    fn compute_matrix(&self, dimensions: Vec2) -> Mat4;
}

pub struct PerspectiveProjection {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

pub struct OrthographicProjection {
    pub near: f32,
    pub far: f32,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            fov: 60.0f32.to_radians(),
            near: 0.01,
            far: 100.0,
        }
    }
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        Self {
            near: -50.0,
            far: 50.0,
            left: -50.0,
            right: 50.0,
            top: 50.0,
            bottom: -50.0,
        }
    }
}

impl Projection for PerspectiveProjection {
    fn compute_matrix(&self, dimensions: Vec2) -> Mat4 {
        Mat4::perspective_rh(self.fov, dimensions.x / dimensions.y, self.near, self.far)
    }
}

impl Projection for OrthographicProjection {
    fn compute_matrix(&self, dimensions: Vec2) -> Mat4 {
        Mat4::orthographic_rh(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        )
    }
}
