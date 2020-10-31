//! Types and Parsers for the Tileset File Formats
//!
//! Blizzard heavily optimized for space when they designed these formats.
//! Starcraft maps have an MXTM fields which reference at CV5 element. Each CV5
//! element represents a megatile (32x32 pixels) and references 16 8x8 minitiles
//! via VF4 and VX4 elements. VX4s indicate if the tile is flipped horizontally
//! and is also reference VR4. Each VR4 is a reference to 64 WPEs which represent
//! the color of the pixel. VF4 on the other hand show the gameplay flags such as
//! walkable, elevation, blocks view, etc...

use super::map::MegaTile;
use amethyst::{
    assets::{Asset, Format, Handle},
    ecs::DenseVecStorage,
};
use nom::IResult;
use nom::{
    bytes::complete::take,
    combinator::{all_consuming, map},
    multi::{count, many0},
    number::complete::{le_u8, le_u16},
    sequence::{preceded, tuple},
};

use rayon::prelude::*;
use std::ops::Index;
use std::sync::Arc;

// -----------------------------------------------------------------------------
//  CV5
// -----------------------------------------------------------------------------

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
#[derive(Debug, Clone)]
pub struct CV5s(Arc<Vec<CV5>>);

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

    Ok((remaining, CV5s(Arc::new(cv5s))))
}

pub type CV5sHandle = Handle<CV5s>;

impl Asset for CV5s {
    const NAME: &'static str = "bw_assets::tileset::CV5s";
    type Data = Self;
    type HandleStorage = DenseVecStorage<CV5sHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CV5Format;

impl Format<CV5s> for CV5Format {
    fn name(&self) -> &'static str {
        "CV5Format"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> amethyst::Result<CV5s> {
        let (_, cv5s) = parse_cv5s(&bytes).map_err(|err| err.to_owned())?;

        Ok(cv5s)
    }
}

// -----------------------------------------------------------------------------
//  VX4
// -----------------------------------------------------------------------------

/// Mini-tile image pointer. Referenced by CV5.
///
/// Bit 0 will indicate if the tile is flipped, and the 7 high bits is the
/// index to the VR4 asset.
#[derive(Debug)]
pub struct VX4(u16);

impl VX4 {
    pub fn is_horizontally_flipped(&self) -> bool {
        return self.0 & 1 == 1;
    }

    pub fn index(&self) -> usize {
        return (self.0 >> 1) as usize;
    }
}

fn parse_vx4(b: &[u8]) -> IResult<&[u8], VX4> {
    map(le_u16, VX4)(b)
}

#[derive(Debug, Clone)]
pub struct VX4s(Arc<Vec<Vec<VX4>>>);

impl VX4s {
    /// Each megatile has 16 (4x4) minitiles.
    pub const BLOCK_SIZE: usize = 16;

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<MinitileReference> for VX4s {
    type Output = Vec<VX4>;

    fn index(&self, megatile_reference: MinitileReference) -> &Self::Output {
        &self.index(&megatile_reference)
    }
}

impl Index<&MinitileReference> for VX4s {
    type Output = Vec<VX4>;

    fn index(&self, megatile_reference: &MinitileReference) -> &Self::Output {
        &self.0[megatile_reference.0 as usize]
    }
}

fn parse_vx4s(b: &[u8]) -> IResult<&[u8], VX4s> {
    all_consuming(map(
        map(many0(count(parse_vx4, VX4s::BLOCK_SIZE)), Arc::new),
        VX4s,
    ))(b)
}

pub type VX4sHandle = Handle<VX4s>;

impl Asset for VX4s {
    const NAME: &'static str = "bw_assets::tileset::VX4s";
    type Data = Self;
    type HandleStorage = DenseVecStorage<VX4sHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VX4Format;

impl Format<VX4s> for VX4Format {
    fn name(&self) -> &'static str {
        "VX4Format"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> amethyst::Result<VX4s> {
        let (_, vx4s) = parse_vx4s(&bytes).map_err(|err| err.to_owned())?;

        Ok(vx4s)
    }
}

// -----------------------------------------------------------------------------
//  VF4
// -----------------------------------------------------------------------------

/// MiniTile graphic references for each MegaTile. Referenced by CV5.
#[derive(Debug)]
pub struct VF4(u16);

#[derive(Debug, Clone)]
pub struct VF4s(Arc<Vec<Vec<VF4>>>);

impl Index<MinitileReference> for VF4s {
    type Output = Vec<VF4>;

