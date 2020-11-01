use super::super::UnsafeImageBufferCell;
use super::AmethystTileBridge;
use amethyst::{
    assets::{AssetStorage, Handle, Loader, ProgressCounter},
    core::{
        ecs::World,
        {math::Vector3, Transform},
    },
    prelude::*,
    renderer::{
        self,
        rendy::{
            hal::image::{Filter, PackedColor, SamplerInfo, ViewKind, WrapMode},
            {resource::Anisotropic, texture::TextureBuilder},
        },
        types::TextureData,
        Kind, Sprite, SpriteSheet, Texture,
    },
    tiles::TileMap,
};
use bw_assets::{
    map::{self, Map},
    tileset::{VR4s, WPEs},
};
use image::ImageBuffer;
use std::{cell::UnsafeCell, sync::Arc};

const TILEMAP_TEXTURE_SIDE_LENGTH: usize = 2048;
const PADDING: usize = 1;
const MINITILE_SIDE_LENGTH_WITH_PADDING: usize =
    (map::MINITILE_PX_SIDE_LEN + (PADDING as u32) * 2) as usize;

pub fn create(params: (&mut World, &Handle<Map>, &mut ProgressCounter)) {
    let (world, map_handle, progress_counter) = params;

    let tilemap_texture_handle = load_map_texture(world, progress_counter);
    let sprite_sheet_handle =
        load_sprite_sheet_handle(world, progress_counter, tilemap_texture_handle);

    let tilemap = {
        let map_storage = world.read_resource::<AssetStorage<Map>>();

        let map = map_storage.get(&map_handle).expect("map is missing");
        let width = map.dimensions.width as u32 * map::MEGATILE_SIDE_LEN;
        let height = map.dimensions.height as u32 * map::MEGATILE_SIDE_LEN;

        TileMap::<AmethystTileBridge>::new(
            Vector3::new(width, height, 1),
            Vector3::new(map::MINITILE_PX_SIDE_LEN, map::MINITILE_PX_SIDE_LEN, 1),
            Some(sprite_sheet_handle.clone()),
        )
    };

    world
        .create_entity()
        .with(tilemap)
        .with(Transform::default())
        .build();
}

fn load_sprite_sheet_handle(
    world: &World,
    progress_counter: &mut ProgressCounter,
    tilemap_texture_handle: Handle<Texture>,
) -> Handle<SpriteSheet> {
    let loader = world.read_resource::<Loader>();

    let vr4s = (*world.try_fetch::<Arc<VR4s>>().expect("vr4s is missing")).clone();

    loader.load_from_data_async(
        move || create_tilemap_sprite_sheet(tilemap_texture_handle, vr4s),
        progress_counter,
        &world.read_resource::<AssetStorage<SpriteSheet>>(),
    )
}

fn load_map_texture(world: &World, progress_counter: &mut ProgressCounter) -> Handle<Texture> {
    let loader = world.read_resource::<Loader>();

    let vr4s = (*world.try_fetch::<Arc<VR4s>>().expect("vr4s is missing")).clone();
    let wpes = (*world.try_fetch::<Arc<WPEs>>().expect("wpes is missing")).clone();

    loader.load_from_data_async(
        move || {
            let pixels = create_map_texture_pixels(vr4s, wpes);

            // https://stackoverflow.com/questions/57691913/how-to-load-a-texture-from-memory-in-amethyst-engine
            let texture_builder = TextureBuilder::new()
                .with_kind(Kind::D2(
                    TILEMAP_TEXTURE_SIDE_LENGTH as u32,
                    TILEMAP_TEXTURE_SIDE_LENGTH as u32,
                    1,
                    1,
                ))
                .with_view_kind(ViewKind::D2)
                .with_data_width(TILEMAP_TEXTURE_SIDE_LENGTH as u32)
                .with_data_height(TILEMAP_TEXTURE_SIDE_LENGTH as u32)
                .with_sampler_info(SamplerInfo {
                    min_filter: Filter::Nearest,
                    mag_filter: Filter::Nearest,
                    mip_filter: Filter::Nearest,
                    wrap_mode: (WrapMode::Clamp, WrapMode::Clamp, WrapMode::Clamp),
                    lod_bias: 0.0.into(),
                    lod_range: std::ops::Range {
                        start: 0.0.into(),
                        end: 1000.0.into(),
                    },
                    comparison: None,
                    border: PackedColor(0),
                    normalized: true,
                    anisotropic: Anisotropic::Off,
                })
                .with_raw_data(pixels, renderer::Format::Rgb8Srgb);

            TextureData::from(texture_builder)
        },
        progress_counter,
        &world.read_resource::<AssetStorage<Texture>>(),
    )
}

