use amethyst::{
    assets::{Asset, Format, Handle},
    ecs::DenseVecStorage,
};
use nom::{
    combinator::{all_consuming, map},
    multi::{count, many0},
    number::complete::le_u16,
};
use nom::{Finish, IResult};
use std::ops::Index;

use super::MinitileReference;

/// MiniTile graphic references for each MegaTile. Referenced by CV5.
#[derive(Debug)]
pub struct VF4(u16);

#[derive(Debug)]
pub struct VF4s(Vec<Vec<VF4>>);

impl Index<MinitileReference> for VF4s {
    type Output = Vec<VF4>;

    fn index(&self, megatile_reference: MinitileReference) -> &Self::Output {
        &self.index(&megatile_reference)
    }
}

impl Index<&MinitileReference> for VF4s {
    type Output = Vec<VF4>;

    fn index(&self, megatile_reference: &MinitileReference) -> &Self::Output {
        &self.0[usize::from(megatile_reference)]
    }
}

impl VF4 {
    // http://www.staredit.net/wiki/index.php?title=Terrain_Format#VF4

    const WALKABLE: u16 = 0x0001;
    const MID: u16 = 0x0002;
    const HIGH: u16 = 0x0004;
    const LOW: u16 = 0x0004 | 0x0002;
    const BLOCKS_VIEW: u16 = 0x0008;
    const RAMP: u16 = 0x0010;

    pub fn is_walkable(&self) -> bool {
        return self.0 & VF4::WALKABLE == VF4::WALKABLE;
    }

    pub fn is_elevation_mid(&self) -> bool {
        return self.0 & VF4::MID == VF4::MID;
    }

    pub fn is_elevation_high(&self) -> bool {
        return self.0 & VF4::HIGH == VF4::HIGH;
    }

    pub fn is_elevation_low(&self) -> bool {
        return self.0 & VF4::LOW == VF4::LOW;
    }

    pub fn blocks_view(&self) -> bool {
        return self.0 & VF4::BLOCKS_VIEW == VF4::BLOCKS_VIEW;
    }

    pub fn is_ramp(&self) -> bool {
        return self.0 & VF4::RAMP == VF4::RAMP;
    }
}

impl VF4s {
    /// Each megatile has 16 (4x4) minitiles.
    const BLOCK_SIZE: usize = 16;
}

fn parse_vf4s(b: &[u8]) -> IResult<&[u8], VF4s> {
    all_consuming(map(many0(count(map(le_u16, VF4), VF4s::BLOCK_SIZE)), VF4s))(b)
}

pub struct VF4sAsset(Option<VF4s>);

impl VF4sAsset {
    pub fn take(&mut self) -> Option<VF4s> {
        self.0.take()
    }
}

pub type VF4sHandle = Handle<VF4sAsset>;

impl Asset for VF4sAsset {
    const NAME: &'static str = "bw_assets::tileset::VF4sAsset";
    type Data = Self;
    type HandleStorage = DenseVecStorage<VF4sHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VF4Format;

impl Format<VF4sAsset> for VF4Format {
    fn name(&self) -> &'static str {
        "VF4Format"
    }

    fn import_simple(&self, b: Vec<u8>) -> amethyst::Result<VF4sAsset> {
        let (_, vf4s) = parse_vf4s(&b).finish().map_err(|err| {
            amethyst::error::format_err!(
                "failed to load vf4 asset: {} at {}",
                err.code.description(),
                b.len() - err.input.len()
            )
        })?;

        Ok(VF4sAsset(Some(vf4s)))
    }
}
