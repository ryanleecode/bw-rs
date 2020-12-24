use nom::{
    self,
    combinator::{map, map_opt},
    number::complete::le_u16,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug, Clone, Hash, FromPrimitive, Eq, PartialEq)]
pub enum Tileset {
    Badlands = 00,
    SpacePlatform = 01,
    Installation = 02,
    Ashworld = 03,
    Jungle = 04,
    Desert = 05,
    Arctic = 06,
    Twilight = 07,
}

impl Tileset {
    // Mask on the byte value of the tileset to limit it to 2^3.
    const MASK: u16 = 0b0111;

    pub fn name(&self) -> String {
        match self {
            Tileset::Ashworld => "ashworld".into(),
            Tileset::Badlands => "badlands".into(),
            Tileset::Installation => "install".into(),
            Tileset::Jungle => "jungle".into(),
            Tileset::SpacePlatform => "platform".into(),
            Tileset::Desert => "desert".into(),
            Tileset::Arctic => "ice".into(),
            Tileset::Twilight => "twilight".into(),
        }
    }

    pub(super) fn parse(b: &[u8]) -> nom::IResult<&[u8], Tileset> {
        map_opt(map(le_u16, |n| n & Tileset::MASK), FromPrimitive::from_u16)(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    use byteorder::{LittleEndian, WriteBytesExt};

    macro_rules! test_tileset {
        ($tileset:expr) => {{
            let mut b: Vec<u8> = vec![];
            b.write_u16::<LittleEndian>($tileset as u16).unwrap();

            assert_that(&Tileset::parse(&b))
                .is_ok()
                .map(|(_, tileset)| tileset)
                .is_equal_to($tileset);
        }};
    }

    #[test]
    fn it_parses_tileset() {
        test_tileset!(Tileset::Badlands);
        test_tileset!(Tileset::SpacePlatform);
        test_tileset!(Tileset::Installation);
        test_tileset!(Tileset::Ashworld);
        test_tileset!(Tileset::Jungle);
        test_tileset!(Tileset::Desert);
        test_tileset!(Tileset::Arctic);
        test_tileset!(Tileset::Twilight);
    }
}
