use std::ops::Index;

use amethyst::{
    assets::Format,
    assets::{Asset, Handle},
    ecs::DenseVecStorage,
};
use nom::{
    combinator::{all_consuming, map},
    multi::count,
    multi::many0,
    number::complete::le_u16,
    Finish, IResult,
};

use super::MinitileReference;

/// Mini-tile image pointer. Referenced by CV5.
///
/// Bit 0 will indicate if the tile is flipped, and the 7 high bits is the
/// index to the VR4 asset.
#[derive(Debug)]
pub struct VX4(u16);

impl VX4 {
    pub fn is_horizontally_flipped(&self) -> bool {
        return self.0 & 1 == 1;
    }

    pub fn index(&self) -> usize {
        return (self.0 >> 1) as usize;
    }
}

fn parse_vx4(b: &[u8]) -> IResult<&[u8], VX4> {
    map(le_u16, VX4)(b)
}

#[derive(Debug)]
pub struct VX4s(Vec<Vec<VX4>>);

impl VX4s {
    /// Each megatile has 16 (4x4) minitiles.
    pub const BLOCK_SIZE: usize = 16;

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<MinitileReference> for VX4s {
    type Output = Vec<VX4>;

    fn index(&self, megatile_reference: MinitileReference) -> &Self::Output {
        &self.index(&megatile_reference)
    }
}

impl Index<&MinitileReference> for VX4s {
    type Output = Vec<VX4>;

    fn index(&self, megatile_reference: &MinitileReference) -> &Self::Output {
        &self.0[usize::from(megatile_reference)]
    }
}

fn parse_vx4s(b: &[u8]) -> IResult<&[u8], VX4s> {
    all_consuming(map(many0(count(parse_vx4, VX4s::BLOCK_SIZE)), VX4s))(b)
}

/// This asset is a singleton so we will load it and then take it out of the option.
/// Amethyst will then drop the handle.
pub struct VX4sAsset(Option<VX4s>);

impl VX4sAsset {
    pub fn take(&mut self) -> Option<VX4s> {
        self.0.take()
    }
}

pub type VX4sHandle = Handle<VX4sAsset>;

impl Asset for VX4sAsset {
    const NAME: &'static str = "bw_assets::tileset::VX4sAsset";
    type Data = Self;
    type HandleStorage = DenseVecStorage<VX4sHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VX4sAssetFormat;

impl Format<VX4sAsset> for VX4sAssetFormat {
    fn name(&self) -> &'static str {
        "VX4Format"
    }

    fn import_simple(&self, b: Vec<u8>) -> amethyst::Result<VX4sAsset> {
        let (_, vx4s) = parse_vx4s(&b).finish().map_err(|err| {
            amethyst::error::format_err!(
                "failed to load vx4 asset: {} at position {}",
                err.code.description(),
                b.len() - err.input.len()
            )
        })?;

        Ok(VX4sAsset(Some(vx4s)))
    }
}
