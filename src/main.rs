use bevy::{
    asset::AssetPlugin,
    log::LogPlugin,
    prelude::*,
    tasks::{IoTaskPool, TaskPool},
    time::TimePlugin,
};
use bevy_obj::ObjPlugin;
use plugin::RendererPlugin;

pub mod data;
pub mod plugin;
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
        mesh: meshes.add(Mesh::from(shape::Capsule { radius: 1.0, ..Default::default() })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, 3.0, 0.0),
        ..default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 1.0, ..Default::default() })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(-3.0, 1.0, 0.0),
        ..default()
    });

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Torus { radius: 1.0, ..Default::default() })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(3.0, 1.0, 0.0),
        ..default()
    });

    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 0.0, -3.0))
        .insert(mesh2);

    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(5.0, 5.0, 0.0),
        ..Default::default()
    });
}

fn orbit_camera(mut query: Query<&mut Transform, With<Camera3d>>, time: Res<Time>) {
    const CAMERA_DISTANCE: f32 = 10.0;
    const ROTATION_SPEED: f64 = 0.4;

    let t = time.time_since_startup().as_secs_f64();
    for mut transform in query.iter_mut() {
        // TODO transform is currently broken
        *transform = Transform::from_matrix(Mat4::look_at_rh(
            Vec3::new(
                (t * ROTATION_SPEED).cos() as f32 * CAMERA_DISTANCE,
                CAMERA_DISTANCE,
                (t * ROTATION_SPEED).sin() as f32 * CAMERA_DISTANCE,
            ),
            Vec3::ZERO,
            Vec3::Y,
        ));
    }
}

fn main() {
    IoTaskPool::init(|| TaskPool::new());

    App::new()
        .add_plugin(LogPlugin)
        .add_plugin(TimePlugin)
        .add_plugin(AssetPlugin)
        .add_plugin(ObjPlugin)
        .add_plugin(RendererPlugin)
        .add_startup_system(setup)
        .add_system(orbit_camera)
        .run();
}