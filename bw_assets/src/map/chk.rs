use nom::{
    self,
    branch::alt,
    bytes::complete::{tag, take, take_until},
    combinator::{all_consuming, map, map_opt},
    multi::{count, many0},
    number::complete::{le_u16, le_u32, le_u8},
    sequence::{preceded, tuple},
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use std::cmp::max;
use std::convert::From;
use std::fmt;
use std::mem;
use std::str;
use struple::Struple;

const HEADER_NAME_BYTE_SIZE: usize = 4usize;

#[derive(Debug, PartialEq, Eq)]
pub enum Chunk {
    ScenarioType(ScenarioType),
    FileFormatVersion(FileFormatVersion),
    Tileset(Tileset),
    Controllers(Vec<Controller>),
    Dimensions(Dimensions),
    Sides(Vec<Side>),
    MegaTiles(Vec<MegaTile>),
    StringData(StringData),
    Unknown,
}

pub fn parse_chunks(b: &[u8]) -> nom::IResult<&[u8], Vec<Chunk>> {
    all_consuming(many0(parse_chunk))(b)
}

pub fn parse_chunk(b: &[u8]) -> nom::IResult<&[u8], Chunk> {
    let (remaining, header) = parse_header(&b)?;

    match header.name {
        ChunkName::Type => map(parse_scenario_type, Chunk::ScenarioType)(remaining),
        ChunkName::Version => map(parse_file_format_version, Chunk::FileFormatVersion)(remaining),
        ChunkName::Tileset => map(parse_tileset, Chunk::Tileset)(remaining),
        ChunkName::Controllers => {
            let size = header.size as usize / mem::size_of::<Controller>();
            map(count(parse_controller, size), Chunk::Controllers)(remaining)
        }
        ChunkName::Dimensions => map(parse_dimensions, Chunk::Dimensions)(remaining),
        ChunkName::Side => {
            let size = header.size as usize / mem::size_of::<u8>();
            map(count(parse_side, size), Chunk::Sides)(remaining)
        }
        ChunkName::MegaTiles => {
            let size = header.size as usize / mem::size_of::<MegaTile>();
            map(count(parse_megatile, size), Chunk::MegaTiles)(remaining)
        }
        ChunkName::StringData => map(parse_string_data, Chunk::StringData)(remaining),
        _ => map(take(header.size), |_| Chunk::Unknown)(remaining),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChunkName {
    Type,
    Version,
    Tileset,
    Controllers,
    Dimensions,
    Side,
    MegaTiles,
    StringData,
    Unknown,
}

impl fmt::Display for ChunkName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        unsafe { write!(f, "{}", str::from_utf8_unchecked(self.as_bytes())) }
    }
}

impl ChunkName {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            ChunkName::Type => "TYPE".as_bytes(),
            ChunkName::Version => "VER ".as_bytes(),
            ChunkName::Tileset => "ERA ".as_bytes(),
            ChunkName::Controllers => "OWNR".as_bytes(),
            ChunkName::Dimensions => "DIM ".as_bytes(),
            ChunkName::Side => "SIDE".as_bytes(),
            ChunkName::MegaTiles => "MTXM".as_bytes(),
            ChunkName::StringData => "STR ".as_bytes(),
            ChunkName::Unknown => "????".as_bytes(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Header {
    /// Name of the subsequence data block
    pub name: ChunkName,

    /// The size of the subsequent data block
    pub size: u32,
}

impl Header {
    pub fn size_of() -> usize {
        HEADER_NAME_BYTE_SIZE + mem::size_of::<u32>()
    }
}

pub fn parse_header(b: &[u8]) -> nom::IResult<&[u8], Header> {
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
        map(take(HEADER_NAME_BYTE_SIZE), |_| ChunkName::Unknown),
    ))(b)?;

    let (remaining, size) = le_u32(remaining)?;

    Ok((remaining, Header { name, size }))
}

#[derive(Debug, Copy, Clone, FromPrimitive, PartialEq, Eq)]
#[repr(u32)]
pub enum ScenarioType {
    /// Starcraft
    RAWS = 0x53574152,
    /// Brood War
    RAWB = 0x42574152,
}

pub fn parse_scenario_type(b: &[u8]) -> nom::IResult<&[u8], ScenarioType> {
    map_opt(le_u32, FromPrimitive::from_u32)(b)
}

#[derive(Debug, Clone, FromPrimitive, PartialEq, Eq)]
pub enum FileFormatVersion {
    /// 1.00 Starcraft
    Starcraft = 59,

    /// 1.04 Starcraft and above ("hybrid")
    StarcraftHybrid = 63,

    /// Brood War
    BroodWar = 205,
}

pub fn parse_file_format_version(b: &[u8]) -> nom::IResult<&[u8], FileFormatVersion> {
    map_opt(le_u16, FromPrimitive::from_u16)(b)
}

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

    pub fn file_name(&self) -> String {
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
}

pub fn parse_tileset(b: &[u8]) -> nom::IResult<&[u8], Tileset> {
    map_opt(map(le_u16, |n| n & Tileset::MASK), FromPrimitive::from_u16)(b)
}

#[derive(Debug, Clone, FromPrimitive, Eq, PartialEq)]
pub enum Controller {
    Inactive = 00,
    RescuePassive = 03,
    Unused = 04,
    Computer = 05,
    HumanOpenSlot = 06,
    Neutral = 07,
}

pub fn parse_controller(b: &[u8]) -> nom::IResult<&[u8], Controller> {
    map_opt(le_u8, FromPrimitive::from_u8)(b)
}

#[derive(Debug, Clone, Struple, Eq, PartialEq)]
pub struct Dimensions {
    pub width: u16,
    pub height: u16,
}

pub fn parse_dimensions(b: &[u8]) -> nom::IResult<&[u8], Dimensions> {
    map(tuple((le_u16, le_u16)), Dimensions::from_tuple)(b)
}

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

pub fn parse_side(b: &[u8]) -> nom::IResult<&[u8], Side> {
    map_opt(le_u8, FromPrimitive::from_u8)(b)
}

#[derive(Debug, Clone, Hash, Struple, Eq, PartialEq)]
pub struct MegaTile(u16);

impl MegaTile {
    pub const PIXEL_WIDTH: u32 = 32;
    pub const PIXEL_HEIGHT: u32 = 32;

    pub fn group_index(&self) -> usize {
        return ((self.0 >> 4) & 0x7ff) as usize;
    }

    pub fn subtile_index(&self) -> usize {
        return (self.0 & 0xf) as usize;
    }
}

pub fn parse_megatile(b: &[u8]) -> nom::IResult<&[u8], MegaTile> {
    map(le_u16, MegaTile)(b)
}

impl From<u16> for MegaTile {
    fn from(value: u16) -> Self {
        MegaTile(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringData(Vec<Vec<u8>>);

impl StringData {
    pub fn new(str_data: Vec<Vec<u8>>) -> StringData {
        StringData(str_data)
    }
}

pub fn parse_string_data(b: &[u8]) -> nom::IResult<&[u8], StringData> {
    let (remaining, str_count) = le_u16(b)?;
    let (_, str_offsets) = count(le_u16, str_count as usize)(remaining)?;

    // number of bytes of this chunk.
    let mut size = 0;

    let mut str_data = vec![];
    for offset in str_offsets {
        let (_, s) = preceded(take(offset), take_until("\0"))(b)?;
        size = max(size, offset as u32 + s.len() as u32);
        str_data.push(s.to_vec());
    }

    // jump over the last null terminator if there are strings
    if str_count > 0 {
        size += 1
    }

    let (remaining, _) = take(size)(b)?;

    Ok((remaining, StringData(str_data)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use spectral::prelude::*;
    use std::mem::size_of;

    use byteorder::{LittleEndian, WriteBytesExt};

    macro_rules! test_header {
        ($name:expr) => {{
            let mut rng = rand::thread_rng();
            let size = rng.gen_range(0, 100);
            let mut b: Vec<u8> = $name.as_bytes().into();
            b.write_u32::<LittleEndian>(size).unwrap();

            assert_that(&parse_header(&b))
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

    macro_rules! test_scenario_type {
        ($scenario_type:expr) => {{
            let mut b: Vec<u8> = vec![];
            b.write_u32::<LittleEndian>($scenario_type as u32).unwrap();

            assert_that(&parse_scenario_type(&b))
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

    macro_rules! test_file_format_version {
        ($version:expr) => {{
            let mut b: Vec<u8> = vec![];
            b.write_u16::<LittleEndian>($version as u16).unwrap();

            assert_that(&parse_file_format_version(&b))
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

    macro_rules! test_tileset {
        ($tileset:expr) => {{
            let mut b: Vec<u8> = vec![];
            b.write_u16::<LittleEndian>($tileset as u16).unwrap();

            assert_that(&parse_tileset(&b))
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

    macro_rules! test_controller {
        ($controller:expr) => {{
            let mut b: Vec<u8> = vec![];
            b.write_u8($controller as u8).unwrap();

            assert_that(&parse_controller(&b))
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

    #[test]
    fn it_parses_dimensions() {
        let mut rng = rand::thread_rng();
        let mut b: Vec<u8> = vec![];

        let width = rng.gen_range(0, 256);
        let height = rng.gen_range(0, 256);

        b.write_u16::<LittleEndian>(width).unwrap();
        b.write_u16::<LittleEndian>(height).unwrap();

        assert_that(&parse_dimensions(&b))
            .is_ok()
            .map(|(_, dimensions)| dimensions)
            .is_equal_to(Dimensions { width, height });
    }

    macro_rules! test_sides {
        ($side:expr) => {{
            let mut b: Vec<u8> = vec![];
            b.write_u8($side as u8).unwrap();

            assert_that(&parse_side(&b))
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

    #[test]
    fn it_parses_mega_tile() {
        let mut rng = rand::thread_rng();
        let mut b: Vec<u8> = vec![];
        let value = rng.gen_range(0, 256);
        b.write_u16::<LittleEndian>(value).unwrap();

        assert_that(&parse_megatile(&b))
            .is_ok()
            .map(|(_, megatile)| megatile)
            .is_equal_to(MegaTile(value));
    }

    #[test]
    fn it_parses_string_data() {
        let s1 = b"starcraft\0";
        let s2 = b"broodwar\0";

        let mut b: Vec<u8> = vec![];

        // string count
        b.write_u16::<LittleEndian>(2).unwrap();
        // s1 offset
        b.write_u16::<LittleEndian>((size_of::<u16>() * 3) as u16)
            .unwrap();
        // s2 offset
        b.write_u16::<LittleEndian>(((size_of::<u16>() * 3) + s1.len()) as u16)
            .unwrap();
        // write s1
        b.extend(s1);
        // write s2
        b.extend(s2);

        let expected_remaining_bytes: &[u8] = &[];
        let expected = (
            expected_remaining_bytes,
            StringData::new(
                // null terminator is removed
                vec![b"starcraft".to_vec(), b"broodwar".to_vec()],
            ),
        );

        assert_that(&parse_string_data(&b))
            .is_ok()
            .is_equal_to(expected);
    }
}
