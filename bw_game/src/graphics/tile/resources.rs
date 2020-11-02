use crate::assets::ProgressCounterMutRef;
use amethyst::{
    assets::{AssetStorage, Loader, ProgressCounter},
    ecs::{World, WorldExt},
};
use bw_assets::{
    map::{Map, MapHandle},
    tileset::{
        CV5Format, CV5sAsset, CV5sHandle, VF4Format, VF4sAsset, VF4sHandle, VR4Format, VR4sAsset,
        VR4sHandle, VX4sAsset, VX4sAssetFormat, VX4sHandle, WPEFormat, WPEsAsset, WPEsHandle,
    },
};

#[derive(Clone)]
pub struct TilesetHandles {
    pub vx4s: VX4sHandle,
    pub vr4s: VR4sHandle,
    pub vf4s: VF4sHandle,
    pub cv5s: CV5sHandle,
    pub wpes: WPEsHandle,
}

pub fn load(
    world: &mut World,
    map_handle: MapHandle,
    progress_counter: &mut ProgressCounter,
) -> Option<TilesetHandles> {
    let map_storage = world.read_resource::<AssetStorage<Map>>();
    let loader = world.read_resource::<Loader>();
    let map = map_storage.get(&map_handle)?;
    let tileset_file_name = map.tileset.file_name();

    let mut progress_counter_newtype = ProgressCounterMutRef::new(progress_counter);

    let vx4_handle = loader.load_from(
        format!("tileset\\{}.vx4", tileset_file_name),
        VX4sAssetFormat,
        "bw_assets",
        &mut progress_counter_newtype,
        &world.read_resource::<AssetStorage<VX4sAsset>>(),
    );

    let vr4_handle = loader.load_from(
        format!("tileset\\{}.vr4", tileset_file_name),
        VR4Format,
        "bw_assets",
        &mut progress_counter_newtype,
        &world.read_resource::<AssetStorage<VR4sAsset>>(),
    );

    let vf4_handle = loader.load_from(
        format!("tileset\\{}.vf4", tileset_file_name),
        VF4Format,
        "bw_assets",
        &mut progress_counter_newtype,
        &world.read_resource::<AssetStorage<VF4sAsset>>(),
    );

    let wpe_handle = loader.load_from(
        format!("tileset\\{}.wpe", tileset_file_name),
        WPEFormat,
        "bw_assets",
        &mut progress_counter_newtype,
        &world.read_resource::<AssetStorage<WPEsAsset>>(),
    );

    let cv5_handle = loader.load_from(
        format!("tileset\\{}.cv5", tileset_file_name),
        CV5Format,
        "bw_assets",
        &mut progress_counter_newtype,
        &world.read_resource::<AssetStorage<CV5sAsset>>(),
    );

    Some(TilesetHandles {
        vx4s: vx4_handle,
        vr4s: vr4_handle,
        vf4s: vf4_handle,
        cv5s: cv5_handle,
        wpes: wpe_handle,
    })
}
