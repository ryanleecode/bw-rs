use amethyst::{
    assets::Format,
    assets::{Asset, Handle},
    ecs::DenseVecStorage,
};
use nom::{
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u16, le_u32},
    IResult,
};

#[derive(Debug)]
pub struct Flingy {
    sprite: u16,
    top_speed: u32,
    acceleration: u16,
    halt_distance: u32,
    turn_radius: u8,
    move_control: u8,
}

pub struct FlingyDat(Vec<Flingy>);

pub struct FlingyDatAsset(Option<FlingyDat>);

impl FlingyDatAsset {
    pub fn take(&mut self) -> Option<FlingyDat> {
        self.0.take()
    }
}

pub type FlingyDatHandle = Handle<FlingyDatAsset>;

impl Asset for FlingyDatAsset {
    const NAME: &'static str = "bw_assets::dat::FlingyDatAsset";
    type Data = Self;
    type HandleStorage = DenseVecStorage<FlingyDatHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FlingyDatFormat;

impl Format<FlingyDatAsset> for FlingyDatFormat {
    fn name(&self) -> &'static str {
        "FlingyDatFormat"
    }

    fn import_simple(&self, b: Vec<u8>) -> amethyst::Result<FlingyDatAsset> {
        let (_, flingy_dat) = parse_flingy_dat(&b).map_err(|err| err.to_owned())?;


        Ok(FlingyDatAsset(Some(flingy_dat)))
    }
}

const BLOCK_SIZE: usize = 209;

pub fn count_total<I, O, E, F>(f: F) -> impl Fn(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    F: Fn(I) -> IResult<I, O, E>,
    E: ParseError<I>,
{
    count(f, BLOCK_SIZE)
}

fn parse_flingy_dat(b: &[u8]) -> IResult<&[u8], FlingyDat> {
    let (remaining, sprite_col) = count_total(le_u16)(b)?;
    let (remaining, top_speed_col) = count_total(le_u32)(remaining)?;
    let (remaining, acceleration_col) = count_total(le_u16)(remaining)?;
    let (remaining, halt_distance_col) = count_total(le_u32)(remaining)?;
    let (remaining, turn_radius_col) = count_total(le_u8)(remaining)?;

    // unknown block
    let (remaining, _) = count_total(le_u8)(remaining)?;

    let (remaining, move_control_col) = count_total(le_u8)(remaining)?;

    let flingies = (0..BLOCK_SIZE)
        .map(|i| Flingy {
            sprite: sprite_col[i],
            top_speed: top_speed_col[i],
            acceleration: acceleration_col[i],
            halt_distance: halt_distance_col[i],
            turn_radius: turn_radius_col[i],
            move_control: move_control_col[i],
        })
        .collect::<Vec<_>>();

    Ok((remaining, FlingyDat(flingies)))
}
