use std::{collections::HashMap, path::PathBuf};

use crate::{Dimensions, LoadMapEvent, TileAtlasLoadedEvent};

use self::texture::{
    create_tile_map_texture_buffer, RGBAImageBuffer, MINITILE_PX_SIDE_LEN, TILE_PADDING,
};

use super::loader;
use bevy::{
    math::vec2,
    prelude::*,
    render::texture::{Extent3d, TextureDimension, TextureFormat},
    tasks::AsyncComputeTaskPool,
};
use bw_assets::map::MEGATILE_SIDE_LEN;
pub use tile_atlas::*;

mod texture;
mod tile_atlas;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TilesetHandles {
    cv5s: Handle<loader::CV5s>,
    vr4s: Handle<loader::VR4s>,
    vf4s: Handle<loader::VF4s>,
    vx4s: Handle<loader::VX4s>,
    wpes: Handle<loader::WPEs>,
}

#[derive(Debug, Eq, PartialEq)]
enum ProcessingState {
    NotProcessed,
    Processing {
        tileset_handles: TilesetHandles,
        tilemap_atlas_handle: Option<Handle<TileAtlas>>,
    },
    Processed,
}

impl Default for ProcessingState {
    fn default() -> Self {
        ProcessingState::NotProcessed
    }
}

#[derive(Debug, Default)]
pub(crate) struct MapsProcessingState {
    processing_state: HashMap<Handle<loader::Map>, ProcessingState>,
}

pub(crate) fn load_map_event_listener(
    mut state: ResMut<MapsProcessingState>,
    mut event_reader: Local<EventReader<LoadMapEvent>>,
    load_map_events: Res<Events<LoadMapEvent>>,
    asset_server: Res<AssetServer>,
) {
    for load_map_event in event_reader.iter(&load_map_events) {
        let map_handle: Handle<loader::Map> = asset_server.load(load_map_event.path.clone());
        state
            .processing_state
            .insert(map_handle, ProcessingState::NotProcessed);
    }
}

pub(crate) fn begin_processing_map_event_listener(
    mut state: ResMut<MapsProcessingState>,
    maps: Res<Assets<loader::Map>>,
    asset_server: Res<AssetServer>,
) {
    for (map_handle, map_processing_state) in &mut state.processing_state {
        if let (Some(map), ProcessingState::NotProcessed) =
            (maps.get(map_handle), &map_processing_state)
        {
            macro_rules! load {
                ($ext:literal) => {{
                    asset_server.load(PathBuf::from(format!(
                        "tileset\\{}.{}",
                        map.tileset.name(),
                        $ext
                    )))
                }};
            }

            let tileset_handles = TilesetHandles {
                cv5s: load!("cv5"),
                vr4s: load!("vr4"),
                vf4s: load!("vf4"),
                vx4s: load!("vx4"),
                wpes: load!("wpe"),
            };

            *map_processing_state = ProcessingState::Processing {
                tileset_handles,
                tilemap_atlas_handle: None,
            };
        }
    }
}

#[cfg(feature = "ui")]
pub(crate) fn load_tile_atlas(
    mut state: ResMut<MapsProcessingState>,
    task_pool: Res<AsyncComputeTaskPool>,
    vr4_assets: Res<Assets<loader::VR4s>>,
    wpes_assets: Res<Assets<loader::WPEs>>,
    cv5s_assets: Res<Assets<loader::CV5s>>,
    vx4s_assets: Res<Assets<loader::VX4s>>,
    maps: Res<Assets<loader::Map>>,
    mut textures: ResMut<Assets<Texture>>,
    mut tile_atlases: ResMut<Assets<TileAtlas>>,
    mut tile_atlas_loaded_events: ResMut<Events<TileAtlasLoadedEvent>>,
) {
    for (map_handle, map_processing_state) in &mut state.processing_state {
        if let ProcessingState::Processing {
            tileset_handles,
            tilemap_atlas_handle: tilemap_atlas_handle @ None,
            ..
        } = map_processing_state
        {
            if let (Some(vr4s), Some(wpes), Some(cv5s), Some(vx4s), Some(map)) = (
                vr4_assets.get(&tileset_handles.vr4s),
                wpes_assets.get(&tileset_handles.wpes),
                cv5s_assets.get(&tileset_handles.cv5s),
                vx4s_assets.get(&tileset_handles.vx4s),
                maps.get(map_handle.clone()),
            ) {
                let buffer = create_tile_map_texture_buffer((**task_pool).clone(), vr4s, wpes);
                let tilemap_texture = process_texture(buffer);
                let texture_dimen = tilemap_texture.size;
                let tilemap_texture_handle = textures.add(tilemap_texture);

                let mut tile_atlas = TileAtlas::from_grid_with_padding(
                    tilemap_texture_handle,
                    vec2(MINITILE_PX_SIDE_LEN as f32, MINITILE_PX_SIDE_LEN as f32),
                    vec2(texture_dimen.width as f32, texture_dimen.height as f32),
                    PaddingSpecification::uniform(TILE_PADDING as u32),
                );

                let tiles = create_minitiles((**task_pool).clone(), map, cv5s, vx4s);

                tile_atlas.insert_tiles(tiles);

                *tilemap_atlas_handle = Some(tile_atlases.add(tile_atlas));

                tile_atlas_loaded_events.send(TileAtlasLoadedEvent {
                    handle: (*tilemap_atlas_handle).clone().unwrap(),
                    map_dimensions: Dimensions {
                        width: map.dimensions.width as u32 * MEGATILE_SIDE_LEN,
                        height: map.dimensions.height as u32 * MEGATILE_SIDE_LEN,
                    },
                });
            }
        }
    }
}

pub(crate) fn map_processor(mut state: ResMut<MapsProcessingState>) {
    for map_processing_state in &mut state.processing_state.values_mut() {
        if let ProcessingState::Processing {
            tilemap_atlas_handle: Some(_),
            ..
        } = map_processing_state
        {
            *map_processing_state = ProcessingState::Processed;
        }
    }

    state
        .processing_state
        .retain(|_, processing_state| *processing_state != ProcessingState::Processed);
}

#[cfg(feature = "ui")]
fn process_texture(buffer: RGBAImageBuffer) -> Texture {
    Texture::new(
        Extent3d::new(buffer.width(), buffer.height(), 1),
        TextureDimension::D2,
        buffer.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
    )
}
