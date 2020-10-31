use struple::Struple;

mod unit_id;

pub use unit_id::UnitId;

/// Owner of a unit.
///
/// Can be a player, computer, NPC, etc...
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnitOwner(u8);

impl UnitOwner {
    pub fn new(x: u8) -> UnitOwner {
        UnitOwner(x)
    }
}

impl From<UnitOwner> for usize {
    fn from(unit_owner: UnitOwner) -> Self {
        usize::from(&unit_owner)
    }
}

impl From<&UnitOwner> for usize {
    fn from(unit_owner: &UnitOwner) -> Self {
        unit_owner.0 as usize
    }
}

/// Pre-placed units on the map and their properties
///
/// http://www.staredit.net/wiki/index.php?title=Scenario.chk#.22UNIT.22_-_Placed_Units
#[derive(Debug, Clone, Struple, Eq, PartialEq)]
pub struct Unit {
    serial_number: u32,
    x: u16,
    y: u16,
    unit_id: Option<UnitId>,
    relation_flag: u16,
    special_property_flags: u16,
    map_maker_flags: u16,
    owner: UnitOwner,

    /// Hit points % (1-100)
    hitpoints_percentage: u8,

    /// Shield points % (1-100)
    shield_points_percentage: u8,

    /// Energy points % (1-100)
    energy_points_percentage: u8,

    resource_amount: u32,
    units_in_hangar: u16,
    unit_state_flags: u16,

    /// Class instance of the unit to which this unit is related to (i.e. via an
    /// add-on, nydus link, etc.). It is "0" if the unit is not linked to any other
    /// unit.
    class_instance: u32,
}

impl Unit {
    pub fn is_mineral_field(&self) -> bool {
        self.unit_id == Some(UnitId::ResourceMineralField)
            || self.unit_id == Some(UnitId::ResourceMineralFieldType2)
            || self.unit_id == Some(UnitId::ResourceMineralFieldType3)
    }

    /// Checks if the unit is a structure that is placed on top of a Vespene Geyser.
    ///
    /// Refinery types are Terran Refinery, Zerg Extractor, and Protoss Assimilator.
    pub fn is_refinery(&self) -> bool {
        self.unit_id == Some(UnitId::TerranRefinery)
            || self.unit_id == Some(UnitId::ZergExtractor)
            || self.unit_id == Some(UnitId::ProtossAssimilator)
    }

    pub fn is_worker(&self) -> bool {
        self.unit_id == Some(UnitId::TerranScv)
            || self.unit_id == Some(UnitId::ZergDrone)
            || self.unit_id == Some(UnitId::ProtossProbe)
    }
}