    fn index(&self, megatile_reference: MinitileReference) -> &Self::Output {
        &self.index(&megatile_reference)
    }
}

impl Index<&MinitileReference> for VF4s {
    type Output = Vec<VF4>;

    fn index(&self, megatile_reference: &MinitileReference) -> &Self::Output {
        &self.0[megatile_reference.0 as usize]
    }
}

impl VF4 {
    // http://www.staredit.net/wiki/index.php?title=Terrain_Format#VF4

    const WALKABLE: u16 = 0x0001;
    const MID: u16 = 0x0002;
    const HIGH: u16 = 0x0004;
    const LOW: u16 = 0x0004 | 0x0002;
    const BLOCKS_VIEW: u16 = 0x0008;
    const RAMP: u16 = 0x0010;

    pub fn is_walkable(&self) -> bool {
        return self.0 & VF4::WALKABLE == VF4::WALKABLE;
    }

    pub fn is_elevation_mid(&self) -> bool {
        return self.0 & VF4::MID == VF4::MID;
    }

    pub fn is_elevation_high(&self) -> bool {
        return self.0 & VF4::HIGH == VF4::HIGH;
    }

    pub fn is_elevation_low(&self) -> bool {
        return self.0 & VF4::LOW == VF4::LOW;
    }

    pub fn blocks_view(&self) -> bool {
        return self.0 & VF4::BLOCKS_VIEW == VF4::BLOCKS_VIEW;
    }

    pub fn is_ramp(&self) -> bool {
        return self.0 & VF4::RAMP == VF4::RAMP;
    }
}

impl VF4s {
    /// Each megatile has 16 (4x4) minitiles.
    const BLOCK_SIZE: usize = 16;
}

fn parse_vf4s(b: &[u8]) -> IResult<&[u8], VF4s> {
    all_consuming(map(
        map(many0(count(map(le_u16, VF4), VF4s::BLOCK_SIZE)), Arc::new),
        VF4s,
    ))(b)
}

pub type VF4sHandle = Handle<VF4s>;

impl Asset for VF4s {
    const NAME: &'static str = "bw_assets::tileset::VF4s";
    type Data = Self;
    type HandleStorage = DenseVecStorage<VF4sHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VF4Format;

impl Format<VF4s> for VF4Format {
    fn name(&self) -> &'static str {
        "VF4Format"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> amethyst::Result<VF4s> {
        let (_, vf4s) = parse_vf4s(&bytes).map_err(|err| err.to_owned())?;

        Ok(vf4s)
    }
}

// -----------------------------------------------------------------------------
//  VR4
// -----------------------------------------------------------------------------

/// Index to WPE (pixel color)
#[derive(Debug)]
pub struct VR4(u8);

fn parse_vr4(b: &[u8]) -> IResult<&[u8], VR4> {
    map(le_u8, VR4)(b)
}

#[derive(Debug, Clone)]
pub struct VR4s(pub Arc<Vec<Vec<VR4>>>);

impl VR4s {
    /// Each minitile has a side length of 8 pixels
    pub const MINITILE_SIDE_LENGTH: usize = 8;
    /// 8x8 = 64 pixels
    pub const BLOCK_SIZE: usize = 64;

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> VR4sIterator {
        VR4sIterator(self.0.iter())
    }

    pub fn par_iter(&self) -> rayon::slice::Iter<Vec<VR4>> {
        self.0.par_iter()
    }
}

impl Index<VX4> for VR4s {
    type Output = Vec<VR4>;

