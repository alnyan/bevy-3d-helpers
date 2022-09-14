use bevy::{
    app::PluginGroupBuilder, asset::AssetPlugin, input::InputPlugin, prelude::PluginGroup,
    window::WindowPlugin,
};

use self::camera::CameraPlugin;
pub use self::renderer::RendererPlugin;

pub struct DefaultRendererPlugins;

pub mod camera;
pub mod renderer;

impl PluginGroup for DefaultRendererPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        // Dependencies
        group.add(AssetPlugin);
        group.add(InputPlugin);
        group.add(WindowPlugin);

        group.add(RendererPlugin);
        group.add(CameraPlugin);
    }
}
