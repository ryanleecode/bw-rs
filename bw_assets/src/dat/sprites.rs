use amethyst::{
    assets::Format,
    assets::{Asset, Handle},
    ecs::DenseVecStorage,
};
use boolinator::Boolinator;
use nom::{
    bytes::complete::take,
    combinator::all_consuming,
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u16},
    Finish, IResult, Parser,
};

#[derive(Debug)]
pub struct Sprite {
    image_file: u16,
    health_bar: Option<u8>,
    is_visible: u8,
    selection_circle_image: Option<u8>,
    selection_circle_offset: Option<u8>,
}

pub struct SpritesDat(Vec<Sprite>);

pub struct SpritesDatAsset(Option<SpritesDat>);

impl SpritesDatAsset {
    pub fn take(&mut self) -> Option<SpritesDat> {
        self.0.take()
    }
}

pub type SpritesDatHandle = Handle<SpritesDatAsset>;

impl Asset for SpritesDatAsset {
    const NAME: &'static str = "bw_assets::dat::SpritesDatAsset";
    type Data = Self;
    type HandleStorage = DenseVecStorage<SpritesDatHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SpritesDatFormat;

impl Format<SpritesDatAsset> for SpritesDatFormat {
    fn name(&self) -> &'static str {
        "SpritesDatAsset"
    }

    fn import_simple(&self, b: Vec<u8>) -> amethyst::Result<SpritesDatAsset> {
        let (_, sprites_dat) = parse_sprites_dat(&b).finish().map_err(|err| {
            amethyst::error::format_err!(
                "failed to load sprites.dat asset: {} at position {}",
                err.code.description(),
                b.len() - err.input.len()
            )
        })?;

        Ok(SpritesDatAsset(Some(sprites_dat)))
    }
}

const SELECTABLE_COUNT: usize = 387;
const NON_SELECTABLE_COUNT: usize = 130;
const BLOCK_SIZE: usize = 517;

pub fn count_total<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    count(f, BLOCK_SIZE)
}

pub fn count_selectable_block<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    count(f, SELECTABLE_COUNT)
}

fn parse_sprites_dat(b: &[u8]) -> IResult<&[u8], SpritesDat> {
    let (remaining, image_file_col) = count_total(le_u16)(b)?;
    let (remaining, health_bar_col) = count_selectable_block(le_u8)(remaining)?;

    let (remaining, _) = count_total(le_u8)(remaining)?;

    let (remaining, is_visible_col) = count_total(le_u8)(remaining)?;
    let (remaining, selection_circle_image_col) = count_selectable_block(le_u8)(remaining)?;
    let (remaining, selection_circle_offset_col) = count_selectable_block(le_u8)(remaining)?;

    all_consuming(take(0u8))(remaining)?;

    let sprites = (0..BLOCK_SIZE)
        .map(|i| Sprite {
            image_file: image_file_col[i],
            health_bar: (i >= NON_SELECTABLE_COUNT && i < NON_SELECTABLE_COUNT + SELECTABLE_COUNT)
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
