use nom::{
    bytes::complete::take,
    combinator::all_consuming,
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u16},
    IResult, Parser,
};

use crate::count_total;

#[derive(Debug)]
pub struct Upgrade {
    pub mineral_cost: u16,
    pub mineral_factor: u16,
    pub vespene_cost: u16,
    pub vespene_factor: u16,
    pub time_cost: u16,
    pub time_factor: u16,
    pub icon: u16,
    pub label: u16,
    pub race: u8,
    pub max_repeats: u8,
    pub brood_war_specific: u8,
}

const BLOCK_SIZE: usize = 61;
count_total!(BLOCK_SIZE);

#[derive(Debug)]
pub struct UpgradesDat(Vec<Upgrade>);

impl UpgradesDat {
    pub fn parse(b: &[u8]) -> IResult<&[u8], UpgradesDat> {
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
}
