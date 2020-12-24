use bevy::reflect::TypeUuid;
use nom::{
    bytes::complete::take,
    combinator::all_consuming,
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u16, le_u32},
    IResult, Parser,
};

use crate::count_total;

#[derive(Debug, Getters)]
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

#[derive(Debug, TypeUuid)]
#[uuid = "189fe04a-64a8-4aab-8f55-828fbe495b32"]
pub struct TechDataDat(Vec<TechData>);

const BLOCK_SIZE: usize = 44;

count_total!(BLOCK_SIZE);

pub(super) fn parse_tech_data_dat(b: &[u8]) -> IResult<&[u8], TechDataDat> {
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

    let (remaining, _) = all_consuming(take(0u8))(remaining)?;

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
