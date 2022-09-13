use std::sync::Arc;

use vulkano::{
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily},
        Device,
    },
    format::Format,
    image::{
        view::ImageView, AttachmentImage, ImageUsage, ImageViewAbstract, SampleCount,
        SwapchainImage,
    },
    instance::Instance,
    pipeline::{
        graphics::{
            depth_stencil::DepthStencilState,
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    shader::ShaderModule,
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
};
use vulkano_win::SafeBorrow;
use winit::window::Window;

use crate::data::Vertex;

use super::WindowHandle;

pub type SwapchainCreateOutput = (
    Arc<Swapchain<WindowHandle>>,
    Vec<Arc<ImageView<SwapchainImage<WindowHandle>>>>,
);

pub fn select_physical_device<'b, T: SafeBorrow<Window>>(
    instance: &'b Arc<Instance>,
    surface: Arc<Surface<T>>,
) -> (PhysicalDevice<'b>, QueueFamily<'b>) {
    PhysicalDevice::enumerate(instance)
        .filter_map(|p| {
            p.queue_families()
                .find(|&q| q.supports_graphics() && q.supports_surface(&surface).unwrap_or(false))
                .map(|q| (p, q))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .unwrap()
}

pub fn create_swapchain(
    device: Arc<Device>,
    surface: Arc<Surface<WindowHandle>>,
    format: Format,
) -> SwapchainCreateOutput {
    let caps = device
        .physical_device()
        .surface_capabilities(&surface, Default::default())
        .unwrap();

    let image_format = Some(format);

    let (swapchain, images) = Swapchain::new(
        device,
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count,
            image_extent: surface.window().inner_size().into(),
            image_usage: ImageUsage {
                color_attachment: true,
                transfer_dst: true,
                ..ImageUsage::none()
            },
            composite_alpha: caps.supported_composite_alpha.iter().next().unwrap(),
            image_format,
            ..Default::default()
        },
    )
    .unwrap();

    let swapchain_images = images
        .into_iter()
        .map(|image| ImageView::new_default(image))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    (swapchain, swapchain_images)
}

pub fn create_viewport(dimensions: [u32; 2]) -> Viewport {
    Viewport {
        origin: [0.0, dimensions[1] as f32],
        dimensions: [dimensions[0] as f32, -(dimensions[1] as f32)],
        depth_range: 0.0..1.0,
    }
}

pub fn create_framebuffers(
    render_pass: Arc<RenderPass>,
    device: Arc<Device>,
    swapchain_images: &Vec<Arc<ImageView<SwapchainImage<WindowHandle>>>>,
) -> (
    Vec<Arc<Framebuffer>>,
    Arc<ImageView<AttachmentImage>>,
    Arc<ImageView<AttachmentImage>>,
) {
    let dimensions = swapchain_images[0].dimensions().width_height();
    let depth_view = ImageView::new_default(
        AttachmentImage::transient_multisampled(
            device.clone(),
            dimensions,
            SampleCount::Sample4,
            Format::D32_SFLOAT,
        )
        .unwrap(),
    )
    .unwrap();

    let color_view = ImageView::new_default(
        AttachmentImage::transient_multisampled(
            device.clone(),
            dimensions,
            SampleCount::Sample4,
            swapchain_images[0].format().unwrap(),
        )
        .unwrap(),
    )
    .unwrap();

    let framebuffers = swapchain_images
        .iter()
        .map(|image| {
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![image.clone(), color_view.clone(), depth_view.clone()],
                    ..Default::default()
                },
            )
        })
        .collect::<Result<_, _>>()
        .unwrap();

    (framebuffers, color_view, depth_view)
}

pub fn create_pipeline(
    render_pass: Arc<RenderPass>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
    viewport: Viewport,
    device: Arc<Device>,
) -> Arc<GraphicsPipeline> {
    let pipeline = GraphicsPipeline::start()
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .input_assembly_state(InputAssemblyState::new())
        .multisample_state(MultisampleState {
            rasterization_samples: SampleCount::Sample4,
            ..Default::default()
        })
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant(Some(
            viewport,
        )))
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        .build(device.clone())
        .unwrap();

    pipeline
}
