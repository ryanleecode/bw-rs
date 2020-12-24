use crate::make_pointer;
use nom::{
    bytes::complete::{take, take_until},
    combinator::all_consuming,
    multi::length_count,
    number::complete::le_u16,
    sequence::preceded,
    IResult,
};
use std::ops::Deref;

make_pointer!(ImagesTblPointer, u32);

#[derive(Debug)]
pub struct ImagesTbl(Vec<String>);

impl Deref for ImagesTbl {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ImagesTbl {
    pub fn parse(b: &[u8]) -> IResult<&[u8], ImagesTbl> {
        let (_, offsets) = length_count(le_u16, le_u16)(b)?;

        let mut str_data: Vec<String> = Vec::with_capacity(offsets.len());

        for offset in offsets {
            let (_, slice) = preceded(take(offset), take_until("\0"))(b)?;
            let s = String::from_utf8_lossy(slice);
            str_data.push(s.into_owned());
        }

        let (remaining, _) = all_consuming(take(b.len()))(b)?;

        Ok((remaining, ImagesTbl(str_data)))
    }
}
