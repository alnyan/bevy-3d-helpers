use std::sync::Arc;

use bevy::{
    pbr::StandardMaterial,
    prelude::{
        info, AddAsset, App, Assets, Changed, Commands, Entity, Events, Handle, Mesh, Or, Plugin,
        Query, Res, Without,
    },
    window::{WindowCreated, WindowId, WindowResized, Windows},
};
use vulkano::device::Queue;
use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    data::Vertex,
    renderer::{mesh::DisplayMesh, VulkanContext},
};

pub struct RendererPlugin;

fn update_meshes(
    mut commands: Commands,
    query: Query<(Entity, &Handle<Mesh>), Or<(Changed<Handle<Mesh>>, Without<DisplayMesh>)>>,
    meshes: Res<Assets<Mesh>>,
    queue: Res<Arc<Queue>>,
) {
    for (entity, mesh) in query.iter() {
        if let Some(mesh) = meshes.get(mesh) {
            info!("Uploading a mesh for {:?}", entity);

            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            let normals = mesh
                .attribute(Mesh::ATTRIBUTE_NORMAL)
                .unwrap()
                .as_float3()
                .unwrap();
            assert_eq!(positions.len(), normals.len());

            let vertices = positions.into_iter().zip(normals).map(|(&p, &n)| Vertex {
                position: p,
                normal: n,
            });
            let indices: Vec<u32> = mesh.indices().unwrap().iter().map(|p| p as u32).collect();

            let dmesh = DisplayMesh::new(vertices, indices, queue.clone());

            commands.entity(entity).insert(dmesh);
        }
    }
}

fn renderer_runner(mut app: App) {
    info!("Running");
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

    event_loop.run(move |event, _, flow| match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
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
            .add_asset::<StandardMaterial>()
            .set_runner(renderer_runner)
            .add_system(update_meshes);
    }

    fn name(&self) -> &str {
        "alnyan-renderer"
    }
}
