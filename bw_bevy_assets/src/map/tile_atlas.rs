use bevy::{core::Byteable, tasks::TaskPool};
use bw_assets::map::{CV5s, Dimensions, Map, VX4s};

use bevy::{prelude::*, reflect::TypeUuid, render::renderer::RenderResources};

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Tile {
    pub sprite_index: u32,
    pub is_horizontally_flipped: bool,
}

unsafe impl Byteable for Tile {}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Rect {
    /// The beginning point of the rect
    pub min: Vec2,
    /// The ending point of the rect
    pub max: Vec2,
}

impl Rect {
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }
}

unsafe impl Byteable for Rect {}

#[derive(Debug)]
pub struct PaddingSpecification {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Default for PaddingSpecification {
    fn default() -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
        }
    }
}

impl PaddingSpecification {
    pub fn uniform(padding: u32) -> PaddingSpecification {
        Self {
            left: padding as f32,
            top: padding as f32,
            right: padding as f32,
            bottom: padding as f32,
        }
    }
}

#[derive(Debug, RenderResources, TypeUuid)]
#[uuid = "8a56f23c-ae6d-489a-bb16-d3597032d61d"]
pub struct TileAtlas {
    pub texture: Handle<Texture>,
    pub size: Vec2,
    #[render_resources(buffer)]
    pub textures: Vec<Rect>,
    #[render_resources(buffer)]
    pub tiles: Vec<Tile>,
}

impl TileAtlas {
    pub fn from_grid_with_padding(
        texture: Handle<Texture>,
        tile_size: Vec2,
        texture_dimensions: Vec2,
        padding: PaddingSpecification,
    ) -> Self {
        let padding: PaddingSpecification = padding.into();
        let x_padding = padding.left + padding.right;
        let y_padding = padding.top + padding.bottom;
        let rows = (texture_dimensions.x / (tile_size.x + x_padding)) as i32;
        let columns = (texture_dimensions.y / (tile_size.y + y_padding)) as i32;

        let mut sprites = Vec::with_capacity((rows * columns) as usize);

        for y in 0..rows {
            for x in 0..columns {
                let rect_min = Vec2::new(
                    (tile_size.x + x_padding) * x as f32 + padding.left,
                    (tile_size.y + y_padding) * y as f32 + padding.top,
                );

                sprites.push(Rect {
                    min: rect_min,
                    max: Vec2::new(rect_min.x + tile_size.x, rect_min.y + tile_size.y),
                })
            }
        }

        TileAtlas {
            size: texture_dimensions,
            textures: sprites,
            texture,
            tiles: Vec::new(),
        }
    }

    pub fn insert_tiles(&mut self, tiles: Vec<Tile>) {
        self.tiles = tiles;
    }
}

const MEGATILE_SIDE_LEN: usize = bw_assets::map::MEGATILE_SIDE_LEN as usize;

fn megatile_index(x: usize, y: usize, dimensions: &Dimensions) -> usize {
    x / MEGATILE_SIDE_LEN + (y / MEGATILE_SIDE_LEN) * usize::from(dimensions.width)
}

fn minitile_index(x: usize, y: usize) -> usize {
    (x % MEGATILE_SIDE_LEN) as usize + (y % MEGATILE_SIDE_LEN) as usize * MEGATILE_SIDE_LEN as usize
}

pub(crate) fn create_minitiles(
    task_pool: TaskPool,
    map: &Map,
    cv5s: &CV5s,
    vx4s: &VX4s,
) -> Vec<Tile> {
    let dimensions = &map.dimensions;
    let megatiles = &map.megatiles;

    let width = usize::from(dimensions.width) * MEGATILE_SIDE_LEN as usize;
    let height = usize::from(dimensions.height) * MEGATILE_SIDE_LEN as usize;

    let minitiles = task_pool
        .scope(|scope| {
            for y in 0..height {
                scope.spawn(async move {
                    (0..width)
                        .map(|x| {
                            let megatile = &megatiles[megatile_index(x, y, dimensions)];

                            let megatile_reference = &cv5s[megatile][megatile];

                            let vx4 = &vx4s[megatile_reference];
                            let minitile = &vx4[minitile_index(x, y)];

                            let sprite_index = minitile.index();

                            Tile {
                                sprite_index: sprite_index as u32,
                                is_horizontally_flipped: minitile.is_horizontally_flipped(),
                            }
                        })
                        .collect::<Vec<_>>()
                });
            }
        })
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();


    minitiles
}
