use std::sync::Arc;

use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseMotion},
    math::Vec2,
    prelude::{
        debug, AddAsset, App, Assets, Changed, Commands, CoreStage, Entity, EventReader, Events,
        Handle, Mesh, Or, Plugin, Query, Res, SystemSet, Without,
    },
    window::{WindowCreated, WindowId, WindowResized}, render::mesh::VertexAttributeValues,
};
use vulkano::device::Queue;
use winit::{
    event::{DeviceEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    conversion::{convert_element_state, convert_virtual_keycode},
    data::Vertex,
    renderer::{
        material::{DisplayMaterial, TextureImage},
        mesh::DisplayMesh,
        VulkanContext,
    },
};

pub struct RendererPlugin;

pub enum WindowSetting {
    SetFullscreen(bool),
    SetMouseGrab(bool),
}

#[allow(clippy::type_complexity)]
fn update_meshes(
    mut commands: Commands,
    query: Query<(Entity, &Handle<Mesh>), Or<(Changed<Handle<Mesh>>, Without<DisplayMesh>)>>,
    meshes: Res<Assets<Mesh>>,
    queue: Res<Arc<Queue>>,
) {
    for (entity, mesh) in query.iter() {
        if let Some(mesh) = meshes.get(mesh) {
            debug!("Uploading a mesh for {:?}", entity);

            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            let tex_coords = mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap();
            let tex_coords = match tex_coords {
                VertexAttributeValues::Float32x2(v) => v,
                _ => panic!("Texture coordinates are not in float x2 format")
            };
            let normals = mesh
                .attribute(Mesh::ATTRIBUTE_NORMAL)
                .unwrap()
                .as_float3()
                .unwrap();
            assert_eq!(positions.len(), normals.len());

            let vertices =
                itertools::izip!(positions, tex_coords, normals).map(|(&p, &t, &n)| Vertex {
                    position: p,
                    tex_coords: t,
                    normal: n,
                });
            let indices: Vec<u32> = mesh.indices().unwrap().iter().map(|p| p as u32).collect();

            let dmesh = DisplayMesh::new(vertices, indices, queue.clone());

            commands.entity(entity).insert(dmesh);
        }
    }
}

fn update_window(window: Res<Arc<Window>>, mut window_setting_events: EventReader<WindowSetting>) {
    for event in window_setting_events.iter() {
        match event {
            WindowSetting::SetMouseGrab(true) => {
                window.set_cursor_visible(false);
                window.set_cursor_grab(true).unwrap();
            }
            WindowSetting::SetMouseGrab(false) => {
                window.set_cursor_visible(true);
                window.set_cursor_grab(false).unwrap();
            }
            _ => (),
        }
    }
}

fn renderer_runner(mut app: App) {
    debug!("Running");
    let event_loop = EventLoop::new();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Test")
            .with_resizable(false)
            .build(&event_loop)
            .unwrap(),
    );

    let mut renderer = VulkanContext::new_windowed(window.clone());

    // TODO somehow interate with "Windows" resource
    app.insert_resource(window.clone())
        .insert_resource(renderer.gfx_queue().clone());

    app.world.send_event(WindowCreated {
        id: WindowId::default(),
    });

    app.update();

    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);

    event_loop.run(move |event, _, flow| match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => {
                let mut keyboard_input_events = app.world.resource_mut::<Events<KeyboardInput>>();

                keyboard_input_events.send(KeyboardInput {
                    scan_code: input.scancode,
                    key_code: input.virtual_keycode.map(convert_virtual_keycode),
                    state: convert_element_state(input.state),
                });
            }
            WindowEvent::Resized(size) => {
                let mut window_resized_events = app.world.resource_mut::<Events<WindowResized>>();
                renderer.invalidate_surface();
                window_resized_events.send(WindowResized {
                    width: size.width as f32,
                    height: size.height as f32,
                    id: WindowId::default(),
                });
            }
            _ => *flow = ControlFlow::Poll,
        },
        winit::event::Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta: (x, y) },
            ..
        } => {
            let mut mouse_motion_events = app.world.resource_mut::<Events<MouseMotion>>();
            let delta = Vec2::new(x as f32, y as f32);
            mouse_motion_events.send(MouseMotion { delta });
        }
        winit::event::Event::MainEventsCleared => {
            app.update();
        }
        winit::event::Event::RedrawRequested(_) => {
            renderer.do_frame(flow, &mut app.world);
        }
        winit::event::Event::RedrawEventsCleared => {
            window.request_redraw();
        }
        _ => *flow = ControlFlow::Poll,
    });
}

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Mesh>()
            .add_asset::<TextureImage>()
            .add_asset::<DisplayMaterial>()
            .add_event::<WindowSetting>()
            .set_runner(renderer_runner)
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new().with_system(update_meshes),
            )
            .add_system_to_stage(CoreStage::PostUpdate, update_window);
    }

    fn name(&self) -> &str {
        "alnyan-renderer"
    }
}
