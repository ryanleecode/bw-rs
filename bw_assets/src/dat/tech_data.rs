use amethyst::{
    assets::Format,
    assets::{Asset, Handle},
    ecs::DenseVecStorage,
};
use nom::{
    bytes::complete::take,
    combinator::all_consuming,
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u16, le_u32},
    Finish, IResult, Parser,
};

#[derive(Debug)]
pub struct TechData {
    mineral_cost: u16,
    vespene_cost: u16,
    research_time: u16,
    energy_cost: u16,
    icon: u16,
    label: u16,
    race: u8,
    broodwar: u8,
}

pub struct TechDataDat(Vec<TechData>);

pub struct TechDataDatAsset(Option<TechDataDat>);

impl TechDataDatAsset {
    pub fn take(&mut self) -> Option<TechDataDat> {
        self.0.take()
    }
}

pub type TechDataDatHandle = Handle<TechDataDatAsset>;

impl Asset for TechDataDatAsset {
    const NAME: &'static str = "bw_assets::dat::TechDataDatAsset";
    type Data = Self;
    type HandleStorage = DenseVecStorage<TechDataDatHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TechDataDatFormat;

impl Format<TechDataDatAsset> for TechDataDatFormat {
    fn name(&self) -> &'static str {
        "TechDataDatAsset"
    }

    fn import_simple(&self, b: Vec<u8>) -> amethyst::Result<TechDataDatAsset> {
        let (_, techdata_dat) = parse_tech_dat_dat(&b).finish().map_err(|err| {
            amethyst::error::format_err!(
                "failed to load techdata.dat asset: {} at position {}",
                err.code.description(),
                b.len() - err.input.len()
            )
        })?;

        Ok(TechDataDatAsset(Some(techdata_dat)))
    }
}

const BLOCK_SIZE: usize = 44;

pub fn count_total<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    count(f, BLOCK_SIZE)
}

fn parse_tech_dat_dat(b: &[u8]) -> IResult<&[u8], TechDataDat> {
    let (remaining, mineral_cost_col) = count_total(le_u16)(b)?;
    let (remaining, vespene_cost_col) = count_total(le_u16)(remaining)?;
    let (remaining, research_time_col) = count_total(le_u16)(remaining)?;
    let (remaining, energy_cost_col) = count_total(le_u16)(remaining)?;

    // unknown block
    let (remaining, _) = count_total(le_u32)(remaining)?;

    let (remaining, icon_cost_col) = count_total(le_u16)(remaining)?;
    let (remaining, label_cost_col) = count_total(le_u16)(remaining)?;
    let (remaining, race_cost_col) = count_total(le_u8)(remaining)?;

    // unknown block
    let (remaining, _) = count_total(le_u8)(remaining)?;

    // unknown block
    let (remaining, broodwar_col) = count_total(le_u8)(remaining)?;

    all_consuming(take(0u8))(remaining)?;

    let tech_data = (0..BLOCK_SIZE)
        .map(|i| TechData {
            mineral_cost: mineral_cost_col[i],
            vespene_cost: vespene_cost_col[i],
            research_time: research_time_col[i],
            energy_cost: energy_cost_col[i],
            icon: icon_cost_col[i],
            label: label_cost_col[i],
            race: race_cost_col[i],
            broodwar: broodwar_col[i],
        })
        .collect::<Vec<_>>();

    Ok((remaining, TechDataDat(tech_data)))
}
