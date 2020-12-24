use nom::{
    combinator::{all_consuming, map},
    multi::{count, many0},
    number::complete::le_u8,
    IResult,
};
use std::ops::Deref;

use crate::make_pointer;

use std::ops::Index;

use super::VX4;

// Index to WPE (pixel color)
make_pointer!(VR4, u8);

impl VR4 {
    fn parse(b: &[u8]) -> IResult<&[u8], VR4> {
        map(le_u8, VR4)(b)
    }
}

#[derive(Debug)]
pub struct VR4s(Vec<Vec<VR4>>);

impl VR4s {
    /// 8x8 = 64 pixels
    pub const BLOCK_SIZE: usize = 64;
    /// Each minitile has a side length of 8 pixels
    pub const MINITILE_SIDE_LENGTH: usize = 8;

    pub fn parse(b: &[u8]) -> IResult<&[u8], VR4s> {
        all_consuming(map(many0(count(VR4::parse, VR4s::BLOCK_SIZE)), VR4s))(b)
    }
}

impl Deref for VR4s {
    type Target = Vec<Vec<VR4>>;

    fn deref(&self) -> &Self::Target {
        &self.0
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
