use super::super::map::MegaTile;
use amethyst::{
    assets::{Asset, Format, Handle},
    ecs::DenseVecStorage,
};
use nom::{
    bytes::complete::take,
    combinator::{all_consuming, map},
    multi::{count, many0},
    number::complete::{le_u8, le_u16},
    sequence::{preceded, tuple},
};
use nom::{Finish, IResult};

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

#[derive(Debug)]
pub struct MinitileReference(u16);

impl From<MinitileReference> for usize {
    fn from(minitile_ref: MinitileReference) -> Self {
        usize::from(&minitile_ref)
    }
}

impl From<&MinitileReference> for usize {
    fn from(minitile_ref: &MinitileReference) -> Self {
        minitile_ref.0 as usize
    }
}

#[derive(Debug)]
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

impl
    From<(
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
    )> for CV5Data
{
    fn from(
        t: (
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
        ),
    ) -> Self {
        CV5Data(t.0, t.1, t.2, t.3, t.4, t.5, t.6, t.7, t.8, t.9, t.10)
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
        CV5Data::from,
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

fn parse_cv5s(b: &[u8]) -> IResult<&[u8], CV5s> {
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

pub struct CV5sAsset(Option<CV5s>);

impl CV5sAsset {
    pub fn take(&mut self) -> Option<CV5s> {
        self.0.take()
    }
}

pub type CV5sHandle = Handle<CV5sAsset>;

impl Asset for CV5sAsset {
    const NAME: &'static str = "bw_assets::tileset::CV5sAsset";
    type Data = Self;
    type HandleStorage = DenseVecStorage<CV5sHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CV5Format;

impl Format<CV5sAsset> for CV5Format {
    fn name(&self) -> &'static str {
        "CV5Format"
    }

    fn import_simple(&self, b: Vec<u8>) -> amethyst::Result<CV5sAsset> {
        let (_, cv5s) = parse_cv5s(&b).finish().map_err(|err| {
            amethyst::error::format_err!(
                "failed to load cv5 asset: {} at position {}",
                err.code.description(),
                b.len() - err.input.len()
            )
        })?;

        Ok(CV5sAsset(Some(cv5s)))
    }
}
