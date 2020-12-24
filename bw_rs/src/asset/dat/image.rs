use crate::{asset::ImagesTblPointer, count_total, make_dat, make_pointer};
use bevy::reflect::TypeUuid;
use nom::{
    bytes::complete::take,
    combinator::{all_consuming, map},
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u32},
    IResult, Parser,
};
use std::ops::Index;

#[derive(Debug, Getters)]
pub struct Image {
    grp_file_name_ptr: ImagesTblPointer,
    has_directional_frames: bool,
    is_clickable: bool,
    has_iscript_animations: bool,
    always_visible: bool,
    modifier: u8,
    color_shift: u8,
    iscript_id: u32,
    shield_filename_index: u32,
    attack_filename_index: u32,
    damage_filename_index: u32,
    special_filename_index: u32,
    landing_dust_filename_index: u32,
    lift_off_filename_index: u32,
}

make_pointer!(ImagePointer, u16);

make_dat!(
    ImagesDat,
    Image,
    ImagePointer,
    "b4c17166-6764-4e3b-ab45-abe8afd4fbbc"
);

const BLOCK_SIZE: usize = 999;
count_total!(BLOCK_SIZE);

fn parse_bool(b: &[u8]) -> IResult<&[u8], bool> {
    map(le_u8, |x| x > 0)(b)
}

pub(super) fn parse_images_dat(b: &[u8]) -> IResult<&[u8], ImagesDat> {
    let (remaining, grp_col) = count_total(map(le_u32, ImagesTblPointer))(b)?;
    let (remaining, has_directional_frames_col) = count_total(parse_bool)(remaining)?;
    let (remaining, is_clickable_col) = count_total(parse_bool)(remaining)?;
    let (remaining, has_iscript_animations_col) = count_total(parse_bool)(remaining)?;
    let (remaining, always_visible_col) = count_total(parse_bool)(remaining)?;
    let (remaining, modifier_col) = count_total(le_u8)(remaining)?;
    let (remaining, color_shift_col) = count_total(le_u8)(remaining)?;
    let (remaining, iscript_id_col) = count_total(le_u32)(remaining)?;
    let (remaining, shield_filename_index_col) = count_total(le_u32)(remaining)?;
    let (remaining, attack_filename_index_col) = count_total(le_u32)(remaining)?;
    let (remaining, damage_filename_index_col) = count_total(le_u32)(remaining)?;
    let (remaining, special_filename_index_col) = count_total(le_u32)(remaining)?;
    let (remaining, landing_dust_filename_index_col) = count_total(le_u32)(remaining)?;
    let (remaining, lift_off_filename_index_col) = count_total(le_u32)(remaining)?;

    let (remaining, _) = all_consuming(take(0u8))(remaining)?;

    let images = (0..BLOCK_SIZE)
        .map(|i| Image {
            grp_file_name_ptr: grp_col[i].clone(),
            has_directional_frames: has_directional_frames_col[i],
            is_clickable: is_clickable_col[i],
            has_iscript_animations: has_iscript_animations_col[i],
            always_visible: always_visible_col[i],
            modifier: modifier_col[i],
            color_shift: color_shift_col[i],
            iscript_id: iscript_id_col[i],
            shield_filename_index: shield_filename_index_col[i],
            attack_filename_index: attack_filename_index_col[i],
            damage_filename_index: damage_filename_index_col[i],
            special_filename_index: special_filename_index_col[i],
            landing_dust_filename_index: landing_dust_filename_index_col[i],
            lift_off_filename_index: lift_off_filename_index_col[i],
        })
        .collect::<Vec<_>>();

    Ok((remaining, ImagesDat(images)))
}
