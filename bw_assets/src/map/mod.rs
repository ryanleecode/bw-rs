//! Asset format for SCX and SCM Starcraft map formats

use anyhow::Result;
use nom::Finish;

pub use chk::*;
pub use tileset::*;

mod chk;
mod tileset;

/// Every Starcraft map will have this file.
const MAP_FILE_NAME: &str = "staredit\\scenario.chk";

pub const MINITILE_PX_SIDE_LEN: u32 = 8;
pub const MEGATILE_SIDE_LEN: u32 = 4;
pub const MEGATILE_PX_SIDE_LEN: u32 = MINITILE_PX_SIDE_LEN * MEGATILE_SIDE_LEN;

/// Max megatile width
pub const MAX_WIDTH: u32 = 256;

/// Max Megatile height
pub const MAX_HEIGHT: u32 = 256;

#[builder(private, pattern = "owned")]
#[derive(Debug, Builder)]
pub struct Map {
    pub scenario_type: Option<ScenarioType>,
    pub file_format_version: FileFormatVersion,
    pub tileset: Tileset,
    pub controllers: Controllers,
    pub dimensions: Dimensions,
    pub sides: Vec<Side>,
    pub megatiles: Vec<MegaTile>,
    pub placed_units: Vec<Unit>,
    pub string_data: StringData,
}

impl Map {
    pub fn pixel_width(&self) -> u32 {
        self.tile_width() * MEGATILE_PX_SIDE_LEN
    }

    pub fn tile_width(&self) -> u32 {
        self.dimensions.width as u32
    }

    pub fn tile_height(&self) -> u32 {
        self.dimensions.height as u32
    }

    pub fn pixel_height(&self) -> u32 {
        self.tile_height() * MEGATILE_PX_SIDE_LEN
    }

    pub fn parse(b: &[u8]) -> Result<Map> {
        use std::io::Cursor;

        let cursor = Cursor::new(b);

        // A Starcraft map is just a regular MPQ archive with a single file inside.
        let archive = ceres_mpq::Archive::open(cursor)?;

        // The Starcraft map format is divided into chunks denoted by the "chk"
        // format.
        // see: http://www.starcraftai.com/wiki/CHK_Format
        let chunk_bytes = archive.read_file(MAP_FILE_NAME)?;

        let mut map_builder = MapBuilder::default();

        let (_, chunks) = chk::Chunks::parse(&chunk_bytes).finish().map_err(|err| {
            anyhow::format_err!(
                "failed to load chunks: {} at position {}",
                err.code.description(),
                chunk_bytes.len() - err.input.len()
            )
        })?;

        for chunk in chunks.into_iter() {
            match chunk {
                chk::Chunk::ScenarioType(scenario_type) => {
                    map_builder = map_builder.scenario_type(Some(scenario_type));
                }
                chk::Chunk::FileFormatVersion(file_format_version) => {
                    map_builder = map_builder.file_format_version(file_format_version);
                }
                chk::Chunk::Tileset(tileset) => {
                    map_builder = map_builder.tileset(tileset);
                }
                chk::Chunk::Controllers(controllers) => {
                    map_builder = map_builder.controllers(controllers);
                }
                chk::Chunk::Dimensions(dimensions) => {
                    map_builder = map_builder.dimensions(dimensions);
                }
                chk::Chunk::Sides(sides) => {
                    map_builder = map_builder.sides(sides);
                }
                chk::Chunk::MegaTiles(megatiles) => {
                    map_builder = map_builder.megatiles(megatiles);
                }
                chk::Chunk::Units(units) => {
                    map_builder = map_builder.placed_units(units);
                }
                chk::Chunk::StringData(string_data) => {
                    map_builder = map_builder.string_data(string_data);
                }
                _ => {}
            }
        }

        let map = map_builder
            .build()
            .map_err(|s| anyhow::format_err!("Map is missing required components: {}", s))?;

        Ok(map)
    }
}
