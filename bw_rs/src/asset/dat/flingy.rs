use super::SpritePointer;
use bevy::reflect::TypeUuid;
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

#[derive(Debug, Getters)]
pub struct Flingy {
    sprite: SpritePointer,
    top_speed: u32,
    acceleration: u16,
    halt_distance: u32,
    turn_radius: u8,
    move_control: u8,
}

make_pointer!(FlingyPointer, u32);

make_dat!(
    FlingyDat,
    Flingy,
    FlingyPointer,
    "464d53bc-d9c0-43ba-bdc3-cee2b11167b4"
);

const BLOCK_SIZE: usize = 209;
count_total!(BLOCK_SIZE);

pub(super) fn parse_flingy_dat<'a>(b: &'a [u8]) -> IResult<&[u8], FlingyDat> {
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
