use crate::dat::UnitId;

use super::Owner;
use nom::{
    self,
    combinator::{map, map_opt},
    number::complete::{le_u8, le_u16, le_u32},
    sequence::{preceded, tuple},
};

use num_traits::FromPrimitive;
use struple::Struple;

/// Pre-placed units on the map and their properties
///
/// http://www.staredit.net/wiki/index.php?title=Scenario.chk#.22UNIT.22_-_Placed_Units
#[derive(Debug, Struple, Eq, PartialEq)]
pub struct Unit {
    pub serial_number: u32,
    pub x: u16,
    pub y: u16,
    pub unit_id: UnitId,
    pub relation_flag: u16,
    pub special_property_flags: u16,
    pub map_maker_flags: u16,
    pub owner: Owner,

    /// Hit points % (1-100)
    pub hitpoints_percentage: u8,

    /// Shield points % (1-100)
    pub shield_points_percentage: u8,

    /// Energy points % (1-100)
    pub energy_points_percentage: u8,

    pub resource_amount: u32,
    pub units_in_hangar: u16,
    pub unit_state_flags: u16,

    /// Class instance of the unit to which this unit is related to (i.e. via an
    /// add-on, nydus link, etc.). It is "0" if the unit is not linked to any other
    /// unit.
    pub class_instance: u32,
}

impl Unit {
    pub(super) fn parse(b: &[u8]) -> nom::IResult<&[u8], Unit> {
        map(
            tuple((
                le_u32,
                le_u16,
                le_u16,
                map_opt(le_u16, FromPrimitive::from_u16),
                le_u16,
                le_u16,
                le_u16,
                map(le_u8, Owner),
                le_u8,
                le_u8,
                le_u8,
                le_u32,
                le_u16,
                le_u16,
                preceded(le_u32, le_u32),
            )),
            Unit::from_tuple,
        )(b)
    }

    pub fn is_mineral_field(&self) -> bool {
        self.unit_id == UnitId::ResourceMineralField
            || self.unit_id == UnitId::ResourceMineralFieldType2
            || self.unit_id == UnitId::ResourceMineralFieldType3
    }

    /// Checks if the unit is a structure that is placed on top of a Vespene Geyser.
    ///
    /// Refinery types are Terran Refinery, Zerg Extractor, and Protoss Assimilator.
    pub fn is_refinery(&self) -> bool {
        self.unit_id == UnitId::TerranRefinery
            || self.unit_id == UnitId::ZergExtractor
            || self.unit_id == UnitId::ProtossAssimilator
    }

    pub fn is_worker(&self) -> bool {
        self.unit_id == UnitId::TerranScv
            || self.unit_id == UnitId::ZergDrone
            || self.unit_id == UnitId::ProtossProbe
    }
}
