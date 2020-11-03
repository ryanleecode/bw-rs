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
    dat::UnitsDat,
    dat::{FlingyDat, FlingyDatAsset, UnitsDatAsset, WeaponsDat, WeaponsDatAsset},
    map::{Map, MapFormat, MapHandle},
    mpq::MPQHandle,
    mpq::{self, ArcMPQ},
    tileset::CV5s,
    tileset::VR4sAsset,
    tileset::{CV5sAsset, VF4s, VF4sAsset, VR4s, VX4s, WPEs, WPEsAsset},
};
use bw_assets::{mpq::MPQSource, tileset::VX4sAsset};
use incremental_topo::IncrementalTopo;
use log::{error, info};
use std::{
    cell::Cell,
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum AssetType {
    MPQHandles,
    MPQSource,
    UIHud,
    Map,
    DatHandles,
    UnitsDat,
    FlingyDat,
    WeaponsDat,
    Camera,
    TilesetHandles,
    CV5s,
    VF4s,
    VR4s,
    VX4s,
    WPEs,
    Graphics,
}

impl Display for AssetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::MPQHandles => write!(f, "mpq_handles"),
            AssetType::MPQSource => write!(f, "mpq_source"),
            AssetType::UIHud => write!(f, "ui_hud"),
            AssetType::Map => write!(f, "map"),
            AssetType::DatHandles => write!(f, "dat_handles"),
            AssetType::UnitsDat => write!(f, "units.dat"),
            AssetType::FlingyDat => write!(f, "flingy.dat"),
            AssetType::WeaponsDat => write!(f, "weapons.dat"),
            AssetType::Camera => write!(f, "camera"),
            AssetType::TilesetHandles => write!(f, "tileset_handles"),
            AssetType::CV5s => write!(f, "cv5s"),
            AssetType::VF4s => write!(f, "vf4s"),
            AssetType::VR4s => write!(f, "vr4s"),
            AssetType::VX4s => write!(f, "vx4s"),
            AssetType::WPEs => write!(f, "wpes"),
            AssetType::Graphics => write!(f, "graphics"),
        }
    }
}

struct MPQHandles {
    stardat: MPQHandle,
    broodat: MPQHandle,
    patchrt: MPQHandle,
}

#[derive(Clone)]
struct Node {
    name: AssetType,
    loaded: Rc<Cell<bool>>,
}

impl Node {
    fn new(name: AssetType) -> Node {
        Node {
            name,
            loaded: Rc::from(Cell::from(false)),
        }
    }

    fn is_loaded(&self) -> bool {
        self.loaded.get()
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.name, self.is_loaded())
    }
}

impl Eq for Node {}

