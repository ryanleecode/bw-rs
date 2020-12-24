use crate::{count_total, make_dat, make_pointer, tbl::ImagesTblPointer};
use nom::{
    bytes::complete::take,
    combinator::{all_consuming, map},
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u32},
    IResult, Parser,
};
use std::ops::Index;

#[derive(Debug)]
pub struct Image {
    pub grp_file_name_ptr: ImagesTblPointer,
    pub has_directional_frames: bool,
    pub is_clickable: bool,
    pub has_iscript_animations: bool,
    pub always_visible: bool,
    pub modifier: u8,
    pub color_shift: u8,
    pub iscript_id: u32,
    pub shield_filename_index: u32,
    pub attack_filename_index: u32,
    pub damage_filename_index: u32,
    pub special_filename_index: u32,
    pub landing_dust_filename_index: u32,
    pub lift_off_filename_index: u32,
}

make_pointer!(ImagePointer, u16);

make_dat!(ImagesDat, Image, ImagePointer);

const BLOCK_SIZE: usize = 999;
count_total!(BLOCK_SIZE);

impl ImagesDat {
    pub fn parse(b: &[u8]) -> IResult<&[u8], ImagesDat> {
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
}

fn parse_bool(b: &[u8]) -> IResult<&[u8], bool> {
    map(le_u8, |x| x > 0)(b)
}
