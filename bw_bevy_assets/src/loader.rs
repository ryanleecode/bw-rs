use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    utils::BoxedFuture,
};
use nom::Finish;

use bevy::reflect::TypeUuid;
use bw_assets::map::{
    CV5s as InnerCV5s, Map as InnerMap, VF4s as InnerVF4s, VR4s as InnerVR4s, VX4s as InnerVX4s,
    WPEs as InnerWPEs,
};
use std::ops::Deref;

macro_rules! wrap_asset {
    ($name:ident, $inner:ident, $uuid:literal) => {
        #[derive(Debug, TypeUuid)]
        #[uuid = $uuid]
        pub(crate) struct $name($inner);

        impl Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

wrap_asset!(WPEs, InnerWPEs, "2d209648-ec13-4d32-b8bf-38122826ddec");
wrap_asset!(VX4s, InnerVX4s, "2a1237e3-97e1-4195-a864-cd0c453c4d0b");
wrap_asset!(VR4s, InnerVR4s, "b1e9d9ec-5a07-4f8c-958d-ac999c0c226e");
wrap_asset!(VF4s, InnerVF4s, "3947ea31-0fe2-4a00-b0f3-5ed0300ccdb7");
wrap_asset!(CV5s, InnerCV5s, "9909cfc6-744a-456b-8689-43a6061bea46");
wrap_asset!(Map, InnerMap, "72259588-2061-42ae-8c4c-8d5c06a8e09b");

macro_rules! make_asset_loader {
    ($name:ident, $parser:ident, $wrapper:ident, $ext:literal) => {
        #[derive(Default)]
        pub(crate) struct $name;

        impl AssetLoader for $name {
            fn load<'a>(
                &'a self,
                bytes: &'a [u8],
                load_context: &'a mut LoadContext,
            ) -> BoxedFuture<'a, Result<()>> {
                Box::pin(async move {
                    let asset = $wrapper(
                        $parser::parse(bytes)
                            .finish()
                            .map(|(_, asset)| asset)
                            .map_err(|err| {
                                anyhow::format_err!(
                                    "failed to load {} asset: {} at position {}",
                                    $ext,
                                    err.code.description(),
                                    bytes.len() - err.input.len()
                                )
                            })?,
                    );

                    load_context.set_default_asset(LoadedAsset::new(asset));

                    Ok(())
                })
            }

            fn extensions(&self) -> &[&str] {
                &[$ext]
            }
        }
    };
}

make_asset_loader!(WPELoader, InnerWPEs, WPEs, "wpe");
make_asset_loader!(VX4Loader, InnerVX4s, VX4s, "vx4");
make_asset_loader!(VR4Loader, InnerVR4s, VR4s, "vr4");
make_asset_loader!(VF4Loader, InnerVF4s, VF4s, "vf4");
make_asset_loader!(CV5Loader, InnerCV5s, CV5s, "cv5");

#[derive(Default)]
pub struct MapAssetLoader;

impl AssetLoader for MapAssetLoader {
    fn load<'a>(
        &'a self,
        b: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<Result<()>> {
        Box::pin(async move {
            let map = Map(InnerMap::parse(b)?);

            load_context.set_default_asset(LoadedAsset::new(map));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["scx", "scm"]
    }
}
