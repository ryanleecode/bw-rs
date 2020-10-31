use amethyst::{
    assets::AssetStorage,
    core::{
        math::{Point3, Vector3},
        Transform,
    },
    ecs::{Entities, Join, Read, ReadExpect, ReadStorage, World},
    renderer::{ActiveCamera, Camera},
    tiles::{self, CoordinateEncoder, DrawTiles2DBounds, Region, Tile, TileMap},
    window::ScreenDimensions,
};
use bw_assets::{
    map::{Map, MapHandle},
    tileset::{CV5s, VR4s, VX4s},
};
use resources::TilesetHandles;

pub mod map;
pub mod resources;

#[derive(Debug, Default, Clone)]
pub struct AmethystTileBridge;

fn megatile_index(coords: &Point3<u32>, map: &Map) -> usize {
    (coords.x / bw_assets::map::MEGATILE_SIDE_LEN) as usize
        + (coords.y / bw_assets::map::MEGATILE_SIDE_LEN) as usize * map.dimensions.width as usize
}

fn minitile_index(coords: &Point3<u32>) -> usize {
    (coords.x % bw_assets::map::MEGATILE_SIDE_LEN) as usize
        + (coords.y % bw_assets::map::MEGATILE_SIDE_LEN) as usize
            * bw_assets::map::MEGATILE_SIDE_LEN as usize
}

impl Tile for AmethystTileBridge {
    fn sprite(&self, coords: Point3<u32>, world: &World) -> Option<usize> {
        let map_handle = world.try_fetch::<MapHandle>()?;
        let map_storage = world.try_fetch::<AssetStorage<Map>>()?;
        let map = map_storage.get(&map_handle)?;

        let megatile = &map.megatiles[megatile_index(&coords, &map)];

        let tileset_handles = world.try_fetch::<TilesetHandles>()?;
        let cv5s_storage = world.try_fetch::<AssetStorage<CV5s>>()?;
        let cv5s = cv5s_storage.get(&tileset_handles.cv5s)?;

        let megatile_reference = &cv5s[megatile][megatile];

        let vx4s_storage = world.try_fetch::<AssetStorage<VX4s>>()?;
        let vx4s = vx4s_storage.get(&tileset_handles.vx4s)?;

        let minitiles = &vx4s[megatile_reference];
        let minitile = &minitiles[minitile_index(&coords)];

        let vr4s_storage = world.try_fetch::<AssetStorage<VR4s>>()?;
        let vr4s = vr4s_storage.get(&tileset_handles.vr4s)?;

        if minitile.is_horizontally_flipped() {
            Some(minitile.index() + vr4s.len())
        } else {
            Some(minitile.index())
        }
    }
}

#[derive(Debug)]
pub struct ScreenBounds;

impl DrawTiles2DBounds for ScreenBounds {
    fn bounds<T: Tile, E: CoordinateEncoder>(map: &TileMap<T, E>, world: &World) -> Region {
        let (screen_dimensions, active_camera, cameras, transforms, entities): (
            ReadExpect<ScreenDimensions>,
            Read<ActiveCamera>,
            ReadStorage<Camera>,
            ReadStorage<Transform>,
            Entities,
        ) = world.system_data();

        let (w, h) = (screen_dimensions.width(), screen_dimensions.height());
        let mut camera_join = (&cameras, &transforms).join();
        if let Some((camera, camera_transform)) = active_camera
            .entity
            .and_then(|a| camera_join.get(a, &entities))
            .or_else(|| camera_join.next())
        {
            let top_left = camera.screen_to_world_point(
                Point3::origin(),
                screen_dimensions.diagonal(),
                camera_transform,
            );
            let bottom_right = camera.screen_to_world_point(
                Point3::new(w, h, 0.0),
                screen_dimensions.diagonal(),
                camera_transform,
            );

            use tiles::Map;

            let min = map
                .to_tile(&Vector3::new(top_left.x, top_left.y, 1.0), None)
                .unwrap_or_else(|err| {
                    let p = err.point_dimensions;
                    let m = err.max_dimensions;
                    Point3::new(
                        ((p.x - 1).max(0) as u32).min(m.x),
                        ((p.y - 1).max(0) as u32).min(m.y),
                        0,
                    )
                });
            let max = map
                .to_tile(&Vector3::new(bottom_right.x, bottom_right.y, 1.0), None)
                .unwrap_or_else(|err| {
                    let p = err.point_dimensions;
                    let m = err.max_dimensions;
                    Point3::new(
                        (p.x.max(0) as u32).min(m.x),
                        (p.y.max(0) as u32).min(m.y),
                        0,
                    )
                });

            Region::new(min, max)
        } else {
            Region::empty()
        }
    }
}
