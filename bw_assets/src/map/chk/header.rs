use nom::{
    self,
    branch::alt,
    bytes::complete::{tag, take},
    combinator::map,
    number::complete::le_u32,
};

use std::mem;

use super::ChunkName;

const HEADER_NAME_BYTE_SIZE: usize = 4usize;

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    /// Name of the subsequence data block
    pub(super) name: ChunkName,

    /// The size of the subsequent data block
    pub(super) size: u32,
}

impl Header {
    pub(super) fn parse(b: &[u8]) -> nom::IResult<&[u8], Header> {
        let (remaining, name) = alt((
            map(tag(ChunkName::Type.as_bytes()), |_| ChunkName::Type),
            map(tag(ChunkName::Version.as_bytes()), |_| ChunkName::Version),
            map(tag(ChunkName::Tileset.as_bytes()), |_| ChunkName::Tileset),
            map(tag(ChunkName::Controllers.as_bytes()), |_| {
                ChunkName::Controllers
            }),
            map(tag(ChunkName::Dimensions.as_bytes()), |_| {
                ChunkName::Dimensions
            }),
            map(tag(ChunkName::Side.as_bytes()), |_| ChunkName::Side),
            map(tag(ChunkName::MegaTiles.as_bytes()), |_| {
                ChunkName::MegaTiles
            }),
            map(tag(ChunkName::StringData.as_bytes()), |_| {
                ChunkName::StringData
            }),
            map(tag(ChunkName::Unit.as_bytes()), |_| ChunkName::Unit),
            map(take(HEADER_NAME_BYTE_SIZE), |_| ChunkName::Unknown),
        ))(b)?;

        let (remaining, size) = le_u32(remaining)?;

        Ok((remaining, Header { name, size }))
    }

    pub fn size_of() -> usize {
        HEADER_NAME_BYTE_SIZE + mem::size_of::<u32>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use spectral::prelude::*;

    use byteorder::{LittleEndian, WriteBytesExt};

    macro_rules! test_header {
        ($name:expr) => {{
            let mut rng = rand::thread_rng();
            let size = rng.gen_range(0, 100);
            let mut b: Vec<u8> = $name.as_bytes().into();
            b.write_u32::<LittleEndian>(size).unwrap();

            assert_that(&Header::parse(&b))
                .is_ok()
                .map(|(_, header)| header)
                .is_equal_to(Header { name: $name, size });
        }};
    }

    #[test]
    fn it_parses_header() {
        test_header!(ChunkName::Type);
        test_header!(ChunkName::Version);
        test_header!(ChunkName::Tileset);
        test_header!(ChunkName::Controllers);
        test_header!(ChunkName::Dimensions);
        test_header!(ChunkName::Side);
        test_header!(ChunkName::MegaTiles);
        test_header!(ChunkName::StringData);
    }
}
