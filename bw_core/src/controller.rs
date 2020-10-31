use std::ops::Index;

use num_derive::FromPrimitive;

use crate::unit::UnitOwner;

#[derive(Debug, Clone, FromPrimitive, Eq, PartialEq)]
pub enum Controller {
    Inactive = 00,
    RescuePassive = 03,
    Unused = 04,
    Computer = 05,
    HumanOpenSlot = 06,
    Neutral = 07,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Controllers(Vec<Controller>);

impl Controllers {
    pub fn new(controllers: Vec<Controller>) -> Controllers {
        Controllers(controllers)
    }
}

impl Index<UnitOwner> for Controllers {
    type Output = Controller;

    fn index(&self, owner: UnitOwner) -> &Self::Output {
        self.index(&owner)
    }
}

impl Index<&UnitOwner> for Controllers {
    type Output = Controller;

    fn index(&self, owner: &UnitOwner) -> &Self::Output {
        &self.0[usize::from(owner)]
    }
}
