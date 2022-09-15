use bevy::{
    log::LogPlugin,
    prelude::*,
    scene::ScenePlugin,
    tasks::{IoTaskPool, TaskPool},
    time::TimePlugin, render::texture::ImagePlugin,
};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    prelude::{Collider, RigidBody, Velocity, Friction, RapierDebugRenderPlugin},
};
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

    // commands.spawn_bundle(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Capsule {
    //         radius: 1.0,
    //         ..Default::default()
    //     })),
    //     material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
    //     transform: Transform::from_xyz(0.0, 3.0, 0.0),
    //     ..default()
    // });

    // commands.spawn_bundle(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::UVSphere {
    //         radius: 1.0,
    //         ..Default::default()
    //     })),
    //     material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
    //     transform: Transform::from_xyz(-3.0, 1.0, 0.0),
    //     ..default()
    // });

    // commands.spawn_bundle(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Torus {
    //         radius: 1.0,
    //         ..Default::default()
    //     })),
    //     material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
    //     transform: Transform::from_xyz(3.0, 1.0, 0.0),
    //     ..default()
    // });

    let texture0 = asset_server.load("texture0.png");

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0 })),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(texture0),
                ..Default::default()
            }),
            transform: Transform::from_xyz(0.0, -1.0, 0.0),
            ..default()
        })
        .insert(RigidBody::Fixed)
        .insert(Collider::cuboid(50.0, 0.001, 50.0));

    for i in 0..10 {
        let r = rand::random::<f32>() * 2.0;
        let ay = rand::random::<f32>() * 2.0 - 1.0;
        let ax = rand::random::<f32>() * 2.0 - 1.0;
        let dx = rand::random::<f32>() * 1.0 - 0.5;
        let dz = rand::random::<f32>() * 1.0 - 0.5;
        let y = i as f32 * 5.0 + 10.0;

        let k_r = rand::random::<f32>();
        let k_g = rand::random::<f32>();
        let k_b = rand::random::<f32>();

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: r })),
                material: materials.add(Color::rgb(k_r, k_g, k_b).into()),
                transform: Transform::from_xyz(dx, y, dz).with_rotation(
                    Quat::from_axis_angle(Vec3::Y, ay) * Quat::from_axis_angle(Vec3::X, ax),
                ),
                ..Default::default()
            })
            .insert(RigidBody::Dynamic)
            .insert(Velocity::zero())
            .insert(Friction::coefficient(1.2))
            .insert(Collider::cuboid(r / 2.0, r / 2.0, r / 2.0));
    }

    commands
        .spawn()
        .insert(Transform::from_xyz(0.0, 5.0, 5.0))
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
        .add_plugin(ImagePlugin)
        .add_plugin(ScenePlugin)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(ObjPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_startup_system(setup)
        .run();
}
