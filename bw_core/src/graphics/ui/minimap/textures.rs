use super::super::super::UnsafeImageBufferCell;
use super::super::TilesetHandles;
use amethyst::{
    assets::{AssetStorage, Handle, Loader, ProgressCounter},
    core::ecs::World,
    prelude::*,
    renderer::{
        self,
        rendy::{
            resource::ViewKind,
            texture::{MipLevels, TextureBuilder},
        },
        types::TextureData,
        Kind, Texture,
    },
};
use bw_assets::{
    map::{self, Map},
    tileset::{CV5s, VR4s, VX4s, WPEs},
};
use image::ImageBuffer;
use std::cell::UnsafeCell;

pub fn load_minimap(
    params: (&World, &Handle<Map>, &TilesetHandles, &mut ProgressCounter),
) -> Handle<Texture> {
    let (world, map_handle, tileset_handles, progress_counter) = params;

    let map_storage = world.read_resource::<AssetStorage<Map>>();
    let map = map_storage
        .get(&map_handle)
        .expect("map is missing")
        .clone();

    let cv5s_storage = world.read_resource::<AssetStorage<CV5s>>();
    let cv5s = cv5s_storage
        .get(&tileset_handles.cv5s)
        .expect("cv5s is missing")
        .clone();

    let vx4s_storage = world.read_resource::<AssetStorage<VX4s>>();
    let vx4s = vx4s_storage
        .get(&tileset_handles.vx4s)
        .expect("vx4s is missing")
        .clone();

    let vr4s_storage = world.read_resource::<AssetStorage<VR4s>>();
    let vr4s = vr4s_storage
        .get(&tileset_handles.vr4s)
        .expect("vr4s is missing")
        .clone();

    let wpes_storage = world.read_resource::<AssetStorage<WPEs>>();
    let wpes = wpes_storage
        .get(&tileset_handles.wpes)
        .expect("wpes is missing")
        .clone();

    let loader = world.read_resource::<Loader>();
    let minimap_texture = loader.load_from_data_async(
        move || {
            let (pixels, (px_width, px_height)) =
                create_minimap_texture((map, cv5s, vx4s, vr4s, wpes));
            let texture_builder = TextureBuilder::new()
                .with_kind(Kind::D2(px_width, px_height, 1, 1))
                .with_view_kind(ViewKind::D2)
                .with_data_width(px_width)
                .with_data_height(px_height)
                .with_mip_levels(MipLevels::GenerateAuto)
                .with_raw_data(pixels, renderer::Format::Rgb8Srgb);
            TextureData::from(texture_builder)
        },
        progress_counter,
        &world.read_resource::<AssetStorage<Texture>>(),
    );

    minimap_texture
}

fn create_minimap_texture(assets: (Map, CV5s, VX4s, VR4s, WPEs)) -> (Vec<u8>, (u32, u32)) {
    use rayon::prelude::*;

    let (map, cv5s, vx4s, vr4s, wpes) = assets;

    let map_width = map.dimensions.width as u32;
    let map_height = map.dimensions.height as u32;

    let map_px_width = map_width * map::MEGATILE_PX_SIDE_LEN;
    let map_px_height = map_height * map::MEGATILE_PX_SIDE_LEN;

    let img_buffer = UnsafeImageBufferCell(UnsafeCell::new(
        ImageBuffer::<image::Rgb<u8>, Vec<u8>>::new(map_px_width, map_px_height),
    ));
    let px_size = map_px_width * map_px_height;

    (0..px_size).into_par_iter().for_each(|i| {
        let xi = i % map_px_width / map::MEGATILE_PX_SIDE_LEN;
        let yi = i / map_px_width / map::MEGATILE_PX_SIDE_LEN;

        let megatile = &map.megatiles[(xi + yi * map_width) as usize];
        let cv5 = &cv5s[megatile];
        let minitiles = &vx4s[cv5];

        let xj = i % map_px_width % map::MEGATILE_PX_SIDE_LEN / map::MINITILE_PX_SIDE_LEN;
        let yj = i / map_px_width % map::MEGATILE_PX_SIDE_LEN / map::MINITILE_PX_SIDE_LEN;

        let minitile = &minitiles[(xj + yj * map::MEGATILE_SIDE_LEN) as usize];
        let vr4 = &vr4s[minitile];

        let xk = i % map_px_width % map::MEGATILE_PX_SIDE_LEN % map::MINITILE_PX_SIDE_LEN;
        let yk = i / map_px_width % map::MEGATILE_PX_SIDE_LEN % map::MINITILE_PX_SIDE_LEN;

        let color = if minitile.is_horizontally_flipped() {
            &wpes[&vr4
                [((map::MINITILE_PX_SIDE_LEN - 1 - xk) + yk * map::MINITILE_PX_SIDE_LEN) as usize]]
        } else {
            &wpes[&vr4[(xk + yk * map::MINITILE_PX_SIDE_LEN) as usize]]
        };

        unsafe {
            (*img_buffer.get()).put_pixel(
                (xi * map::MEGATILE_PX_SIDE_LEN + xj * map::MINITILE_PX_SIDE_LEN + xk) as u32,
                (yi * map::MEGATILE_PX_SIDE_LEN + yj * map::MINITILE_PX_SIDE_LEN + yk) as u32,
                image::Rgb(color.rgb()),
            );
        }
    });

    (
        img_buffer.into_inner().into_raw(),
        (map_px_width, map_px_height),
    )
}
