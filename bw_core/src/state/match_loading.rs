use crate::config::BWConfig;

use crate::graphics::{self};
use amethyst::{
    assets::{AssetStorage, Completion, Handle, Loader, ProgressCounter},
    prelude::*,
    SimpleState, SimpleTrans,
};
use bw_assets::mpq::MPQSource;
use bw_assets::{
    map::{Map, MapFormat, MapHandle},
    mpq::{self, ArcMPQ},
};
use log::{error, info, warn};
use std::path::PathBuf;

pub enum MapAssetsLoadingState {
    Idle,
    BWAssets(Vec<Handle<ArcMPQ>>),
    Map(MapHandle),
    Tileset(graphics::tile::resources::TilesetHandles, MapHandle),
    Done,
}

pub struct MatchLoadingState {
    assets_dir: PathBuf,
    progress_counter: ProgressCounter,
    map_assets_loading_state: MapAssetsLoadingState,
    config: BWConfig,
}

impl MatchLoadingState {
    pub fn new(assets_dir: PathBuf, config: BWConfig) -> MatchLoadingState {
        MatchLoadingState {
            assets_dir,
            config,
            map_assets_loading_state: MapAssetsLoadingState::Idle,
            progress_counter: ProgressCounter::new(),
        }
    }
}

impl MatchLoadingState {
    fn load_mpq(&mut self, world: &mut World, path: &str) -> Handle<ArcMPQ> {
        let mpq_asset_path = self.assets_dir.join(path);
        let loader = world.read_resource::<Loader>();

        loader.load_from_data_async(
            move || {
                mpq::ArcMPQ::from_path(&mpq_asset_path.as_path()).expect(&format!(
                    "failed to load MPQ: {}",
                    mpq_asset_path.as_path().display()
                ))
            },
            &mut self.progress_counter,
            &world.read_resource::<AssetStorage<ArcMPQ>>(),
        )
    }
}

impl SimpleState for MatchLoadingState {
    fn on_start(&mut self, _: StateData<'_, GameData<'_, '_>>) {
        info!("MatchLoadingState started");
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let StateData { world, .. } = data;

        match self.progress_counter.complete() {
            Completion::Complete => match &self.map_assets_loading_state {
                MapAssetsLoadingState::Idle => {
                    self.map_assets_loading_state = MapAssetsLoadingState::BWAssets(vec![
                        self.load_mpq(world, "STARDAT.MPQ"),
                        self.load_mpq(world, "BROODAT.MPQ"),
                        self.load_mpq(world, "patch_rt.mpq"),
                    ]);

                    Trans::None
                }
                MapAssetsLoadingState::BWAssets(mpq_handles) => {
                    let mpq_source = {
                        let mut mpq_source = MPQSource::new();
                        let mpq_storage = world.read_resource::<AssetStorage<ArcMPQ>>();

                        mpq_handles
                            .into_iter()
                            .map(|handle| {
                                mpq_storage
                                    .get(handle)
                                    .expect("mpq resource is missing")
                                    .clone()
                            })
                            .for_each(|mpq| mpq_source.push_front(mpq.clone()));

                        mpq_source
                    };

                    {
                        let mut loader = world.write_resource::<Loader>();
                        loader.add_source("bw_assets", mpq_source);
                    }

                    let map_handle = world.read_resource::<Loader>().load(
                        format!("maps/{}", self.config.map),
                        MapFormat,
                        &mut self.progress_counter,
                        &world.read_resource::<AssetStorage<Map>>(),
                    );

                    // To be used in the `Tile` trait.
                    world.insert(map_handle.clone());
                    // Handle needs to be an Option for ReadStorage because it
                    // must implement default.
                    world.insert(Some(map_handle.clone()));

                    self.map_assets_loading_state = MapAssetsLoadingState::Map(map_handle);

                    Trans::None
                }
                MapAssetsLoadingState::Map(map_handle) => {
                    let tileset_handles = graphics::tile::resources::load(
                        world,
                        map_handle.clone(),
                        &mut self.progress_counter,
                    );

                    world.insert(tileset_handles.clone());

                    self.map_assets_loading_state =
                        MapAssetsLoadingState::Tileset(tileset_handles, (*map_handle).clone());

                    Trans::None
                }
                MapAssetsLoadingState::Tileset(tileset_handles, map_handle) => {
                    graphics::create((
                        world,
                        map_handle,
                        tileset_handles,
                        &mut self.progress_counter,
                    ));

                    self.map_assets_loading_state = MapAssetsLoadingState::Done;

                    Trans::None
                }
                MapAssetsLoadingState::Done => Trans::Switch(Box::new(super::GameplayState::new())),
            },
            Completion::Failed => {
                for err_meta in self.progress_counter.errors() {
                    warn!(
                        "Failed to load asset: {} of type {}: {}",
                        err_meta.asset_name, err_meta.asset_type_name, err_meta.error
                    );
                }
                error!("Failed to initialize game.");

                Trans::Quit
            }
            Completion::Loading => Trans::None,
        }
    }
}
