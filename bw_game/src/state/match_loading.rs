use crate::{
    config::BWConfig,
    graphics::{
        tile::TilesetHandles,
        ui::{resources::load_dats, resources::DatHandles, Minimap, MinimapMarker},
    },
};

use crate::graphics::{self};
use amethyst::{
    assets::{AssetStorage, Completion, Handle, Loader, ProgressCounter},
    core::Transform,
    ecs::storage::MaskedStorage,
    ecs::Entity,
    prelude::*,
    renderer::ActiveCamera,
    renderer::Camera,
    ui::UiCreator,
    SimpleState, SimpleTrans,
};
use bw_assets::{
    dat::UnitDat,
    dat::{FlingyDat, FlingyDatAsset, UnitDatAsset},
    map::{Map, MapFormat, MapHandle},
    mpq::MPQHandle,
    mpq::{self, ArcMPQ},
    tileset::CV5s,
    tileset::VR4sAsset,
    tileset::{CV5sAsset, VF4s, VF4sAsset, VR4s, VX4s, WPEs, WPEsAsset},
};
use bw_assets::{mpq::MPQSource, tileset::VX4sAsset};
use log::{error, info, warn};
use std::{path::PathBuf, sync::Arc};

pub struct MPQHandles {
    stardat: MPQHandle,
    broodat: MPQHandle,
    patchrt: MPQHandle,
}

pub struct MatchLoadingState {
    assets_dir: PathBuf,
    mpq_handles: Option<MPQHandles>,
    are_mpqs_loaded: bool,
    are_tilesets_loaded: bool,
    are_graphics_loaded: bool,
    tileset_handles: Option<TilesetHandles>,
    dat_handles: Option<DatHandles>,
    map_handle: Option<MapHandle>,
    ui: Option<Entity>,
    progress_counter: ProgressCounter,
    config: BWConfig,
}

impl MatchLoadingState {
    pub fn new(assets_dir: PathBuf, config: BWConfig) -> MatchLoadingState {
        MatchLoadingState {
            assets_dir,
            config,
            mpq_handles: None,
            tileset_handles: None,
            are_mpqs_loaded: false,
            are_tilesets_loaded: false,
            are_graphics_loaded: false,
            dat_handles: None,
            map_handle: None,
            ui: None,
            progress_counter: ProgressCounter::default(),
        }
    }
}