fn build_asset_dependency_graph() -> IncrementalTopo<Node> {
    let mut dag = IncrementalTopo::new();

    dag.add_node(Node::new(AssetType::MPQHandles));
    dag.add_node(Node::new(AssetType::MPQSource));

    dag.add_node(Node::new(AssetType::UIHud));
    dag.add_node(Node::new(AssetType::Map));

    dag.add_node(Node::new(AssetType::DatHandles));
    dag.add_node(Node::new(AssetType::UnitsDat));
    dag.add_node(Node::new(AssetType::FlingyDat));
    dag.add_node(Node::new(AssetType::WeaponsDat));

    dag.add_node(Node::new(AssetType::Camera));
    dag.add_node(Node::new(AssetType::TilesetHandles));

    dag.add_node(Node::new(AssetType::CV5s));
    dag.add_node(Node::new(AssetType::VF4s));
    dag.add_node(Node::new(AssetType::VR4s));
    dag.add_node(Node::new(AssetType::VX4s));
    dag.add_node(Node::new(AssetType::WPEs));

    dag.add_node(Node::new(AssetType::Graphics));

    dag.add_dependency(
        &Node::new(AssetType::MPQHandles),
        &Node::new(AssetType::MPQSource),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::MPQSource,
        AssetType::MPQHandles
    ));

    dag.add_dependency(
        &Node::new(AssetType::MPQSource),
        &Node::new(AssetType::DatHandles),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::DatHandles,
        AssetType::MPQSource
    ));
    dag.add_dependency(
        &Node::new(AssetType::DatHandles),
        &Node::new(AssetType::UnitsDat),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::UnitsDat,
        AssetType::DatHandles
    ));
    dag.add_dependency(
        &Node::new(AssetType::DatHandles),
        &Node::new(AssetType::FlingyDat),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::FlingyDat,
        AssetType::DatHandles
    ));
    dag.add_dependency(
        &Node::new(AssetType::DatHandles),
        &Node::new(AssetType::WeaponsDat),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::WeaponsDat,
        AssetType::DatHandles
    ));

    dag.add_dependency(
        &Node::new(AssetType::MPQSource),
        &Node::new(AssetType::TilesetHandles),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::TilesetHandles,
        AssetType::MPQSource
    ));
    dag.add_dependency(
        &Node::new(AssetType::Map),
        &Node::new(AssetType::TilesetHandles),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::TilesetHandles,
        AssetType::Map
    ));

    dag.add_dependency(
        &Node::new(AssetType::TilesetHandles),
        &Node::new(AssetType::CV5s),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::CV5s,
        AssetType::TilesetHandles
    ));

    dag.add_dependency(
        &Node::new(AssetType::TilesetHandles),
        &Node::new(AssetType::VF4s),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::VF4s,
        AssetType::TilesetHandles
    ));

    dag.add_dependency(
        &Node::new(AssetType::TilesetHandles),
        &Node::new(AssetType::VR4s),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::VR4s,
        AssetType::TilesetHandles
    ));

    dag.add_dependency(
        &Node::new(AssetType::TilesetHandles),
        &Node::new(AssetType::VX4s),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::VX4s,
        AssetType::TilesetHandles
    ));

    dag.add_dependency(
        &Node::new(AssetType::TilesetHandles),
        &Node::new(AssetType::WPEs),
    )
    .expect(&format!(
        "add {} <- {} dependency",
        AssetType::WPEs,
        AssetType::TilesetHandles
    ));

    dag.add_dependency(&Node::new(AssetType::Map), &Node::new(AssetType::Camera))
        .expect(&format!(
            "add {} <- {} dependency",
            AssetType::Camera,
            AssetType::Map
        ));

    dag.add_dependency(&Node::new(AssetType::CV5s), &Node::new(AssetType::Graphics))
        .expect(&format!(
            "add {} <- {} dependency",
            AssetType::CV5s,
            AssetType::Graphics
        ));
    dag.add_dependency(&Node::new(AssetType::VF4s), &Node::new(AssetType::Graphics))
        .expect(&format!(
            "add {} <- {} dependency",
            AssetType::VF4s,
            AssetType::Graphics
        ));
    dag.add_dependency(&Node::new(AssetType::VR4s), &Node::new(AssetType::Graphics))
        .expect(&format!(
            "add {} <- {} dependency",
            AssetType::VR4s,
            AssetType::Graphics
        ));
    dag.add_dependency(&Node::new(AssetType::VX4s), &Node::new(AssetType::Graphics))
        .expect(&format!(
            "add {} <- {} dependency",
            AssetType::VX4s,
            AssetType::Graphics
        ));
    dag.add_dependency(&Node::new(AssetType::WPEs), &Node::new(AssetType::Graphics))
        .expect(&format!(
            "add {} <- {} dependency",
            AssetType::WPEs,
            AssetType::Graphics
        ));

    dag
}

pub struct MatchLoadingState {
    assets_dir: PathBuf,
    mpq_handles: Option<MPQHandles>,
    tileset_handles: Option<TilesetHandles>,
    dat_handles: Option<DatHandles>,
    map_handle: Option<MapHandle>,
    ui: Option<Entity>,
    progress_counter: ProgressCounter,
    asset_dependency_graph: IncrementalTopo<Node>,
    config: BWConfig,
}

