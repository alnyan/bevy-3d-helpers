use std::sync::Arc;

use bevy::{
    math::Mat4,
    pbr::StandardMaterial,
    prelude::{Assets, Handle, Image, Transform, World},
};
use vulkano::{
    buffer::{BufferUsage, CpuBufferPool, TypedBufferAccess},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo},
    format::Format,
    image::{
        view::ImageView, AttachmentImage, ImageDimensions, ImmutableImage, MipmapsCount,
        SwapchainImage,
    },
    instance::{
        debug::{
            DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger,
            DebugUtilsMessengerCreateInfo,
        },
        Instance, InstanceCreateInfo, InstanceExtensions,
    },
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline, Pipeline, PipelineBindPoint},
    render_pass::{Framebuffer, RenderPass},
    sampler::{Filter, Sampler, SamplerCreateInfo},
    shader::ShaderModule,
    swapchain::{self, Surface, Swapchain, SwapchainCreateInfo},
    sync::GpuFuture,
};
use winit::{event_loop::ControlFlow, window::Window};

use crate::{plugins::camera::ComputedProjection, shaders};

use self::{mesh::DisplayMesh, material::DisplayMaterial};

pub mod material;
pub mod mesh;
pub mod util;

pub type WindowHandle = Arc<Window>;

pub struct VulkanContext {
    surface: Arc<Surface<WindowHandle>>,

    device: Arc<Device>,
    queue: Arc<Queue>,

    swapchain: Arc<Swapchain<WindowHandle>>,
    swapchain_images: Vec<Arc<ImageView<SwapchainImage<WindowHandle>>>>,
    viewport: Viewport,
    need_swapchain_recreation: bool,
    dimensions: [u32; 2],

    render_pass: Arc<RenderPass>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    pipeline: Arc<GraphicsPipeline>,
    framebuffers: Vec<Arc<Framebuffer>>,
    vp_pool: CpuBufferPool<shaders::vs::ty::ViewProjection_Data>,
    material_pool: CpuBufferPool<shaders::fs::ty::Material_Data>,
    model_pool: CpuBufferPool<shaders::vs::ty::Model_Data>,
    color_view: Arc<ImageView<AttachmentImage>>,
    depth_view: Arc<ImageView<AttachmentImage>>,

    dummy_texture: Arc<ImageView<ImmutableImage>>,
    sampler: Arc<Sampler>,
}

impl VulkanContext {
    pub fn new_windowed(window: WindowHandle) -> Self {
        let instance_extensions = vulkano_win::required_extensions().union(&InstanceExtensions {
            ext_debug_utils: true,
            ..InstanceExtensions::none()
        });
        let instance_layers = vec![
            //"VK_LAYER_KHRONOS_validation".to_owned()
        ];
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            khr_maintenance1: true,
            ..DeviceExtensions::none()
        };

        let instance = Instance::new(InstanceCreateInfo {
            enabled_extensions: instance_extensions,
            enabled_layers: instance_layers,
            ..Default::default()
        })
        .unwrap();

        unsafe {
            let _cb = DebugUtilsMessenger::new(
                instance.clone(),
                DebugUtilsMessengerCreateInfo {
                    message_type: DebugUtilsMessageType::all(),
                    message_severity: DebugUtilsMessageSeverity::all(),
                    ..DebugUtilsMessengerCreateInfo::user_callback(Arc::new(|msg| {
                        bevy::prelude::info!("{:?}", msg.description);
                    }))
                },
            )
            .ok();
        }

        let surface = vulkano_win::create_surface_from_winit(window, instance.clone()).unwrap();
        let dimensions = surface.window().inner_size().into();

        let format = Format::B8G8R8A8_SRGB;

        let (physical, queue_family) = util::select_physical_device(&instance, surface.clone());

