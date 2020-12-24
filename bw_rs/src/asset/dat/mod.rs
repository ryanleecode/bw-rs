use anyhow::{bail, Result};
use bevy::asset::LoadedAsset;
use bevy::{
    asset::{AssetLoader, LoadContext},
    utils::BoxedFuture,
};

use nom::Finish;
use regex::Regex;

pub use self::image::*;
pub use flingy::*;
pub use sprite::*;
pub use tech_data::*;
pub use unit::*;
pub use upgrade::*;
pub use weapon::*;

use crate::make_load_labeled_asset_fn;

mod flingy;
mod image;
mod sprite;
mod tech_data;
mod unit;
mod upgrade;
mod weapon;

pub mod labels {
    pub const FLINGY: &str = "flingy";
    pub const SPRITES: &str = "sprites";
    pub const IMAGES: &str = "images";
    pub const TECH_DATA: &str = "tech_data";
    pub const UPGRADES: &str = "upgrades";
    pub const WEAPONS: &str = "weapons";
}

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! count_total {
        ($block_size:ident) => {
            fn count_total<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
            where
                I: Clone + PartialEq,
                F: Parser<I, O, E>,
                E: ParseError<I>,
            {
                count(f, $block_size)
            }
        };
    }

    #[macro_export]
    macro_rules! make_dat {
        ($name:ident, $type:ty, $index_type:ty, $uuid:literal) => {
            #[derive(Debug, TypeUuid)]
            #[uuid = $uuid]
            pub struct $name(Vec<$type>);

            impl Index<$index_type> for $name {
                type Output = $type;

                fn index(&self, id: $index_type) -> &Self::Output {
                    self.index(&id)
                }
            }

            impl Index<&$index_type> for $name {
                type Output = $type;

                fn index(&self, id: &$index_type) -> &Self::Output {
                    &self.0[usize::from(id)]
                }
            }
        };
    }
}

#[derive(Default)]
pub struct DatAssetLoader;

impl AssetLoader for DatAssetLoader {
    fn load<'a>(
        &'a self,
        b: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            lazy_static! {
                static ref FLINGY_RE: Regex = Regex::new(r"flingy\.dat$").unwrap();
                static ref SPRITES_RE: Regex = Regex::new(r"sprites\.dat$").unwrap();
                static ref IMAGES_RE: Regex = Regex::new(r"images\.dat$").unwrap();
                static ref TECH_DATA_RE: Regex = Regex::new(r"techdata\.dat$").unwrap();
                static ref UPGRADES_RE: Regex = Regex::new(r"upgrades\.dat$").unwrap();
                static ref WEAPONS_RE: Regex = Regex::new(r"weapons\.dat$").unwrap();
            }

            make_load_labeled_asset_fn!(b, load_context);

            let path = load_context.path().to_string_lossy();
            if FLINGY_RE.is_match(&path) {
                load_labeled_asset!(labels::FLINGY, parse_flingy_dat);
            } else if SPRITES_RE.is_match(&path) {
                load_labeled_asset!(labels::SPRITES, parse_sprites_dat);
            } else if IMAGES_RE.is_match(&path) {
                load_labeled_asset!(labels::IMAGES, parse_images_dat);
            } else if TECH_DATA_RE.is_match(&path) {
                load_labeled_asset!(labels::TECH_DATA, parse_tech_data_dat);
            } else if UPGRADES_RE.is_match(&path) {
                load_labeled_asset!(labels::UPGRADES, parse_upgrades_dat);
            } else if WEAPONS_RE.is_match(&path) {
                load_labeled_asset!(labels::WEAPONS, parse_weapons_dat);
            } else {
                bail!(format!("no asset loader for {}", path))
            }

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["dat"]
    }
}
