use bevy::{
    log::LogPlugin,
    prelude::*,
    tasks::{IoTaskPool, TaskPool},
    time::TimePlugin,
};
use bevy_obj::ObjPlugin;
use plugins::{
    camera::{CameraProjection, FlyCamera, FlyCameraPlugin},
    renderer::WindowSetting,
    DefaultRendererPlugins,
};

pub mod conversion;
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
    mut window_setting_events: ResMut<Events<WindowSetting>>,
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
        .insert(Transform::from_xyz(0.0, 0.5, 0.0))
        .insert(FlyCamera::default())
        .insert(CameraProjection::Perspective(default()));

    window_setting_events.send(WindowSetting::SetMouseGrab(true));
}

fn main() {
    IoTaskPool::init(TaskPool::new);

    App::new()
        .add_plugin(LogPlugin)
        .add_plugin(TimePlugin)
        .add_plugins(DefaultRendererPlugins)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(ObjPlugin)
        .add_startup_system(setup)
        .run();
}