impl MatchLoadingState {
    fn load_mpq(&mut self, world: &mut World, path: &str) -> MPQHandle {
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

    fn is_complete(&self) -> bool {
        self.mpq_handles.is_some()
            && self.are_mpqs_loaded
            && self.are_tilesets_loaded
            && self.are_graphics_loaded
            && self.tileset_handles.is_some()
            && self.map_handle.is_some()
            && self.ui.is_some()
            && self.progress_counter.is_complete()
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

        if self.ui.is_none() {
            let progress_counter = &mut self.progress_counter;
            self.ui =
                Some(world.exec(|mut creator: UiCreator<'_>| {
                    creator.create("ui/hud.ron", progress_counter)
                }));
        }

        if self.mpq_handles.is_none() {
            self.mpq_handles = Some(MPQHandles {
                stardat: self.load_mpq(world, "STARDAT.MPQ"),
                broodat: self.load_mpq(world, "BROODAT.MPQ"),
                patchrt: self.load_mpq(world, "patch_rt.mpq"),
            });
        }

        if let Some(mpq_handles) = &self.mpq_handles {
            let mpq_storage = world.read_resource::<AssetStorage<ArcMPQ>>();
            if let (Some(stardat), Some(broodat), Some(patchrt)) = (
                mpq_storage.get(&mpq_handles.stardat),
                mpq_storage.get(&mpq_handles.broodat),
                mpq_storage.get(&mpq_handles.patchrt),
            ) {
                let mut mpq_source = MPQSource::new();
                mpq_source.push_front(stardat.clone());
                mpq_source.push_front(broodat.clone());
                mpq_source.push_front(patchrt.clone());
                let mut loader = world.write_resource::<Loader>();
                loader.add_source("bw_assets", mpq_source);
                self.are_mpqs_loaded = true;
            }
        }

        if self.map_handle.is_none() && self.are_mpqs_loaded {
            let map_handle = world.read_resource::<Loader>().load(
                format!("maps/{}", self.config.map),
                MapFormat,
                &mut self.progress_counter,
                &world.read_resource::<AssetStorage<Map>>(),
            );
            world.insert(map_handle.clone());
            self.map_handle = Some(map_handle);
        }

        if self.dat_handles.is_none() && self.are_mpqs_loaded {
            self.dat_handles = Some(load_dats(world, &mut self.progress_counter));
        }

        if let Some(dat_handles) = &self.dat_handles {
            if !world.has_value::<UnitDat>() {
                let units_dat_opt = world
                    .write_resource::<AssetStorage<UnitDatAsset>>()
                    .get_mut(&dat_handles.units_dat)
                    .and_then(|asset| asset.take());
                if let Some(units_dat) = units_dat_opt {
                    world.insert::<UnitDat>(units_dat);
                }
            }
            if !world.has_value::<FlingyDat>() {
                let flingy_dat_opt = world
                    .write_resource::<AssetStorage<FlingyDatAsset>>()
                    .get_mut(&dat_handles.flingy_dat)
                    .and_then(|asset| asset.take());
                if let Some(flingy_dat) = flingy_dat_opt {
                    world.insert::<FlingyDat>(flingy_dat);
                }
            }
        }

        if let (Some(map_handle), None) = (&self.map_handle, &self.tileset_handles) {
            let tileset_handles = graphics::tile::resources::load(
                world,
                map_handle.clone(),
                &mut self.progress_counter,
            );

            self.tileset_handles = tileset_handles;
        }

        if let (Some(tileset_handles), false) = (&self.tileset_handles, self.are_tilesets_loaded) {
            if !world.has_value::<Arc<CV5s>>() {
                let cv5s_opt = world
                    .write_resource::<AssetStorage<CV5sAsset>>()
                    .get_mut(&tileset_handles.cv5s)
                    .and_then(|asset| asset.take())
                    .map(Arc::new);
                if let Some(cv5s) = cv5s_opt {
                    world.insert::<Arc<CV5s>>(cv5s);
                }
            }
            if !world.has_value::<Arc<VF4s>>() {
                let vf4s_opt = world
                    .write_resource::<AssetStorage<VF4sAsset>>()
                    .get_mut(&tileset_handles.vf4s)
                    .and_then(|asset| asset.take())
                    .map(Arc::new);
                if let Some(vf4s) = vf4s_opt {
                    world.insert::<Arc<VF4s>>(vf4s);
                }
            }
            if !world.has_value::<Arc<VR4s>>() {
                let vr4s_opt = world
                    .write_resource::<AssetStorage<VR4sAsset>>()
                    .get_mut(&tileset_handles.vr4s)
                    .and_then(|asset| asset.take())
                    .map(Arc::new);
                if let Some(vr4s) = vr4s_opt {
                    world.insert::<Arc<VR4s>>(vr4s);
                }
            }
            if !world.has_value::<Arc<VX4s>>() {
                let vx4s_opt = world
                    .write_resource::<AssetStorage<VX4sAsset>>()
                    .get_mut(&tileset_handles.vx4s)
                    .and_then(|asset| asset.take())
                    .map(Arc::new);
                if let Some(vx4s) = vx4s_opt {
                    world.insert::<Arc<VX4s>>(vx4s);
                }
            }
            if !world.has_value::<Arc<WPEs>>() {
                let wpes_opts = world
                    .write_resource::<AssetStorage<WPEsAsset>>()
                    .get_mut(&tileset_handles.wpes)
                    .and_then(|asset| asset.take())
                    .map(Arc::new);
                if let Some(wpes) = wpes_opts {
                    world.insert::<Arc<WPEs>>(wpes);
                }
            }

            self.are_tilesets_loaded = world.has_value::<Arc<CV5s>>()
                && world.has_value::<Arc<VF4s>>()
                && world.has_value::<Arc<VR4s>>()
                && world.has_value::<Arc<VX4s>>()
                && world.has_value::<Arc<WPEs>>()
        }

        if let Some(map_handle) = &self.map_handle {
            initialize_camera(world, map_handle);
        }

        if let (Some(map_handle), true, false) = (
            &self.map_handle,
            self.are_tilesets_loaded,
            self.are_graphics_loaded,
        ) {
            graphics::create((world, map_handle, &mut self.progress_counter));
            self.are_graphics_loaded = true
        }

        if Completion::Failed == self.progress_counter.complete() {
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
        } else if self.is_complete() {
            Trans::Push(Box::new(super::GameplayState::default()))
        } else {
            Trans::None
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