    fn index(&self, vx4: VX4) -> &Self::Output {
        &self.index(&vx4)
    }
}

impl Index<&VX4> for VR4s {
    type Output = Vec<VR4>;

    fn index(&self, vx4: &VX4) -> &Self::Output {
        &self.0[vx4.index()]
    }
}

pub struct VR4sIterator<'a>(std::slice::Iter<'a, Vec<VR4>>);

impl<'a> Iterator for VR4sIterator<'a> {
    type Item = &'a Vec<VR4>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

fn parse_vr4s(b: &[u8]) -> IResult<&[u8], VR4s> {
    all_consuming(map(
        map(many0(count(parse_vr4, VR4s::BLOCK_SIZE)), Arc::new),
        VR4s,
    ))(b)
}

pub type VR4sHandle = Handle<VR4s>;

impl Asset for VR4s {
    const NAME: &'static str = "bw_assets:tileset::VR4s";
    type Data = Self;
    type HandleStorage = DenseVecStorage<VR4sHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VR4Format;

impl Format<VR4s> for VR4Format {
    fn name(&self) -> &'static str {
        "VR4Format"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> amethyst::Result<VR4s> {
        let (_, vr4s) = parse_vr4s(&bytes).map_err(|err| err.to_owned())?;

        Ok(vr4s)
    }
}

// -----------------------------------------------------------------------------
//  WPE
// -----------------------------------------------------------------------------

/// 256-color RGB Palette.
#[derive(Debug, Clone)]
pub struct WPE([u8; WPE::BLOCK_SIZE]);

/// Gamma correction function
///
/// see: https://www.cambridgeincolour.com/tutorials/gamma-correction.htm
fn srgb(x: u8) -> f32 {
    (x as f32).powf(1.0 / 2.2)
}

impl WPE {
    const BLOCK_SIZE: usize = 3;

    pub fn r(&self) -> u8 {
        self.0[0]
    }

    pub fn g(&self) -> u8 {
        self.0[1]
    }

    pub fn b(&self) -> u8 {
        self.0[2]
    }

    /// Raw rgb values without gamma correction
    pub fn rgb(&self) -> [u8; 3] {
        [self.0[0], self.0[1], self.0[2]]
    }

    /// Color in srgb after gamma correction
    pub fn srgb(&self) -> [f32; 3] {
        [srgb(self.0[0]), srgb(self.0[1]), srgb(self.0[2])]
    }
}

fn parse_wpe(b: &[u8]) -> IResult<&[u8], WPE> {
    map(tuple((le_u8, le_u8, le_u8, le_u8)), |(r, g, b, _)| {
        WPE([r, g, b])
    })(b)
}

#[derive(Debug, Clone)]
pub struct WPEs(Arc<Vec<WPE>>);

impl Index<VR4> for WPEs {
    type Output = WPE;

    fn index(&self, vr4: VR4) -> &Self::Output {
        &self.index(&vr4)
    }
}

impl Index<&VR4> for WPEs {
    type Output = WPE;

    fn index(&self, vr4: &VR4) -> &Self::Output {
        &self.0[vr4.0 as usize]
    }
}

fn parse_wpes(b: &[u8]) -> IResult<&[u8], WPEs> {
    all_consuming(map(map(many0(parse_wpe), Arc::new), WPEs))(b)
}

pub type WPEsHandle = Handle<WPEs>;

impl Asset for WPEs {
    const NAME: &'static str = "bw_assets::tileset::WPEs";
    type Data = Self;
    type HandleStorage = DenseVecStorage<WPEsHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WPEFormat;

impl Format<WPEs> for WPEFormat {
    fn name(&self) -> &'static str {
        "WPEFormat"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> amethyst::Result<WPEs> {
        let (_, wpes) = parse_wpes(&bytes).map_err(|err| err.to_owned())?;

        Ok(wpes)
    }
}
