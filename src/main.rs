use bevy::{
    log::LogPlugin,
    prelude::*,
    scene::ScenePlugin,
    tasks::{IoTaskPool, TaskPool},
    time::TimePlugin,
};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    prelude::{Collider, Friction, RigidBody, Velocity},
};
use plugins::{
    camera::{CameraProjection, FlyCamera, FlyCameraPlugin},
    renderer::WindowSetting,
    DefaultRendererPlugins,
};
use renderer::material::DisplayMaterial;

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
    mut window_setting_events: ResMut<Events<WindowSetting>>,
) {
    let texture0 = asset_server.load("texture0.png");
    let texture1 = asset_server.load("texture1.png");

    commands
        .spawn()
        .insert(meshes.add(Mesh::from(shape::Plane { size: 50.0 })))
        .insert(Transform::from_xyz(0.0, -1.0, 0.0))
        .insert(GlobalTransform::identity())
        .insert(DisplayMaterial {
            k_diffuse: Color::WHITE,
            k_diffuse_map: Some(texture0),
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
            .spawn()
            .insert(meshes.add(Mesh::from(shape::Cube { size: r })))
            .insert(Transform::from_xyz(dx, y, dz).with_rotation(
                Quat::from_axis_angle(Vec3::Y, ay) * Quat::from_axis_angle(Vec3::X, ax),
            ))
            .insert(GlobalTransform::identity())
            .insert(DisplayMaterial {
                k_diffuse: Color::rgb(k_r, k_g, k_b),
                k_diffuse_map: Some(texture1.clone()),
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
        .add_plugin(ScenePlugin)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(ObjPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_startup_system(setup)
        .run();
}