impl MatchLoadingState {
    pub fn new(assets_dir: PathBuf, config: BWConfig) -> MatchLoadingState {
        MatchLoadingState {
            assets_dir,
            config,
            mpq_handles: None,
            tileset_handles: None,
            asset_dependency_graph: build_asset_dependency_graph(),
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
        self.asset_dependency_graph
            .iter_unsorted()
            .filter(|(_, node)| !node.loaded.get())
            .collect::<Vec<_>>()
            .len()
            == 0
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

        let dependencies = self
            .asset_dependency_graph
            .iter_unsorted()
            .map(|(i, node)| (i, node.clone()))
            .collect::<Vec<_>>();
        for (_, node) in dependencies {
            if node.is_loaded() {
                continue;
            }
            let are_dependencies_satisfied = self
                .asset_dependency_graph
                .iter_unsorted()
                .filter(|(_, other)| {
                    self.asset_dependency_graph
                        .contains_dependency(other, &node)
                })
                .filter(|(_, dependency)| !dependency.is_loaded())
                .count()
                == 0;
            if !are_dependencies_satisfied {
                continue;
            }

            match node.name {
                AssetType::MPQHandles => {
                    self.mpq_handles = Some(MPQHandles {
                        stardat: self.load_mpq(world, "STARDAT.MPQ"),
                        broodat: self.load_mpq(world, "BROODAT.MPQ"),
                        patchrt: self.load_mpq(world, "patch_rt.mpq"),
                    });
                    node.loaded.set(true);
                }
                AssetType::MPQSource => {
                    let mpq_handles = self.mpq_handles.as_ref().expect("mpq handles are missing");
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
                        node.loaded.set(true);
                    }
                }
                AssetType::UIHud => {
                    let progress_counter = &mut self.progress_counter;
                    self.ui = Some(world.exec(|mut creator: UiCreator<'_>| {
                        creator.create("ui/hud.ron", progress_counter)
                    }));
                    node.loaded.set(true);
                }
                AssetType::Map => {
                    let map_handle = world.read_resource::<Loader>().load(
                        format!("maps/{}", self.config.map),
                        MapFormat,
                        &mut self.progress_counter,
                        &world.read_resource::<AssetStorage<Map>>(),
                    );
                    world.insert(map_handle.clone());
                    self.map_handle = Some(map_handle);
                    node.loaded.set(true);
                }
                AssetType::DatHandles => {
                    self.dat_handles = Some(load_dats(world, &mut self.progress_counter));
                    node.loaded.set(true);
                }
                AssetType::UnitsDat => {
                    let dat_handles = self.dat_handles.as_ref().expect("dat handles are missing");
                    let units_dat_opt = world
                        .write_resource::<AssetStorage<UnitsDatAsset>>()
                        .get_mut(&dat_handles.units_dat)
                        .and_then(|asset| asset.take());
                    if let Some(units_dat) = units_dat_opt {
                        world.insert::<UnitsDat>(units_dat);
                        node.loaded.set(true);
                    }
                }
                AssetType::FlingyDat => {
                    let dat_handles = self.dat_handles.as_ref().expect("dat handles are missing");
                    let flingy_dat_opt = world
                        .write_resource::<AssetStorage<FlingyDatAsset>>()
                        .get_mut(&dat_handles.flingy_dat)
                        .and_then(|asset| asset.take());
                    if let Some(flingy_dat) = flingy_dat_opt {
                        world.insert::<FlingyDat>(flingy_dat);
                        node.loaded.set(true);
                    }
                }
                AssetType::WeaponsDat => {
                    let dat_handles = self.dat_handles.as_ref().expect("dat handles are missing");
                    let weapons_dat_opt = world
                        .write_resource::<AssetStorage<WeaponsDatAsset>>()
                        .get_mut(&dat_handles.weapons_dat)
                        .and_then(|asset| asset.take());
                    if let Some(weapons_dat) = weapons_dat_opt {
                        world.insert::<WeaponsDat>(weapons_dat);
                        node.loaded.set(true);
                    }
                }
                AssetType::Camera => {
                    let map_handle = self.map_handle.as_ref().expect("map handle is missing");
                    let camera_width = 640.0;
                    let camera_height = 500.0;

                    let camera =
                        Camera::orthographic(0.0, camera_width, 0.0, camera_height, 0.0, 20.0);
                    let mut camera_transform = Transform::default();
                    camera_transform.set_translation_xyz(
                        -(camera_width / 2.0),
                        camera_height / 2.0,
                        10.0,
                    );

                    let camera_entity = world
                        .create_entity()
                        .with(camera)
                        .with(camera_transform)
                        .with(map_handle.clone())
                        .build();
                    world.fetch_mut::<ActiveCamera>().entity = Some(camera_entity);
                    node.loaded.set(true);
                }
                AssetType::TilesetHandles => {
                    let map_handle = self.map_handle.as_ref().expect("map handle is missing");
                    self.tileset_handles = graphics::tile::resources::load(
                        world,
                        map_handle.clone(),
                        &mut self.progress_counter,
                    );
                    node.loaded.set(true);
                }
                AssetType::CV5s => {
                    let tileset_handles = self
                        .tileset_handles
                        .as_ref()
                        .expect("tileset handles not loaded");
                    let cv5s_opt = world
                        .write_resource::<AssetStorage<CV5sAsset>>()
                        .get_mut(&tileset_handles.cv5s)
                        .and_then(|asset| asset.take())
                        .map(Arc::new);
                    if let Some(cv5s) = cv5s_opt {
                        world.insert::<Arc<CV5s>>(cv5s);
                        node.loaded.set(true);
                    }
                }
                AssetType::VF4s => {
                    let tileset_handles = self
                        .tileset_handles
                        .as_ref()
                        .expect("tileset handles not loaded");
                    let vf4s_opt = world
                        .write_resource::<AssetStorage<VF4sAsset>>()
                        .get_mut(&tileset_handles.vf4s)
                        .and_then(|asset| asset.take())
                        .map(Arc::new);
                    if let Some(vf4s) = vf4s_opt {
                        world.insert::<Arc<VF4s>>(vf4s);
                        node.loaded.set(true);
                    }
                }
                AssetType::VR4s => {
                    let tileset_handles = self
                        .tileset_handles
                        .as_ref()
                        .expect("tileset handles not loaded");
                    let vr4s_opt = world
                        .write_resource::<AssetStorage<VR4sAsset>>()
                        .get_mut(&tileset_handles.vr4s)
                        .and_then(|asset| asset.take())
                        .map(Arc::new);
                    if let Some(vr4s) = vr4s_opt {
                        world.insert::<Arc<VR4s>>(vr4s);
                        node.loaded.set(true);
                    }
                }
                AssetType::VX4s => {
                    let tileset_handles = self
                        .tileset_handles
                        .as_ref()
                        .expect("tileset handles not loaded");
                    let vx4s_opt = world
                        .write_resource::<AssetStorage<VX4sAsset>>()
                        .get_mut(&tileset_handles.vx4s)
                        .and_then(|asset| asset.take())
                        .map(Arc::new);
                    if let Some(vx4s) = vx4s_opt {
                        world.insert::<Arc<VX4s>>(vx4s);
                        node.loaded.set(true);
                    }
                }
                AssetType::WPEs => {
                    let tileset_handles = self
                        .tileset_handles
                        .as_ref()
                        .expect("tileset handles not loaded");
                    let wpes_opts = world
                        .write_resource::<AssetStorage<WPEsAsset>>()
                        .get_mut(&tileset_handles.wpes)
                        .and_then(|asset| asset.take())
                        .map(Arc::new);
                    if let Some(wpes) = wpes_opts {
                        world.insert::<Arc<WPEs>>(wpes);
                        node.loaded.set(true);
                    }
                }
                AssetType::Graphics => {
                    let map_handle = self.map_handle.as_ref().expect("map handle is missing");
                    graphics::create((world, map_handle, &mut self.progress_counter));
                    node.loaded.set(true);
                }
            }
        }

        if Completion::Failed == self.progress_counter.complete() {
            error!(
                "Failed to initialize game due to {} error(s) loading assets",
                self.progress_counter.errors().len()
            );

            Trans::Quit
        } else if self.is_complete() {
            Trans::Push(Box::new(super::GameplayState::default()))
        } else {
            Trans::None
        }
    }
}
