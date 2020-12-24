use crate::{make_pointer, map::MegaTile};

use nom::{
    bytes::complete::take,
    combinator::{all_consuming, map},
    multi::{count, many0},
    number::complete::{le_u8, le_u16},
    sequence::{preceded, tuple},
    IResult,
};
use struple::Struple;

use std::ops::Index;

pub enum BuildFlag {
    Buildable,
    Creep,
    Unbuildable,
}

#[derive(Debug)]
pub struct TileMetadata(CV5Data);

impl TileMetadata {
    pub fn build_flag(&self) -> BuildFlag {
        self.0.build_flag()
    }
}

pub enum OverlayFlag {
    None,
    // Reference to a sprite in a Sprites.dat file
    SpriteReference,
    /// Reference to a unit in a Units.dat file
    UnitReference,
    Flipped,
}

#[derive(Debug)]
pub struct Doodad(CV5Data);

impl Doodad {
    pub fn build_flag(&self) -> BuildFlag {
        self.0.build_flag()
    }

    pub fn overlay_flags(&self) -> OverlayFlag {
        match self.0 .1 >> 4 {
            0x0 => OverlayFlag::None,
            0x1 => OverlayFlag::SpriteReference,
            0x2 => OverlayFlag::UnitReference,
            0x4 => OverlayFlag::Flipped,
            _ => OverlayFlag::None,
        }
    }

    pub fn overlay_id(&self) -> u16 {
        self.0 .2
    }

    pub fn doodad_group_str_idx(&self) -> u16 {
        self.0 .4
    }

    pub fn dddata_bin_idx(&self) -> u16 {
        self.0 .6
    }

    pub fn width(&self) -> u16 {
        self.0 .7
    }

    pub fn height(&self) -> u16 {
        self.0 .8
    }
}

make_pointer!(MinitileReference, u16);

#[derive(Debug, Struple)]
pub struct CV5Data(
    u8,
    u8,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    u16,
    Vec<MinitileReference>,
);

impl CV5Data {
    pub fn megatile_references(&self) -> &Vec<MinitileReference> {
        &self.10
    }
}

impl Index<usize> for CV5Data {
    type Output = MinitileReference;

    fn index(&self, i: usize) -> &Self::Output {
        &self.10[i]
    }
}

impl CV5Data {
    /// Each megatile has 16 (4x4) minitiles.
    const MEGA_TILE_REFERENCE_COUNT: usize = 16;

    pub fn build_flag(&self) -> BuildFlag {
        match self.0 >> 4 {
            0 => BuildFlag::Buildable,
            4 => BuildFlag::Creep,
            8 => BuildFlag::Unbuildable,
            _ => BuildFlag::Buildable,
        }
    }

    pub fn as_doodad(self) -> Doodad {
        Doodad(self)
    }

    pub fn as_tile_metadata(self) -> TileMetadata {
        TileMetadata(self)
    }
}

impl Index<MegaTile> for CV5 {
    type Output = MinitileReference;

    fn index(&self, megatile: MegaTile) -> &Self::Output {
        self.index(&megatile)
    }
}

impl Index<&MegaTile> for CV5 {
    type Output = MinitileReference;

    fn index(&self, megatile: &MegaTile) -> &Self::Output {
        match &self {
            CV5::Doodad(doodad) => &doodad.0[megatile.subtile_index()],
            CV5::TileMetadata(tile_metadata) => &tile_metadata.0[megatile.subtile_index()],
        }
    }
}

fn parse_cv5(b: &[u8]) -> IResult<&[u8], CV5Data> {
    map(
        tuple((
            preceded(take(2u8), le_u8),
            le_u8,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            le_u16,
            count(
                map(le_u16, MinitileReference),
                CV5Data::MEGA_TILE_REFERENCE_COUNT,
            ),
        )),
        CV5Data::from_tuple,
    )(b)
}

#[derive(Debug)]
pub enum CV5 {
    Doodad(Doodad),
    TileMetadata(TileMetadata),
}

/// A list of CV5. Each CV5 is referenced by the MXTM field from CHK.
#[derive(Debug)]
pub struct CV5s(Vec<CV5>);

impl Index<MegaTile> for CV5s {
    type Output = CV5;

    fn index(&self, megatile: MegaTile) -> &Self::Output {
        self.index(&megatile)
    }
}

impl Index<&MegaTile> for CV5s {
    type Output = CV5;

    fn index(&self, megatile: &MegaTile) -> &Self::Output {
        &self.0[megatile.group_index()]
    }
}

impl CV5s {
    pub fn parse(b: &[u8]) -> IResult<&[u8], CV5s> {
        let (remaining, cv5s_data) = all_consuming(many0(parse_cv5))(b)?;

        let cv5s = cv5s_data
            .into_iter()
            .enumerate()
            .map(|(i, cv5_data)| {
                if i < 1024 {
                    CV5::TileMetadata(cv5_data.as_tile_metadata())
                } else {
                    CV5::Doodad(cv5_data.as_doodad())
                }
            })
            .collect::<Vec<_>>();

        Ok((remaining, CV5s(cv5s)))
    }
}
