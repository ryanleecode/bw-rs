use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};
use bw_bevy_assets::BWAssetsBevyPlugin;

#[derive(Default)]
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(bevy::log::LogPlugin::default());
        group.add(bevy::reflect::ReflectPlugin::default());
        group.add(bevy::core::CorePlugin::default());
        group.add(bevy::transform::TransformPlugin::default());
        group.add(bevy::diagnostic::DiagnosticsPlugin::default());
        group.add(bevy::input::InputPlugin::default());
        group.add(bevy::window::WindowPlugin::default());
        group.add(BWAssetsBevyPlugin::default());
        group.add(bevy::scene::ScenePlugin::default());
        group.add(bevy::render::RenderPlugin::default());
        group.add(bevy::sprite::SpritePlugin::default());
        group.add(bevy::pbr::PbrPlugin::default());
        group.add(bevy::ui::UiPlugin::default());
        group.add(bevy::text::TextPlugin::default());
        group.add(bevy::audio::AudioPlugin::default());
        group.add(bevy::winit::WinitPlugin::default());
        group.add(bevy::wgpu::WgpuPlugin::default());
    }
}
