use super::ImagePointer;
use std::ops::Index;

use crate::{count_total, make_dat, make_pointer};

use boolinator::Boolinator;
use nom::{
    bytes::complete::take,
    combinator::{all_consuming, map},
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u16},
    IResult, Parser,
};

#[derive(Debug)]
pub struct Sprite {
    pub image_file: ImagePointer,
    pub health_bar: Option<u8>,
    pub is_visible: u8,
    pub selection_circle_image: Option<u8>,
    pub selection_circle_offset: Option<u8>,
}

make_pointer!(SpritePointer, u16);

make_dat!(SpritesDat, Sprite, SpritePointer);

const SELECTABLE_COUNT: usize = 387;
const NON_SELECTABLE_COUNT: usize = 130;
const BLOCK_SIZE: usize = 517;

count_total!(BLOCK_SIZE);

fn count_selectable_block<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    count(f, SELECTABLE_COUNT)
}

impl SpritesDat {
    pub fn parse(b: &[u8]) -> IResult<&[u8], SpritesDat> {
        let (remaining, image_file_col) = count_total(map(le_u16, ImagePointer))(b)?;
        let (remaining, health_bar_col) = count_selectable_block(le_u8)(remaining)?;

        let (remaining, _) = count_total(le_u8)(remaining)?;

        let (remaining, is_visible_col) = count_total(le_u8)(remaining)?;
        let (remaining, selection_circle_image_col) = count_selectable_block(le_u8)(remaining)?;
        let (remaining, selection_circle_offset_col) = count_selectable_block(le_u8)(remaining)?;

        let (remaining, _) = all_consuming(take(0u8))(remaining)?;

        let sprites = (0..BLOCK_SIZE)
            .map(|i| Sprite {
                image_file: image_file_col[i].clone(),
                health_bar: (i >= NON_SELECTABLE_COUNT
                    && i < NON_SELECTABLE_COUNT + SELECTABLE_COUNT)
                    .and_option_from(|| {
                        health_bar_col
                            .get(i - NON_SELECTABLE_COUNT)
                            .map(ToOwned::to_owned)
                    }),
                is_visible: is_visible_col[i],
                selection_circle_image: (i >= NON_SELECTABLE_COUNT
                    && i < NON_SELECTABLE_COUNT + SELECTABLE_COUNT)
                    .and_option_from(|| {
                        selection_circle_image_col
                            .get(i - NON_SELECTABLE_COUNT)
                            .map(ToOwned::to_owned)
                    }),
                selection_circle_offset: (i >= NON_SELECTABLE_COUNT
                    && i < NON_SELECTABLE_COUNT + SELECTABLE_COUNT)
                    .and_option_from(|| {
                        selection_circle_offset_col
                            .get(i - NON_SELECTABLE_COUNT)
                            .map(ToOwned::to_owned)
                    }),
            })
            .collect::<Vec<_>>();

        Ok((remaining, SpritesDat(sprites)))
    }
}
