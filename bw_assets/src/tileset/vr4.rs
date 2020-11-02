use amethyst::{
    assets::{Asset, Format, Handle},
    ecs::DenseVecStorage,
};
use nom::{
    combinator::{all_consuming, map},
    multi::{count, many0},
    number::complete::le_u8,
};
use nom::{Finish, IResult};

use rayon::prelude::*;
use std::ops::Index;

use super::VX4;

/// Index to WPE (pixel color)
#[derive(Debug)]
pub struct VR4(u8);

impl From<VR4> for usize {
    fn from(vr4: VR4) -> Self {
        usize::from(&vr4)
    }
}

impl From<&VR4> for usize {
    fn from(vr4: &VR4) -> Self {
        vr4.0 as usize
    }
}

fn parse_vr4(b: &[u8]) -> IResult<&[u8], VR4> {
    map(le_u8, VR4)(b)
}

#[derive(Debug)]
pub struct VR4s(Vec<Vec<VR4>>);

impl VR4s {
    /// Each minitile has a side length of 8 pixels
    pub const MINITILE_SIDE_LENGTH: usize = 8;
    /// 8x8 = 64 pixels
    pub const BLOCK_SIZE: usize = 64;

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> VR4sIterator {
        VR4sIterator(self.0.iter())
    }

    pub fn par_iter(&self) -> rayon::slice::Iter<Vec<VR4>> {
        self.0.par_iter()
    }
}

impl Index<VX4> for VR4s {
    type Output = Vec<VR4>;

    fn index(&self, vx4: VX4) -> &Self::Output {
        &self.index(&vx4)
    }
}

impl Index<&VX4> for VR4s {
    type Output = Vec<VR4>;

    fn index(&self, vx4: &VX4) -> &Self::Output {
        &self.0[vx4.index()]
    }
}

pub struct VR4sIterator<'a>(std::slice::Iter<'a, Vec<VR4>>);

impl<'a> Iterator for VR4sIterator<'a> {
    type Item = &'a Vec<VR4>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

fn parse_vr4s(b: &[u8]) -> IResult<&[u8], VR4s> {
    all_consuming(map(many0(count(parse_vr4, VR4s::BLOCK_SIZE)), VR4s))(b)
}

pub struct VR4sAsset(Option<VR4s>);

impl VR4sAsset {
    pub fn take(&mut self) -> Option<VR4s> {
        self.0.take()
    }
}

pub type VR4sHandle = Handle<VR4sAsset>;

impl Asset for VR4sAsset {
    const NAME: &'static str = "bw_assets:tileset::VR4sAsset";
    type Data = Self;
    type HandleStorage = DenseVecStorage<VR4sHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VR4Format;

impl Format<VR4sAsset> for VR4Format {
    fn name(&self) -> &'static str {
        "VR4Format"
    }

    fn import_simple(&self, b: Vec<u8>) -> amethyst::Result<VR4sAsset> {
        let (_, vr4s) = parse_vr4s(&b).finish().map_err(|err| {
            amethyst::error::format_err!(
                "failed to load vr4 asset: {} at position {}",
                err.code.description(),
                b.len() - err.input.len()
            )
        })?;

        Ok(VR4sAsset(Some(vr4s)))
    }
}
