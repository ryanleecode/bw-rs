use super::Owner;
use nom::{self, combinator::map_opt, number::complete::le_u8};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::ops::Index;

#[derive(Debug, Clone, FromPrimitive, Eq, PartialEq)]
pub enum Controller {
    Inactive = 00,
    RescuePassive = 03,
    Unused = 04,
    Computer = 05,
    HumanOpenSlot = 06,
    Neutral = 07,
}

impl Controller {
    pub(super) fn parse(b: &[u8]) -> nom::IResult<&[u8], Controller> {
        map_opt(le_u8, FromPrimitive::from_u8)(b)
    }
}

#[derive(Debug)]
pub struct Controllers(pub(super) Vec<Controller>);

impl Index<Owner> for Controllers {
    type Output = Controller;

    fn index(&self, owner: Owner) -> &Self::Output {
        self.index(&owner)
    }
}

impl Index<&Owner> for Controllers {
    type Output = Controller;

    fn index(&self, owner: &Owner) -> &Self::Output {
        &self.0[usize::from(owner)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    use byteorder::WriteBytesExt;

    macro_rules! test_controller {
        ($controller:expr) => {{
            let mut b: Vec<u8> = vec![];
            b.write_u8($controller as u8).unwrap();

            assert_that(&Controller::parse(&b))
                .is_ok()
                .map(|(_, controller)| controller)
                .is_equal_to($controller);
        }};
    }

    #[test]
    fn it_parses_controller() {
        test_controller!(Controller::Inactive);
        test_controller!(Controller::RescuePassive);
        test_controller!(Controller::Unused);
        test_controller!(Controller::Computer);
        test_controller!(Controller::HumanOpenSlot);
        test_controller!(Controller::Neutral);
    }
}
