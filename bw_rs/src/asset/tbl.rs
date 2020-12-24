use crate::{make_load_labeled_asset_fn, make_pointer};
use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use nom::{
    bytes::complete::{take, take_until},
    combinator::all_consuming,
    multi::length_count,
    number::complete::le_u16,
    sequence::preceded,
    Finish, IResult,
};
use regex::Regex;
use std::ops::Deref;

pub mod labels {
    pub const IMAGES: &str = "images";
}

make_pointer!(ImagesTblPointer, u32);

#[derive(Default)]
pub struct TblAssetLoader;

impl AssetLoader for TblAssetLoader {
    fn load<'a>(
        &'a self,
        b: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            lazy_static! {
                static ref IMAGE_TBL_RE: Regex = Regex::new(r"images\.tbl$").unwrap();
            }

            make_load_labeled_asset_fn!(b, load_context);

            let path = load_context.path().to_string_lossy();
            if IMAGE_TBL_RE.is_match(&path) {
                load_labeled_asset!(labels::IMAGES, parse_images_tbl);
            }

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tbl"]
    }
}

#[derive(Debug, TypeUuid)]
#[uuid = "50afde69-135b-4769-b2e1-5bbd392884d3"]
pub struct ImagesTbl(Vec<String>);

impl Deref for ImagesTbl {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn parse_images_tbl(b: &[u8]) -> IResult<&[u8], ImagesTbl> {
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
