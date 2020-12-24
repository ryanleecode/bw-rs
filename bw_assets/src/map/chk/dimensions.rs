use nom::{self, combinator::map, number::complete::le_u16, sequence::tuple};

use struple::Struple;
#[derive(Debug, Clone, Struple, Eq, PartialEq)]
pub struct Dimensions {
    pub width: u16,
    pub height: u16,
}

impl Dimensions {
    pub(super) fn parse(b: &[u8]) -> nom::IResult<&[u8], Dimensions> {
        map(tuple((le_u16, le_u16)), Dimensions::from_tuple)(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use spectral::prelude::*;

    use byteorder::{LittleEndian, WriteBytesExt};

    #[test]
    fn it_parses_dimensions() {
        let mut rng = rand::thread_rng();
        let mut b: Vec<u8> = vec![];

        let width = rng.gen_range(0, 256);
        let height = rng.gen_range(0, 256);

        b.write_u16::<LittleEndian>(width).unwrap();
        b.write_u16::<LittleEndian>(height).unwrap();

        assert_that(&Dimensions::parse(&b))
            .is_ok()
            .map(|(_, dimensions)| dimensions)
            .is_equal_to(Dimensions { width, height });
    }
}
