use std::sync::Arc;

use bevy::{
    math::{Mat4, Vec2},
    prelude::{
        App, Changed, Commands, Component, CoreStage, Entity, EventReader, Plugin, Query, Res,
        SystemSet,
    },
    window::{WindowCreated, WindowResized},
};
use winit::window::Window;

use crate::projection::{OrthographicProjection, PerspectiveProjection, Projection};

pub struct CameraPlugin;

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
