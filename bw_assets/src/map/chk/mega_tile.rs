use nom::{self, combinator::map, number::complete::le_u16};

use struple::Struple;

#[derive(Debug, Clone, Hash, Struple, Eq, PartialEq)]
pub struct MegaTile(u16);

impl MegaTile {
    pub const PIXEL_HEIGHT: u32 = 32;
    pub const PIXEL_WIDTH: u32 = 32;
    pub const SIDE_LENGTH: u32 = 4;

    pub(super) fn parse(b: &[u8]) -> nom::IResult<&[u8], MegaTile> {
        map(le_u16, MegaTile)(b)
    }

    pub fn group_index(&self) -> usize {
        return ((self.0 >> 4) & 0x7ff) as usize;
    }

    pub fn subtile_index(&self) -> usize {
        return (self.0 & 0xf) as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use spectral::prelude::*;

    use byteorder::{LittleEndian, WriteBytesExt};

    #[test]
    fn it_parses_mega_tile() {
        let mut rng = rand::thread_rng();
        let mut b: Vec<u8> = vec![];
        let value = rng.gen_range(0, 256);
        b.write_u16::<LittleEndian>(value).unwrap();

        assert_that(&MegaTile::parse(&b))
            .is_ok()
            .map(|(_, megatile)| megatile)
            .is_equal_to(MegaTile(value));
    }
}
