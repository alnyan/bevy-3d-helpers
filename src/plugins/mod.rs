use bevy::{
    app::PluginGroupBuilder, asset::AssetPlugin, input::InputPlugin, prelude::PluginGroup,
    window::WindowPlugin,
};

pub use self::renderer::RendererPlugin;
use self::{camera::CameraPlugin, loader::LoaderPlugin};

pub struct DefaultRendererPlugins;

pub mod camera;
pub mod loader;
pub mod renderer;

impl PluginGroup for DefaultRendererPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        // Dependencies
        group.add(AssetPlugin);
        group.add(InputPlugin);
        group.add(WindowPlugin);

        group.add(LoaderPlugin);
        group.add(RendererPlugin);
        group.add(CameraPlugin);
    }
}
