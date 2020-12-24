use bevy::reflect::TypeUuid;
use nom::{
    bytes::complete::take,
    combinator::all_consuming,
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u16},
    IResult, Parser,
};

use crate::count_total;

#[derive(Debug, Getters)]
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

#[derive(Debug, TypeUuid)]
#[uuid = "b494db96-6f7b-49e4-bb3b-9cf7bb213b57"]
pub struct UpgradesDat(Vec<Upgrade>);

const BLOCK_SIZE: usize = 61;
count_total!(BLOCK_SIZE);

pub(super) fn parse_upgrades_dat(b: &[u8]) -> IResult<&[u8], UpgradesDat> {
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

    let (remaining, _) = all_consuming(take(0u8))(remaining)?;

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
