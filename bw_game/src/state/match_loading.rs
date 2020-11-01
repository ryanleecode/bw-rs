use crate::{
    config::BWConfig,
    graphics::{
        tile::TilesetHandles,
        ui::{resources::load_dats, Minimap, MinimapMarker},
    },
};

use crate::graphics::{self};
use amethyst::{
    assets::{AssetStorage, Completion, Handle, Loader, ProgressCounter},
    core::Transform,
    ecs::storage::MaskedStorage,
    prelude::*,
    renderer::ActiveCamera,
    renderer::Camera,
    ui::UiCreator,
    SimpleState, SimpleTrans,
};
use bw_assets::{
    map::{Map, MapFormat, MapHandle},
    mpq::{self, ArcMPQ},
    tileset::VR4sAsset,
    tileset::{CV5sAsset, VF4sAsset, WPEsAsset},
};
use bw_assets::{mpq::MPQSource, tileset::VX4sAsset};
use log::{error, info, warn};
use std::{path::PathBuf, sync::Arc};

pub enum MapAssetsLoadingState {
    Idle,
    BWAssets(Vec<Handle<ArcMPQ>>),
    Map(MapHandle),
    Prefabs(TilesetHandles, MapHandle),
    Tileset(MapHandle),
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
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        info!("MatchLoadingState started");

        let StateData { world, .. } = data;

        world.insert(MaskedStorage::<Handle<Map>>::default());
        world.insert(MaskedStorage::<Minimap>::default());
        world.insert(MaskedStorage::<MinimapMarker>::default());
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

                    load_dats(world, &mut self.progress_counter);
                    let map_handle = world.read_resource::<Loader>().load(
                        format!("maps/{}", self.config.map),
                        MapFormat,
                        &mut self.progress_counter,
                        &world.read_resource::<AssetStorage<Map>>(),
                    );

                    // To be used in the `Tile` trait.
                    world.insert(map_handle.clone());

                    self.map_assets_loading_state = MapAssetsLoadingState::Map(map_handle);

                    Trans::None
                }
                MapAssetsLoadingState::Map(map_handle) => {
                    let tileset_handles = graphics::tile::resources::load(
                        world,
                        map_handle.clone(),
                        &mut self.progress_counter,
                    );

                    self.map_assets_loading_state =
                        MapAssetsLoadingState::Prefabs(tileset_handles, (*map_handle).clone());

                    Trans::None
                }
                MapAssetsLoadingState::Prefabs(tileset_handles, map_handle) => {
                    {
                        let cv5s = world
                            .write_resource::<AssetStorage<CV5sAsset>>()
                            .get_mut(&tileset_handles.cv5s)
                            .and_then(|asset| asset.take())
                            .expect("cv5s is missing");
                        world.insert(Arc::new(cv5s));
                    }

                    {
                        let vf4s = world
                            .write_resource::<AssetStorage<VF4sAsset>>()
                            .get_mut(&tileset_handles.vf4s)
                            .and_then(|asset| asset.take())
                            .expect("vf4s is missing");
                        world.insert(Arc::new(vf4s));
                    }

                    {
                        let vr4s = world
                            .write_resource::<AssetStorage<VR4sAsset>>()
                            .get_mut(&tileset_handles.vr4s)
                            .and_then(|asset| asset.take())
                            .expect("vr4s is missing");
                        world.insert(Arc::new(vr4s));
                    }


                    {
                        let vx4s = world
                            .write_resource::<AssetStorage<VX4sAsset>>()
                            .get_mut(&tileset_handles.vx4s)
                            .and_then(|asset| asset.take())
                            .expect("vx4s is missing");
                        world.insert(Arc::new(vx4s));
                    }

                    {
                        let wpes = world
                            .write_resource::<AssetStorage<WPEsAsset>>()
                            .get_mut(&tileset_handles.wpes)
                            .and_then(|asset| asset.take())
                            .expect("wpes is missing");
                        world.insert(Arc::new(wpes));
                    }

                    let progress_counter = &mut self.progress_counter;
                    world.exec(|mut creator: UiCreator<'_>| {
                        creator.create("ui/hud.ron", progress_counter);
                    });

                    self.map_assets_loading_state =
                        MapAssetsLoadingState::Tileset(map_handle.clone());

                    Trans::None
                }
                MapAssetsLoadingState::Tileset(map_handle) => {
                    graphics::create((world, map_handle, &mut self.progress_counter));

                    initialize_camera(world, map_handle);

                    self.map_assets_loading_state = MapAssetsLoadingState::Done;

                    Trans::None
                }
                MapAssetsLoadingState::Done => {
                    Trans::Switch(Box::new(super::GameplayState::default()))
                }
            },
            Completion::Failed => {
                for err_meta in self.progress_counter.errors() {
                    warn!(
                        "Failed to load asset: {} of type {}: {} {:#?}",
                        err_meta.asset_name,
                        err_meta.asset_type_name,
                        err_meta.error,
                        err_meta.error.causes()
                    );
                }
                error!("Failed to initialize game.");

                Trans::Quit
            }
            Completion::Loading => Trans::None,
        }
    }
}

fn initialize_camera(world: &mut World, map_handle: &Handle<Map>) {
    let camera_width = 640.0;
    let camera_height = 500.0;

    let camera = Camera::orthographic(0.0, camera_width, 0.0, camera_height, 0.0, 20.0);
    let mut camera_transform = Transform::default();
    camera_transform.set_translation_xyz(-(camera_width / 2.0), camera_height / 2.0, 10.0);

    let camera_entity = world
        .create_entity()
        .with(camera)
        .with(camera_transform)
        .with(map_handle.clone())
        .build();
    world.fetch_mut::<ActiveCamera>().entity = Some(camera_entity);
}
