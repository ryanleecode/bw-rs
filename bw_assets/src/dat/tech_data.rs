use nom::{
    bytes::complete::take,
    combinator::all_consuming,
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u16, le_u32},
    IResult, Parser,
};

use crate::count_total;

#[derive(Debug)]
pub struct TechData {
    pub mineral_cost: u16,
    pub vespene_cost: u16,
    pub research_time: u16,
    pub energy_cost: u16,
    pub icon: u16,
    pub label: u16,
    pub race: u8,
    pub broodwar: u8,
}

#[derive(Debug)]
pub struct TechDataDat(Vec<TechData>);

const BLOCK_SIZE: usize = 44;
count_total!(BLOCK_SIZE);

impl TechDataDat {
    pub fn parse(b: &[u8]) -> IResult<&[u8], TechDataDat> {
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
}