        let (device, mut queues) = Device::new(
            physical,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
                enabled_extensions: physical
                    .supported_extensions()
                    .intersection(&device_extensions),
                ..Default::default()
            },
        )
        .unwrap();
        let queue = queues.next().unwrap();

        let (swapchain, swapchain_images) =
            util::create_swapchain(device.clone(), surface.clone(), format);

        let viewport = util::create_viewport(dimensions);

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                final_color: {
                    load: Clear,
                    store: Store,
                    format: format,
                    samples: 1,
                },
                color: {
                    load: Clear,
                    store: DontCare,
                    format: format,
                    samples: 4,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D32_SFLOAT,
                    samples: 4,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth},
                resolve: [final_color]
            }
        )
        .unwrap();

        let (dummy_texture, init) = {
            let image_data = [255u8, 255u8, 255u8, 255u8];

            let (image, init) = ImmutableImage::from_iter(
                image_data,
                ImageDimensions::Dim2d {
                    width: 1,
                    height: 1,
                    array_layers: 1,
                },
                MipmapsCount::One,
                Format::R8G8B8A8_UNORM,
                queue.clone(),
            )
            .unwrap();

            (ImageView::new_default(image).unwrap(), init)
        };

        let sampler = Sampler::new(
            device.clone(),
            SamplerCreateInfo {
                mag_filter: Filter::Nearest,
                min_filter: Filter::Nearest,
                ..Default::default()
            },
        )
        .unwrap();

        init.then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        let vs = shaders::vs::load(device.clone()).unwrap();
        let fs = shaders::fs::load(device.clone()).unwrap();
        let pipeline = util::create_pipeline(
            render_pass.clone(),
            vs.clone(),
            fs.clone(),
            viewport.clone(),
            device.clone(),
        );
        let (framebuffers, color_view, depth_view) =
            util::create_framebuffers(render_pass.clone(), device.clone(), &swapchain_images);

        let vp_pool = CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer());
        let material_pool = CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer());
        let model_pool = CpuBufferPool::new(device.clone(), BufferUsage::uniform_buffer());

        Self {
            surface,
            device,
            queue,
            swapchain,
            swapchain_images,
            viewport,
            dimensions,
            need_swapchain_recreation: false,

            render_pass,
            pipeline,
            vs,
            fs,
            framebuffers,
            vp_pool,
            material_pool,
            model_pool,
            depth_view,
            color_view,

            dummy_texture,
            sampler,
        }
    }

    pub const fn gfx_queue(&self) -> &Arc<Queue> {
        &self.queue
    }

    pub fn invalidate_surface(&mut self) {
        self.need_swapchain_recreation = true;
    }

    pub fn do_frame(&mut self, _flow: &mut ControlFlow, world: &mut World) {
        if self.need_swapchain_recreation {
            self.recreate_swapchain();
        }

        let (image_index, suboptimal, acquire_future) =
            swapchain::acquire_next_image(self.swapchain.clone(), None).unwrap();

        if suboptimal {
            self.need_swapchain_recreation = true;
        }

        let (camera_transform, camera_projection) = match world
            .query::<(&Transform, &ComputedProjection)>()
            .get_single(world)
        {
            Ok(e) => e,
            Err(_) => {
                acquire_future
                    .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_index)
                    .then_signal_fence_and_flush()
                    .unwrap()
                    .wait(None)
                    .unwrap();

                return;
            }
        };

        let camera_position = camera_transform.translation;
        let view = camera_transform.compute_matrix().inverse();
        let projection = camera_projection.transform_matrix();

        let vp_buffer = {
            let data = shaders::vs::ty::ViewProjection_Data {
                camera_position: camera_position.into(),
                view: view.to_cols_array_2d(),
                projection: projection.to_cols_array_2d(),
            };

            self.vp_pool.next(data).unwrap()
        };

        let vp_set_layout = self.pipeline.layout().set_layouts().get(0).unwrap();
        let material_set_layout = self.pipeline.layout().set_layouts().get(1).unwrap();
        let model_set_layout = self.pipeline.layout().set_layouts().get(2).unwrap();

        let vp_set = PersistentDescriptorSet::new(
            vp_set_layout.clone(),
            vec![WriteDescriptorSet::buffer(0, vp_buffer)],
        )
        .unwrap();

        let framebuffer = &self.framebuffers[image_index];
        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let render_pass_begin_info = RenderPassBeginInfo {
            clear_values: vec![
                Some([0.0, 0.0, 0.0, 1.0].into()),
                Some([0.0, 0.0, 0.0, 1.0].into()),
                Some(1.0.into()),
            ],
            ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
        };

        builder
            .begin_render_pass(render_pass_begin_info, SubpassContents::Inline)
            .unwrap()
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                vp_set,
            );

        let mut query =
            world.query::<(&Transform, &DisplayMesh, Option<&DisplayMaterial>)>();
        for (transform, mesh, material) in query.iter(world) {
            let model_matrix: Mat4 = transform.compute_matrix();

            let texture;
            let material_buffer = {
                let data;

                if let Some(material) = material {
                    data = shaders::fs::ty::Material_Data {
                        k_diffuse: material.k_diffuse.as_rgba_f32(),
                    };

                    if let Some(k_diffuse_map) = &material.k_diffuse_map {
                        texture = k_diffuse_map.clone();
                    } else {
                        texture = self.dummy_texture.clone();
                    }
                } else {
                    texture = self.dummy_texture.clone();
                    data = shaders::fs::ty::Material_Data {
                        k_diffuse: [1.0, 0.0, 0.0, 1.0],
                    };
                }

                self.material_pool.next(data).unwrap()
            };
            let model_buffer = {
                let data = shaders::vs::ty::Model_Data {
                    model: model_matrix.to_cols_array_2d(),
                };

                self.model_pool.next(data).unwrap()
            };

            let material_set = PersistentDescriptorSet::new(
                material_set_layout.clone(),
                vec![
                    WriteDescriptorSet::buffer(0, material_buffer),
                    WriteDescriptorSet::image_view_sampler(1, texture, self.sampler.clone()),
                ],
            )
            .unwrap();
            let model_set = PersistentDescriptorSet::new(
                model_set_layout.clone(),
                vec![WriteDescriptorSet::buffer(0, model_buffer)],
            )
            .unwrap();

            builder
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    self.pipeline.layout().clone(),
                    1,
                    (material_set, model_set),
                )
                .bind_vertex_buffers(0, mesh.vertices().clone())
                .bind_index_buffer(mesh.indices().clone())
                .draw_indexed(mesh.indices().len() as u32, 1, 0, 0, 0)
                .unwrap();
        }
        builder.end_render_pass().unwrap();

        let future = acquire_future
            .then_execute(self.queue.clone(), builder.build().unwrap())
            .unwrap()
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_index)
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();
    }

    fn recreate_swapchain(&mut self) {
        self.dimensions = self.surface.window().inner_size().into();
        let (new_swapchain, new_images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: self.dimensions,
                ..self.swapchain.create_info()
            })
            .unwrap();

        self.swapchain = new_swapchain;
        self.swapchain_images = new_images
            .into_iter()
            .map(ImageView::new_default)
            .collect::<Result<_, _>>()
            .unwrap();

        self.viewport = util::create_viewport(self.dimensions);

        self.pipeline = util::create_pipeline(
            self.render_pass.clone(),
            self.vs.clone(),
            self.fs.clone(),
            self.viewport.clone(),
            self.device.clone(),
        );
        (self.framebuffers, self.color_view, self.depth_view) = util::create_framebuffers(
            self.render_pass.clone(),
            self.device.clone(),
            &self.swapchain_images,
        );
    }
}
