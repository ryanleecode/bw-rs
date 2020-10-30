use super::TilesetHandles;
use amethyst::{
    assets::{Handle, ProgressCounter},
    core::ecs::World,
};
use bw_assets::map::Map;

mod minimap;
pub mod resources;

pub use self::minimap::{
    Minimap, MinimapMarker, MinimapMarkerCameraTrackingSystem, MinimapMouseMovementTrackingSystem,
};

pub fn create(
    params: (
        &mut World,
        &Handle<Map>,
        &TilesetHandles,
        &mut ProgressCounter,
    ),
) {
    let (world, map_handle, tileset_handles, progress_counter) = params;

    let minimap_texture_handle =
        minimap::load_minimap_texture((world, map_handle, tileset_handles, progress_counter));

    Minimap::attach((world, map_handle, &minimap_texture_handle));
    MinimapMarker::attach(world, map_handle);

    resources::load_fonts(world, progress_counter)
}
