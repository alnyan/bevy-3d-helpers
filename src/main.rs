use bevy::{
    audio::AudioPlugin,
    log::LogPlugin,
    prelude::*,
    tasks::{IoTaskPool, TaskPool},
    time::TimePlugin,
};
use bevy_obj::ObjPlugin;
use plugins::{camera::CameraProjection, DefaultRendererPlugins};

pub mod data;
pub mod plugins;
pub mod projection;
pub mod renderer;
pub mod shaders;

fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh2 = asset_server.load::<Mesh, _>("model2.obj");

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 25.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, -1.0, 0.0),
        ..default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Capsule {
            radius: 1.0,
            ..Default::default()
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, 3.0, 0.0),
        ..default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere {
            radius: 1.0,
            ..Default::default()
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(-3.0, 1.0, 0.0),
        ..default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Torus {
            radius: 1.0,
            ..Default::default()
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(3.0, 1.0, 0.0),
        ..default()
    });

    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 0.0, -3.0))
        .insert(mesh2);

    commands
        .spawn()
        .insert(Transform::from_xyz(5.0, 5.0, 0.0))
        .insert(CameraProjection::Perspective(default()));
}

fn orbit_camera(mut query: Query<(&mut Transform, &mut CameraProjection)>, time: Res<Time>) {
    const CAMERA_DISTANCE: f32 = 10.0;
    const ROTATION_SPEED: f64 = 0.4;

    let t = time.time_since_startup().as_secs_f64();
    for (mut transform, mut settings) in query.iter_mut() {
        *transform = Transform::from_xyz(
            (t * ROTATION_SPEED).cos() as f32 * CAMERA_DISTANCE,
            CAMERA_DISTANCE,
            (t * ROTATION_SPEED).sin() as f32 * CAMERA_DISTANCE,
        )
        .looking_at(Vec3::ZERO, Vec3::Y);

        if (t / 5.0) as u64 % 2 == 0 {
            *settings = CameraProjection::Perspective(projection::PerspectiveProjection {
                fov: (60.0 + 20.0 * t.cos() as f32).to_radians(),
                near: 0.01,
                far: 100.0,
            });
        } else {
            let dim = 20.0 + 10.0 * (t + 237.0).cos() as f32;
            *settings = CameraProjection::Orthographic(projection::OrthographicProjection {
                left: -dim,
                right: dim,
                bottom: -dim,
                top: dim,
                near: -dim,
                far: dim + 20.0,
            });
        }
    }
}

fn main() {
    IoTaskPool::init(|| TaskPool::new());

    App::new()
        .add_plugin(LogPlugin)
        .add_plugin(TimePlugin)
        .add_plugins(DefaultRendererPlugins)
        .add_plugin(ObjPlugin)
        .add_startup_system(setup)
        .add_system(orbit_camera)
        .run();
}
