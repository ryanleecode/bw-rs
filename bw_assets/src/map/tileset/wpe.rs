use nom::{
    combinator::{all_consuming, map},
    multi::many0,
    number::complete::le_u8,
    sequence::tuple,
    IResult,
};

use std::ops::Index;

use super::VR4;

#[derive(Debug)]
pub struct WPE(image::Rgba<u8>);

impl From<&WPE> for image::Rgba<u8> {
    fn from(wpe: &WPE) -> Self {
        wpe.0
    }
}

impl WPE {
    fn parse(b: &[u8]) -> IResult<&[u8], WPE> {
        map(tuple((le_u8, le_u8, le_u8, le_u8)), |(r, g, b, _)| {
            WPE(image::Rgba([r, g, b, 255]))
        })(b)
    }
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

impl WPEs {
    pub fn parse(b: &[u8]) -> IResult<&[u8], WPEs> {
        all_consuming(map(many0(WPE::parse), WPEs))(b)
    }
}
