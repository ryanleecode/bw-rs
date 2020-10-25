use super::TilesetHandles;
use amethyst::{
    assets::{AssetStorage, Handle, Loader, ProgressCounter},
    core::ecs::World,
    prelude::*,
    renderer::{ImageFormat, Texture},
};
use bw_assets::map::Map;

mod minimap;
pub mod resources;

pub use minimap::MinimapMarkerCameraTrackingSystem;

pub fn create(
    params: (
        &mut World,
        &Handle<Map>,
        &TilesetHandles,
        &mut ProgressCounter,
    ),
) {
    let (world, map, tileset_handles, progress_counter) = params;

    let minimap_texture_handle =
        minimap::load_minimap_texture((world, map, tileset_handles, progress_counter));
    minimap::create_minimap_entity((world, map, &minimap_texture_handle));

    let white_square_texture_handle = load_white_square_texture((world, progress_counter));
    minimap::create_minimap_marker_entity((world, white_square_texture_handle));

    resources::load_fonts(world, progress_counter)
}

fn load_white_square_texture(params: (&mut World, &mut ProgressCounter)) -> Handle<Texture> {
    let (world, progress_counter) = params;

    let loader = world.read_resource::<Loader>();
    loader.load(
        "texture/transparent_rectangle.png",
        ImageFormat::default(),
        progress_counter,
        &world.read_resource::<AssetStorage<Texture>>(),
    )
}
