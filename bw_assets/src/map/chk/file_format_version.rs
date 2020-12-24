use nom::{self, combinator::map_opt, number::complete::le_u16};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug, Clone, FromPrimitive, PartialEq, Eq)]
pub enum FileFormatVersion {
    /// 1.00 Starcraft
    Starcraft = 59,

    /// 1.04 Starcraft and above ("hybrid")
    StarcraftHybrid = 63,

    /// Brood War
    BroodWar = 205,
}

impl FileFormatVersion {
    pub(super) fn parse(b: &[u8]) -> nom::IResult<&[u8], FileFormatVersion> {
        map_opt(le_u16, FromPrimitive::from_u16)(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    use byteorder::{LittleEndian, WriteBytesExt};

    macro_rules! test_file_format_version {
        ($version:expr) => {{
            let mut b: Vec<u8> = vec![];
            b.write_u16::<LittleEndian>($version as u16).unwrap();

            assert_that(&FileFormatVersion::parse(&b))
                .is_ok()
                .map(|(_, version)| version)
                .is_equal_to($version);
        }};
    }

    #[test]
    fn it_parses_file_format_version() {
        test_file_format_version!(FileFormatVersion::Starcraft);
        test_file_format_version!(FileFormatVersion::StarcraftHybrid);
        test_file_format_version!(FileFormatVersion::BroodWar);
    }
}
