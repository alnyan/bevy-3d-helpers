use std::sync::Arc;

use bevy::{
    math::Vec4,
    pbr::StandardMaterial,
    prelude::{Assets, Color, Component, Image, Res},
};
use vulkano::{
    device::Queue,
    format::Format,
    image::{view::ImageView, ImageDimensions, ImmutableImage, MipmapsCount},
    sync::GpuFuture,
};

#[derive(Component)]
pub struct DisplayMaterial {
    pub k_diffuse: Color,
    pub k_diffuse_map: Option<Arc<ImageView<ImmutableImage>>>,
}

impl DisplayMaterial {
    pub fn from_standard_material(
        material: &StandardMaterial,
        images: &Res<Assets<Image>>,
        queue: Arc<Queue>,
    ) -> Option<Self> {
        let k_diffuse_map = if let Some(handle) = &material.base_color_texture {
            eprintln!("Uploading a texture");

            if let Some(src_image) = images.get(handle) {
                let dim = src_image.size();
                let (image, init) = ImmutableImage::from_iter(
                    src_image.data.clone(),
                    ImageDimensions::Dim2d {
                        width: dim.x as u32,
                        height: dim.y as u32,
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

                Some(ImageView::new_default(image).unwrap())
            } else {
                return None;
            }
        } else {
            None
        };

        Some(Self {
            k_diffuse: material.base_color,
            k_diffuse_map,
        })
    }
}
