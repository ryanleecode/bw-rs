use bevy::tasks::TaskPool;
use image::ImageBuffer;
use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use crate::loader::{VR4s, WPEs};

pub type RGBAImageBuffer = ImageBuffer<image::Rgba<u8>, Vec<u8>>;

const TILEMAP_TEXTURE_SIDE_LENGTH: usize = 2048;
pub const TILE_PADDING: usize = 1;
pub const MINITILE_PX_SIDE_LEN: u32 = 8;
const MINITILE_SIDE_LENGTH_WITH_PADDING: usize =
    (MINITILE_PX_SIDE_LEN + (TILE_PADDING as u32) * 2) as usize;
const COLUMNS: usize = TILEMAP_TEXTURE_SIDE_LENGTH / MINITILE_SIDE_LENGTH_WITH_PADDING;

struct UnsafeImageBufferCell(UnsafeCell<RGBAImageBuffer>);

unsafe impl Sync for UnsafeImageBufferCell {}

impl Deref for UnsafeImageBufferCell {
    type Target = UnsafeCell<RGBAImageBuffer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for UnsafeImageBufferCell {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl UnsafeImageBufferCell {
    pub fn into_inner(self) -> RGBAImageBuffer {
        self.0.into_inner()
    }
}

pub(crate) fn create_tile_map_texture_buffer(
    task_pool: TaskPool,
    vr4s: &VR4s,
    wpes: &WPEs,
) -> RGBAImageBuffer {
    let img_buffer = UnsafeImageBufferCell(UnsafeCell::new(RGBAImageBuffer::new(
        TILEMAP_TEXTURE_SIDE_LENGTH as u32,
        TILEMAP_TEXTURE_SIDE_LENGTH as u32,
    )));

    let img_buffer_ref = &img_buffer;

    task_pool.scope(|scope| {
        vr4s.chunks(COLUMNS)
            .enumerate()
            .for_each(|(row, row_minitiles)| {
                scope.spawn(async move {
                    row_minitiles
                        .iter()
                        .enumerate()
                        .for_each(|(column, minitile)| {
                            let xi = column * MINITILE_SIDE_LENGTH_WITH_PADDING;
                            let yi = row * MINITILE_SIDE_LENGTH_WITH_PADDING;

                            (0..(MINITILE_PX_SIDE_LEN * MINITILE_PX_SIDE_LEN) as usize).for_each(
                                |j| {
                                    let xj = j % MINITILE_PX_SIDE_LEN as usize;
                                    let yj = j / MINITILE_PX_SIDE_LEN as usize;

                                    let vr4 = &minitile[xj + yj * MINITILE_PX_SIDE_LEN as usize];
                                    let color = &wpes[vr4];

                                    unsafe {
                                        if xj == 0 {
                                            (*img_buffer_ref.get()).put_pixel(
                                                (xi + xj) as u32,
                                                (yi + TILE_PADDING + yj) as u32,
                                                color.into(),
                                            )
                                        }

                                        if xj == (MINITILE_PX_SIDE_LEN - 1) as usize {
                                            (*img_buffer_ref.get()).put_pixel(
                                                (xi + (TILE_PADDING * 2) + xj) as u32,
                                                (yi + TILE_PADDING + yj) as u32,
                                                color.into(),
                                            )
                                        }

                                        if yj == 0 {
                                            (*img_buffer_ref.get()).put_pixel(
                                                (xi + TILE_PADDING + xj) as u32,
                                                (yi + yj) as u32,
                                                color.into(),
                                            )
                                        }

                                        if yj == (MINITILE_PX_SIDE_LEN - 1) as usize {
                                            (*img_buffer_ref.get()).put_pixel(
                                                (xi + TILE_PADDING + xj) as u32,
                                                (yi + (TILE_PADDING * 2) + yj) as u32,
                                                color.into(),
                                            )
                                        }

                                        (*img_buffer_ref.get()).put_pixel(
                                            (xi + TILE_PADDING + xj) as u32,
                                            (yi + TILE_PADDING + yj) as u32,
                                            color.into(),
                                        )
                                    }
                                },
                            );
                        });
                });
            });
    });

    img_buffer.into_inner()
}
