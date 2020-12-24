use nom::{
    combinator::{all_consuming, map},
    multi::{count, many0},
    number::complete::le_u16,
    IResult,
};
use std::ops::Index;

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

    fn parse(b: &[u8]) -> IResult<&[u8], VX4> {
        map(le_u16, VX4)(b)
    }

    pub fn index(&self) -> usize {
        return (self.0 >> 1) as usize;
    }
}

#[derive(Debug)]
pub struct VX4s(Vec<Vec<VX4>>);

impl VX4s {
    /// Each megatile has 16 (4x4) minitiles.
    pub const BLOCK_SIZE: usize = 16;

    pub fn parse(b: &[u8]) -> IResult<&[u8], VX4s> {
        all_consuming(map(many0(count(VX4::parse, VX4s::BLOCK_SIZE)), VX4s))(b)
    }

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
