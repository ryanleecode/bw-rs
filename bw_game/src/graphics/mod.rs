use amethyst::{
    assets::{Handle, ProgressCounter},
    core::ecs::World,
};
use bw_assets::map::Map;
use image::ImageBuffer;
use std::cell::UnsafeCell;

pub mod camera;
pub mod tile;
pub mod ui;

pub fn create(params: (&mut World, &Handle<Map>, &mut ProgressCounter)) {
    let (world, map_handle, progress_counter) = params;

    tile::map::create((world, map_handle, progress_counter));
    ui::create((world, map_handle, progress_counter));
}

pub type RGBImageBuffer = ImageBuffer<image::Rgb<u8>, Vec<u8>>;

pub struct UnsafeImageBufferCell(UnsafeCell<RGBImageBuffer>);

unsafe impl Sync for UnsafeImageBufferCell {}

impl UnsafeImageBufferCell {
    pub fn get(&self) -> *mut RGBImageBuffer {
        self.0.get()
    }

    pub fn into_inner(self) -> RGBImageBuffer {
        self.0.into_inner()
    }
}
