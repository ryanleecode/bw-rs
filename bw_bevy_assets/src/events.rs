use std::path::PathBuf;

use bevy::prelude::Handle;

use crate::TileAtlas;

#[derive(Debug)]
pub struct LoadMapEvent {
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct TileAtlasLoadedEvent {
    pub handle: Handle<TileAtlas>,
    // Dimensions in tiles
    pub map_dimensions: Dimensions,
}