fn create_tilemap_sprite_sheet(texture: Handle<Texture>, vr4s: Arc<VR4s>) -> SpriteSheet {
    use rayon::prelude::*;

    let sprite_count = vr4s.len();
    let mut sprites: Vec<Sprite> = (0..sprite_count)
        .into_par_iter()
        .map(|i| {
            let columns = TILEMAP_TEXTURE_SIDE_LENGTH / MINITILE_SIDE_LENGTH_WITH_PADDING;
            let row = i / columns;
            let column = i - (row * columns);
            let x = column * MINITILE_SIDE_LENGTH_WITH_PADDING;
            let y = row * MINITILE_SIDE_LENGTH_WITH_PADDING;
            let sprite = Sprite::from_pixel_values(
                TILEMAP_TEXTURE_SIDE_LENGTH as u32,
                TILEMAP_TEXTURE_SIDE_LENGTH as u32,
                map::MINITILE_PX_SIDE_LEN,
                map::MINITILE_PX_SIDE_LEN,
                (x + PADDING) as u32,
                (y + PADDING) as u32,
                [0.0; 2],
                false,
                false,
            );
            sprite
        })
        .collect();

    let flipped_sprites: Vec<Sprite> = (0..sprite_count)
        .into_par_iter()
        .map(|i| {
            let columns = TILEMAP_TEXTURE_SIDE_LENGTH / MINITILE_SIDE_LENGTH_WITH_PADDING;
            let row = i / columns;
            let column = i - (row * columns);
            let x = column * MINITILE_SIDE_LENGTH_WITH_PADDING;
            let y = row * MINITILE_SIDE_LENGTH_WITH_PADDING;
            let sprite = Sprite::from_pixel_values(
                TILEMAP_TEXTURE_SIDE_LENGTH as u32,
                TILEMAP_TEXTURE_SIDE_LENGTH as u32,
                map::MINITILE_PX_SIDE_LEN,
                map::MINITILE_PX_SIDE_LEN,
                (x + PADDING) as u32,
                (y + PADDING) as u32,
                [0.0; 2],
                true,
                false,
            );
            sprite
        })
        .collect();

    sprites.extend(flipped_sprites);

    SpriteSheet { texture, sprites }
}

fn create_map_texture_pixels(vr4s: Arc<VR4s>, wpes: Arc<WPEs>) -> Vec<u8> {
    use rayon::prelude::*;

    let img_buffer = UnsafeImageBufferCell(UnsafeCell::new(
        ImageBuffer::<image::Rgb<u8>, Vec<u8>>::new(
            TILEMAP_TEXTURE_SIDE_LENGTH as u32,
            TILEMAP_TEXTURE_SIDE_LENGTH as u32,
        ),
    ));

    vr4s.par_iter().enumerate().for_each(|(i, minitile)| {
        let columns = TILEMAP_TEXTURE_SIDE_LENGTH / MINITILE_SIDE_LENGTH_WITH_PADDING;
        let row = i / columns;
        let column = i - (row * columns);
        let xi = column * MINITILE_SIDE_LENGTH_WITH_PADDING;
        let yi = row * MINITILE_SIDE_LENGTH_WITH_PADDING;

        (0..(map::MINITILE_PX_SIDE_LEN as usize))
            .into_par_iter()
            .for_each(|xj| {
                (0..(map::MINITILE_PX_SIDE_LEN as usize))
                    .into_par_iter()
                    .for_each(|yj| {
                        let vr4 = &minitile[xj + yj * map::MINITILE_PX_SIDE_LEN as usize];
                        let color = &wpes[vr4];

                        unsafe {
                            if xj == 0 {
                                (*img_buffer.get()).put_pixel(
                                    (xi + xj) as u32,
                                    (yi + PADDING + yj) as u32,
                                    image::Rgb(color.rgb()),
                                )
                            }

                            if xj == (map::MINITILE_PX_SIDE_LEN - 1) as usize {
                                (*img_buffer.get()).put_pixel(
                                    (xi + (PADDING * 2) + xj) as u32,
                                    (yi + PADDING + yj) as u32,
                                    image::Rgb(color.rgb()),
                                )
                            }

                            if yj == 0 {
                                (*img_buffer.get()).put_pixel(
                                    (xi + PADDING + xj) as u32,
                                    (yi + yj) as u32,
                                    image::Rgb(color.rgb()),
                                )
                            }

                            if yj == (map::MINITILE_PX_SIDE_LEN - 1) as usize {
                                (*img_buffer.get()).put_pixel(
                                    (xi + PADDING + xj) as u32,
                                    (yi + (PADDING * 2) + yj) as u32,
                                    image::Rgb(color.rgb()),
                                )
                            }
                            (*img_buffer.get()).put_pixel(
                                (xi + PADDING + xj) as u32,
                                (yi + PADDING + yj) as u32,
                                image::Rgb(color.rgb()),
                            )
                        }
                    })
            });
    });

    img_buffer.into_inner().into_raw()
}
