use super::SpritePointer;
use nom::{
    bytes::complete::take,
    combinator::{all_consuming, map},
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u16, le_u32},
    IResult, Parser,
};
use std::ops::Index;

use crate::{count_total, make_dat, make_pointer};

#[derive(Debug)]
pub struct Flingy {
    pub sprite: SpritePointer,
    pub top_speed: u32,
    pub acceleration: u16,
    pub halt_distance: u32,
    pub turn_radius: u8,
    pub move_control: u8,
}

make_pointer!(FlingyPointer, u32);

make_dat!(FlingyDat, Flingy, FlingyPointer);

const BLOCK_SIZE: usize = 209;
count_total!(BLOCK_SIZE);

impl FlingyDat {
    pub fn parse(b: &[u8]) -> IResult<&[u8], FlingyDat> {
        let (remaining, sprite_col) = count_total(map(le_u16, SpritePointer))(b)?;
        let (remaining, top_speed_col) = count_total(le_u32)(remaining)?;
        let (remaining, acceleration_col) = count_total(le_u16)(remaining)?;
        let (remaining, halt_distance_col) = count_total(le_u32)(remaining)?;
        let (remaining, turn_radius_col) = count_total(le_u8)(remaining)?;

        // unknown block
        let (remaining, _) = count_total(le_u8)(remaining)?;

        let (remaining, move_control_col) = count_total(le_u8)(remaining)?;

        let (remaining, _) = all_consuming(take(0u8))(remaining)?;

        let flingies = (0..BLOCK_SIZE)
            .map(|i| Flingy {
                sprite: sprite_col[i].clone(),
                top_speed: top_speed_col[i],
                acceleration: acceleration_col[i],
                halt_distance: halt_distance_col[i],
                turn_radius: turn_radius_col[i],
                move_control: move_control_col[i],
            })
            .collect::<Vec<_>>();

        Ok((remaining, FlingyDat(flingies)))
    }
}
