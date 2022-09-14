use std::sync::Arc;

use bevy::{
    math::{Mat4, Vec2, Vec3, Quat},
    prelude::{
        App, Changed, Commands, Component, CoreStage, Entity, EventReader, Plugin, Query, Res,
        SystemSet, Transform, With, Time, KeyCode,
    },
    window::{WindowCreated, WindowResized}, input::{Input, mouse::MouseMotion},
};
use winit::window::Window;

use crate::projection::{OrthographicProjection, PerspectiveProjection, Projection};

pub struct CameraPlugin;
pub struct FlyCameraPlugin;

#[derive(Component, Default)]
pub struct FlyCamera {
    pitch: f32,
    yaw: f32,
}

#[derive(Component)]
pub struct ComputedProjection {
    dimensions: Vec2,
    projection: Mat4,
}
#[derive(Component)]
pub enum CameraProjection {
    Perspective(PerspectiveProjection),
    Orthographic(OrthographicProjection),
}

impl Projection for CameraProjection {
    fn compute_matrix(&self, dimensions: Vec2) -> Mat4 {
        match self {
            Self::Perspective(p) => p.compute_matrix(dimensions),
            Self::Orthographic(p) => p.compute_matrix(dimensions),
        }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(setup_camera_initial)
                .with_system(update_camera_dimensions),
        );
        app.add_system_to_stage(CoreStage::PostUpdate, update_camera_settings);
    }

    fn name(&self) -> &str {
        "alnyan-camera"
    }
}

impl Plugin for FlyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::new()
                .with_system(camera_movement)
                .with_system(camera_rotation),
        );
    }
}

impl ComputedProjection {
    pub const fn transform_matrix(&self) -> &Mat4 {
        &self.projection
    }
}

fn setup_camera_initial(
    mut commands: Commands,
    mut window_create_events: EventReader<WindowCreated>,
    mut query: Query<(Entity, &CameraProjection)>,
    window: Res<Arc<Window>>,
) {
    let create = window_create_events.iter().last();

    if create.is_some() {
        let dim = window.inner_size();
        let dim = Vec2::new(dim.width as f32, dim.height as f32);

        for (entity, settings) in query.iter_mut() {
            let new = ComputedProjection {
                dimensions: dim,
                projection: settings.compute_matrix(dim),
            };

            commands.entity(entity).insert(new);
        }
    }
}

fn update_camera_dimensions(
    mut window_resize_events: EventReader<WindowResized>,
    mut query: Query<(&CameraProjection, &mut ComputedProjection)>,
) {
    if let Some(resize) = window_resize_events
        .iter()
        .last()
        .map(|e| Vec2::new(e.width, e.height))
    {
        for (settings, mut computed) in query.iter_mut() {
            computed.dimensions = resize;
            computed.projection = settings.compute_matrix(resize);
        }
    }
}

fn update_camera_settings(
    mut query: Query<(&CameraProjection, &mut ComputedProjection), Changed<CameraProjection>>,
) {
    for (settings, mut computed) in query.iter_mut() {
        computed.projection = settings.compute_matrix(computed.dimensions);
    }
}

#[inline]
fn movement(v: Vec3, a: bool, b: bool) -> Vec3 {
    if a {
        v
    } else if b {
        -v
    } else {
        Vec3::ZERO
    }
}

// TODO use action resource instead of keycode
fn camera_movement(
    mut query: Query<&mut Transform, With<FlyCamera>>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    for mut transform in query.iter_mut() {
        let up = movement(
            Vec3::Y,
            keyboard_input.pressed(KeyCode::Space),
            keyboard_input.pressed(KeyCode::LShift),
        );
        let forward = movement(
            transform.forward(),
            keyboard_input.pressed(KeyCode::W),
            keyboard_input.pressed(KeyCode::S),
        );
        let right = movement(
            transform.right(),
            keyboard_input.pressed(KeyCode::D),
            keyboard_input.pressed(KeyCode::A),
        );

        let v = up + forward + right;
        if v.length() != 0.0 {
            let wantvec = v.normalize();
            let movevec = wantvec * time.delta_seconds() * 4.0;

            transform.translation += movevec;
        }
    }
}

fn camera_rotation(
    mut query: Query<(&mut Transform, &mut FlyCamera)>,
    mut mouse_input: EventReader<MouseMotion>,
) {
    if let Some(motion) = mouse_input.iter().last() {
        for (mut transform, mut camera) in query.iter_mut() {
            camera.pitch -= motion.delta.y * 0.005;
            camera.yaw += motion.delta.x * 0.005;

            transform.rotation = Quat::from_axis_angle(Vec3::Y, -camera.yaw)
                * Quat::from_axis_angle(Vec3::X, camera.pitch);
        }
    }
}
