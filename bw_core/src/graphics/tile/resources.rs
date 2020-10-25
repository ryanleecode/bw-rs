use crate::assets::ProgressCounterMutRef;
use amethyst::{
    assets::{AssetStorage, Loader, ProgressCounter},
    ecs::{World, WorldExt},
};
use bw_assets::{
    map::{Map, MapHandle},
    tileset::{
        CV5Format, CV5s, CV5sHandle, VF4Format, VF4s, VF4sHandle, VR4Format, VR4s, VR4sHandle,
        VX4Format, VX4s, VX4sHandle, WPEFormat, WPEs, WPEsHandle,
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
) -> TilesetHandles {
    let tileset_handles = {
        let map_storage = world.read_resource::<AssetStorage<Map>>();
        let loader = world.read_resource::<Loader>();
        let map = map_storage.get(&map_handle).expect("map should be loaded");
        let tileset_file_name = map.tileset.file_name();

        let mut progress_counter_newtype = ProgressCounterMutRef::new(progress_counter);

        let vx4_handle = loader.load_from(
            format!("tileset\\{}.vx4", tileset_file_name),
            VX4Format,
            "bw_assets",
            &mut progress_counter_newtype,
            &world.read_resource::<AssetStorage<VX4s>>(),
        );

        let vr4_handle = loader.load_from(
            format!("tileset\\{}.vr4", tileset_file_name),
            VR4Format,
            "bw_assets",
            &mut progress_counter_newtype,
            &world.read_resource::<AssetStorage<VR4s>>(),
        );

        let vf4_handle = loader.load_from(
            format!("tileset\\{}.vf4", tileset_file_name),
            VF4Format,
            "bw_assets",
            &mut progress_counter_newtype,
            &world.read_resource::<AssetStorage<VF4s>>(),
        );

        let wpe_handle = loader.load_from(
            format!("tileset\\{}.wpe", tileset_file_name),
            WPEFormat,
            "bw_assets",
            &mut progress_counter_newtype,
            &world.read_resource::<AssetStorage<WPEs>>(),
        );

        let cv5_handle = loader.load_from(
            format!("tileset\\{}.cv5", tileset_file_name),
            CV5Format,
            "bw_assets",
            &mut progress_counter_newtype,
            &world.read_resource::<AssetStorage<CV5s>>(),
        );

        TilesetHandles {
            vx4s: vx4_handle,
            vr4s: vr4_handle,
            vf4s: vf4_handle,
            cv5s: cv5_handle,
            wpes: wpe_handle,
        }
    };

    world.insert(tileset_handles.clone());

    tileset_handles
}
