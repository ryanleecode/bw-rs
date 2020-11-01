use amethyst::{
    assets::{Asset, Format, Handle},
    ecs::DenseVecStorage,
};
use nom::IResult;
use nom::{
    combinator::{all_consuming, map},
    multi::many0,
    number::complete::le_u8,
    sequence::tuple,
};

use std::ops::Index;

use super::VR4;

/// 256-color RGB Palette.
#[derive(Debug)]
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

#[derive(Debug)]
pub struct WPEs(Vec<WPE>);

impl Index<VR4> for WPEs {
    type Output = WPE;

    fn index(&self, vr4: VR4) -> &Self::Output {
        &self.index(&vr4)
    }
}

impl Index<&VR4> for WPEs {
    type Output = WPE;

    fn index(&self, vr4: &VR4) -> &Self::Output {
        &self.0[usize::from(vr4) as usize]
    }
}

fn parse_wpes(b: &[u8]) -> IResult<&[u8], WPEs> {
    all_consuming(map(many0(parse_wpe), WPEs))(b)
}

#[derive(Debug)]
pub struct WPEsAsset(Option<WPEs>);

impl WPEsAsset {
    pub fn take(&mut self) -> Option<WPEs> {
        self.0.take()
    }
}

pub type WPEsHandle = Handle<WPEsAsset>;

impl Asset for WPEsAsset {
    const NAME: &'static str = "bw_assets::tileset::WPEsAsset";
    type Data = Self;
    type HandleStorage = DenseVecStorage<WPEsHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WPEFormat;

impl Format<WPEsAsset> for WPEFormat {
    fn name(&self) -> &'static str {
        "WPEFormat"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> amethyst::Result<WPEsAsset> {
        let (_, wpes) = parse_wpes(&bytes).map_err(|err| err.to_owned())?;

        Ok(WPEsAsset(Some(wpes)))
    }
}
