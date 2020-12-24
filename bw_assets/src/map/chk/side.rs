use nom::{self, combinator::map_opt, number::complete::le_u8};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug, Clone, FromPrimitive, Eq, PartialEq)]
pub enum Side {
    Zerg = 00,
    Terran = 01,
    Protoss = 02,
    Independent = 03,
    Neutral = 04,
    UserSelectable = 05,
    Random = 06,
    Inactive = 07,
}

impl Side {
    pub(super) fn parse(b: &[u8]) -> nom::IResult<&[u8], Side> {
        map_opt(le_u8, FromPrimitive::from_u8)(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    use byteorder::WriteBytesExt;

    macro_rules! test_sides {
        ($side:expr) => {{
            let mut b: Vec<u8> = vec![];
            b.write_u8($side as u8).unwrap();

            assert_that(&Side::parse(&b))
                .is_ok()
                .map(|(_, side)| side)
                .is_equal_to($side);
        }};
    }

    #[test]
    fn it_parses_sides() {
        test_sides!(Side::Zerg);
        test_sides!(Side::Terran);
        test_sides!(Side::Protoss);
        test_sides!(Side::Independent);
        test_sides!(Side::Neutral);
        test_sides!(Side::UserSelectable);
        test_sides!(Side::Random);
        test_sides!(Side::Inactive);
    }
}
