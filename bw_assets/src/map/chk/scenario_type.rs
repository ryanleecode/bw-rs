use nom::{self, combinator::map_opt, number::complete::le_u32};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug, Copy, Clone, FromPrimitive, PartialEq, Eq)]
#[repr(u32)]
pub enum ScenarioType {
    /// Starcraft
    RAWS = 0x53574152,
    /// Brood War
    RAWB = 0x42574152,
}

impl ScenarioType {
    pub(super) fn parse(b: &[u8]) -> nom::IResult<&[u8], ScenarioType> {
        map_opt(le_u32, FromPrimitive::from_u32)(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    use byteorder::{LittleEndian, WriteBytesExt};

    macro_rules! test_scenario_type {
        ($scenario_type:expr) => {{
            let mut b: Vec<u8> = vec![];
            b.write_u32::<LittleEndian>($scenario_type as u32).unwrap();

            assert_that(&ScenarioType::parse(&b))
                .is_ok()
                .map(|(_, scenario_type)| scenario_type)
                .is_equal_to($scenario_type);
        }};
    }

    #[test]
    fn it_parses_scenario_type() {
        test_scenario_type!(ScenarioType::RAWS);
        test_scenario_type!(ScenarioType::RAWB);
    }
}
