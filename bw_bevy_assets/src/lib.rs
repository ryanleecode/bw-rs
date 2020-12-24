mod events;
mod io;
mod loader;
mod map;

use bevy::{
    asset::{free_unused_assets_system, FileAssetIo, HandleId},
    prelude::*,
    tasks::IoTaskPool,
};
pub use events::*;
use io::{MPQAssetIO, UnifiedMPQAssetIO};
pub use map::*;

pub struct AssetsServerSettings {
    pub asset_folder: String,
    pub stardat_name: String,
    pub broodat_name: String,
    pub patch_rt_name: String,
}

impl Default for AssetsServerSettings {
    fn default() -> Self {
        Self {
            asset_folder: "assets".to_string(),
            stardat_name: "STARDAT.MPQ".to_string(),
            broodat_name: "BROODAT.MPQ".to_string(),
            patch_rt_name: "patch_rt.mpq".to_string(),
        }
    }
}

#[derive(Default)]
pub struct BWAssetsBevyPlugin;

impl Plugin for BWAssetsBevyPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let task_pool = app
            .resources()
            .get::<IoTaskPool>()
            .expect("`IoTaskPool` resource not found.")
            .0
            .clone();

        let asset_server = {
            let AssetsServerSettings {
                asset_folder,
                stardat_name,
                broodat_name,
                patch_rt_name,
            } = &(*app
                .resources_mut()
                .get_or_insert_with(AssetsServerSettings::default));

            let fmt = |name: &str| format!("Failed to load {}", name);

            let mut source_io = UnifiedMPQAssetIO::new();

            for name in vec![stardat_name, broodat_name, patch_rt_name] {
                let dat = MPQAssetIO::new(&asset_folder, &name).expect(&fmt(&name));
                source_io.add_source(dat);
            }

            let file_asset_io = FileAssetIo::new(&asset_folder);
            source_io.add_source(file_asset_io);

            AssetServer::new(source_io, task_pool)
        };

        app.add_stage_before(
            stage::PRE_UPDATE,
            bevy::asset::stage::LOAD_ASSETS,
            SystemStage::parallel(),
        )
        .add_stage_after(
            stage::POST_UPDATE,
            bevy::asset::stage::ASSET_EVENTS,
            SystemStage::parallel(),
        )
        .add_resource(asset_server)
        .add_resource(MapsProcessingState::default())
        .add_asset::<loader::WPEs>()
        .add_asset::<loader::VX4s>()
        .add_asset::<loader::VR4s>()
        .add_asset::<loader::VF4s>()
        .add_asset::<loader::CV5s>()
        .add_asset::<loader::Map>()
        .add_asset::<TileAtlas>()
        .init_asset_loader::<loader::WPELoader>()
        .init_asset_loader::<loader::VX4Loader>()
        .init_asset_loader::<loader::VR4Loader>()
        .init_asset_loader::<loader::VF4Loader>()
        .init_asset_loader::<loader::CV5Loader>()
        .init_asset_loader::<loader::MapAssetLoader>()
        .register_type::<HandleId>()
        .add_event::<LoadMapEvent>()
        .add_event::<TileAtlasLoadedEvent>()
        .add_system_to_stage(stage::PRE_UPDATE, free_unused_assets_system.system())
        .add_system_to_stage(stage::POST_UPDATE, load_map_event_listener.system())
        .add_system_to_stage(
            bevy::asset::stage::ASSET_EVENTS,
            begin_processing_map_event_listener.system(),
        )
        .add_system_to_stage(bevy::asset::stage::ASSET_EVENTS, load_tile_atlas.system())
        .add_system_to_stage(bevy::asset::stage::ASSET_EVENTS, map_processor.system());
    }
}
