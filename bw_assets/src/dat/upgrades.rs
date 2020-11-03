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
    number::complete::{le_u8, le_u16},
    Finish, IResult, Parser,
};

#[derive(Debug)]
pub struct Upgrade {
    mineral_cost: u16,
    mineral_factor: u16,
    vespene_cost: u16,
    vespene_factor: u16,
    time_cost: u16,
    time_factor: u16,
    icon: u16,
    label: u16,
    race: u8,
    max_repeats: u8,
    brood_war_specific: u8,
}

pub struct UpgradesDat(Vec<Upgrade>);

pub struct UpgradesDatAsset(Option<UpgradesDat>);

impl UpgradesDatAsset {
    pub fn take(&mut self) -> Option<UpgradesDat> {
        self.0.take()
    }
}

pub type UpgradesDatHandle = Handle<UpgradesDatAsset>;

impl Asset for UpgradesDatAsset {
    const NAME: &'static str = "bw_assets::dat::UpgradesDatAsset";
    type Data = Self;
    type HandleStorage = DenseVecStorage<UpgradesDatHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UpgradesDatFormat;

impl Format<UpgradesDatAsset> for UpgradesDatFormat {
    fn name(&self) -> &'static str {
        "UpgradesDatAsset"
    }

    fn import_simple(&self, b: Vec<u8>) -> amethyst::Result<UpgradesDatAsset> {
        let (_, upgrades_dat) = parse_upgrades_dat(&b).finish().map_err(|err| {
            amethyst::error::format_err!(
                "failed to load upgrades.dat asset: {} at position {}",
                err.code.description(),
                b.len() - err.input.len()
            )
        })?;

        Ok(UpgradesDatAsset(Some(upgrades_dat)))
    }
}

const BLOCK_SIZE: usize = 61;

pub fn count_total<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    count(f, BLOCK_SIZE)
}

fn parse_upgrades_dat(b: &[u8]) -> IResult<&[u8], UpgradesDat> {
    let (remaining, mineral_cost_col) = count_total(le_u16)(b)?;
    let (remaining, mineral_factor_col) = count_total(le_u16)(remaining)?;
    let (remaining, vespene_cost_col) = count_total(le_u16)(remaining)?;
    let (remaining, vespene_factor_col) = count_total(le_u16)(remaining)?;
    let (remaining, time_cost_col) = count_total(le_u16)(remaining)?;
    let (remaining, time_factor_col) = count_total(le_u16)(remaining)?;

    // unknown block
    let (remaining, _) = count_total(le_u16)(remaining)?;

    let (remaining, icon_col) = count_total(le_u16)(remaining)?;
    let (remaining, label_col) = count_total(le_u16)(remaining)?;
    let (remaining, race_col) = count_total(le_u8)(remaining)?;
    let (remaining, max_repeats_col) = count_total(le_u8)(remaining)?;
    let (remaining, brood_war_specific_col) = count_total(le_u8)(remaining)?;

    all_consuming(take(0u8))(remaining)?;

    let upgrades = (0..BLOCK_SIZE)
        .map(|i| Upgrade {
            mineral_cost: mineral_cost_col[i],
            mineral_factor: mineral_factor_col[i],
            vespene_cost: vespene_cost_col[i],
            vespene_factor: vespene_factor_col[i],
            time_cost: time_cost_col[i],
            time_factor: time_factor_col[i],
            icon: icon_col[i],
            label: label_col[i],
            race: race_col[i],
            max_repeats: max_repeats_col[i],
            brood_war_specific: brood_war_specific_col[i],
        })
        .collect::<Vec<_>>();

    Ok((remaining, UpgradesDat(upgrades)))
}
