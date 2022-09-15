use std::sync::Arc;

use bevy::{
    math::UVec2,
    prelude::{Color, Component, Handle},
    reflect::TypeUuid,
};
use vulkano::{
    device::Queue,
    format::Format,
    image::{view::ImageView, ImageDimensions, ImmutableImage, MipmapsCount},
    sync::GpuFuture,
};

#[derive(Component, TypeUuid)]
#[uuid = "de491a16-cf4c-4ef9-8f02-0f7837b4dea8"]
pub struct DisplayMaterial {
    pub k_diffuse: Color,
    pub k_diffuse_map: Option<Handle<TextureImage>>,
}

#[derive(Component, TypeUuid)]
#[uuid = "e42d2fc2-aad3-4b25-a50e-dd2232099d4a"]
pub struct TextureImage {
    pub data: Vec<u8>,
    pub format: Format,
    pub dimensions: UVec2,
    pub image: Option<Arc<ImageView<ImmutableImage>>>,
}

impl TextureImage {
    pub fn from_bytes(data: &[u8], format: Format, dimensions: UVec2) -> Self {
        Self {
            data: Vec::from(data),
            dimensions,
            format,
            image: None,
        }
    }

    pub fn upload_to_gpu(&mut self, queue: Arc<Queue>) {
        let (image, init) = ImmutableImage::from_iter(
            self.data.clone(),
            ImageDimensions::Dim2d {
                width: self.dimensions.x,
                height: self.dimensions.y,
                array_layers: 1,
            },
            MipmapsCount::One,
            Format::R8G8B8A8_UNORM,
            queue,
        )
        .unwrap();

        init.then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        self.image = Some(ImageView::new_default(image).unwrap());
    }
}
