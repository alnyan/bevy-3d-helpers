use std::sync::Arc;

use bevy::{
    asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
    math::UVec2,
    prelude::{AddAsset, Assets, CoreStage, Plugin, ResMut, Res, debug},
};
use image::EncodableLayout;
use vulkano::{format::Format, device::Queue};

use crate::renderer::material::TextureImage;

pub struct LoaderPlugin;
pub struct TextureImageLoader;

impl AssetLoader for TextureImageLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let image = image::load_from_memory(bytes)
                .map_err(bevy::asset::Error::new)?
                .to_rgba8();
            let width = image.width();
            let height = image.height();

            load_context.set_default_asset(LoadedAsset::new(TextureImage::from_bytes(
                image.as_bytes(),
                Format::R8G8B8A8_UNORM,
                UVec2::new(width, height),
            )));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["png"]
    }
}

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset_loader(TextureImageLoader)
            .add_system_to_stage(CoreStage::PreUpdate, upload_textures);
    }

    fn name(&self) -> &str {
        "alnyan-loader"
    }
}

fn upload_textures(mut textures: ResMut<Assets<TextureImage>>, queue: Res<Arc<Queue>>) {
    for (handle, texture) in textures.iter_mut() {
        if texture.image.is_none() {
            debug!("Uploading texture {:?} to GPU\n", handle);
            texture.upload_to_gpu(queue.clone());
        }
    }
}
