use nom::{
    self,
    bytes::complete::take,
    combinator::{all_consuming, map},
    multi::{count, many0},
};

use std::ops::Deref;

use std::mem;

mod chunk_name;
mod controller;
mod dimensions;
mod file_format_version;
mod header;
mod mega_tile;
mod scenario_type;
mod side;
mod string_data;
mod tileset;
mod unit;

pub use chunk_name::*;
pub use controller::*;
pub use dimensions::*;
pub use file_format_version::*;
pub use header::*;
pub use mega_tile::*;
pub use scenario_type::*;
pub use side::*;
pub use string_data::*;
pub use tileset::*;
pub use unit::*;

use crate::make_pointer;

make_pointer!(Owner, u8);

#[derive(Debug)]
pub enum Chunk {
    ScenarioType(ScenarioType),
    FileFormatVersion(FileFormatVersion),
    Tileset(Tileset),
    Controllers(Controllers),
    Dimensions(Dimensions),
    Sides(Vec<Side>),
    MegaTiles(Vec<MegaTile>),
    Units(Vec<Unit>),
    StringData(StringData),
    Unknown,
}

impl Chunk {
    fn parse(b: &[u8]) -> nom::IResult<&[u8], Chunk> {
        let (remaining, header) = Header::parse(&b)?;

        match header.name {
            ChunkName::Type => map(ScenarioType::parse, Chunk::ScenarioType)(remaining),
            ChunkName::Version => {
                map(FileFormatVersion::parse, Chunk::FileFormatVersion)(remaining)
            }
            ChunkName::Tileset => map(Tileset::parse, Chunk::Tileset)(remaining),
            ChunkName::Controllers => {
                let size = header.size as usize / mem::size_of::<Controller>();
                map(
                    map(count(Controller::parse, size), Controllers),
                    Chunk::Controllers,
                )(remaining)
            }
            ChunkName::Dimensions => map(Dimensions::parse, Chunk::Dimensions)(remaining),
            ChunkName::Side => {
                let size = header.size as usize / mem::size_of::<u8>();
                map(count(Side::parse, size), Chunk::Sides)(remaining)
            }
            ChunkName::MegaTiles => {
                let size = header.size as usize / mem::size_of::<MegaTile>();
                map(count(MegaTile::parse, size), Chunk::MegaTiles)(remaining)
            }
            ChunkName::Unit => {
                const UNIT_BYTE_SIZE: usize = 36;
                let size = header.size as usize / UNIT_BYTE_SIZE;
                map(count(Unit::parse, size), Chunk::Units)(remaining)
            }
            ChunkName::StringData => map(StringData::parse, Chunk::StringData)(remaining),
            _ => map(take(header.size), |_| Chunk::Unknown)(remaining),
        }
    }
}

pub struct Chunks(Vec<Chunk>);

impl Deref for Chunks {
    type Target = Vec<Chunk>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Chunks {
    pub(super) fn parse(b: &[u8]) -> nom::IResult<&[u8], Vec<Chunk>> {
        all_consuming(many0(Chunk::parse))(b)
    }
}
