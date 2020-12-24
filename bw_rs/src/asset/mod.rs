use bevy::{
    asset::{filesystem_watcher_system, free_unused_assets_system, FileAssetIo, HandleId},
    prelude::*,
    tasks::IoTaskPool,
};
use dat::UpgradesDat;

use self::{
    dat::WeaponsDat,
    io::{MPQAssetIO, UnifiedMPQAssetIO},
    tbl::TblAssetLoader,
};

pub use dat::{
    DatAssetLoader, Flingy, FlingyDat, Image, ImagePointer, ImagesDat, Sprite, SpritePointer,
    SpritesDat, TechData, TechDataDat, Unit, UnitId, UnitsDat,
};
pub use tbl::{ImagesTbl, ImagesTblPointer};

mod dat;
mod io;
mod tbl;

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! make_pointer {
        ($name:ident, $type:ty) => {
            #[derive(Clone, Debug, Eq, PartialEq)]
            pub struct $name(pub(in crate::asset) $type);

            impl From<$name> for usize {
                fn from(p: $name) -> Self {
                    usize::from(&p)
                }
            }

            impl From<&$name> for usize {
                fn from(p: &$name) -> Self {
                    p.0 as usize
                }
            }
        };
    }

    #[macro_export]
    macro_rules! make_load_labeled_asset_fn {
        ($b:ident, $load_context:ident) => {
            macro_rules! load_labeled_asset {
                ($label:expr, $f:ident) => {{
                    let asset = $f($b).finish().map(|(_, asset)| asset).map_err(|err| {
                        anyhow::format_err!(
                            "failed to load {} asset: {} at position {}",
                            $label,
                            err.code.description(),
                            $b.len() - err.input.len()
                        )
                    })?;
                    $load_context.set_labeled_asset($label, LoadedAsset::new(asset));
                }};
            }
        };
    }
}

pub mod labels {
    pub mod dat {
        use super::super::dat::labels;

        pub const FLINGY: &str = labels::FLINGY;
        pub const SPRITES: &str = labels::SPRITES;
        pub const IMAGES: &str = labels::IMAGES;
        pub const TECH_DATA: &str = labels::TECH_DATA;
        pub const UPGRADES: &str = labels::UPGRADES;
        pub const WEAPONS: &str = labels::WEAPONS;
    }

    pub mod tbl {
        use super::super::tbl::labels;

        pub const IMAGES: &str = labels::IMAGES;
    }
}

pub struct AssetsServerSettings {
    pub asset_folder: String,
    pub stardat_name: String,
    pub broodat_name: String,
    pub patch_rt_name: String,
}

impl Default for AssetsServerSettings {
    fn default() -> Self {
        Self {
            asset_folder: "assets".to_string(),
            stardat_name: "STARDAT.MPQ".to_string(),
            broodat_name: "BROODAT.MPQ".to_string(),
            patch_rt_name: "patch_rt.mpq".to_string(),
        }
    }
}

#[derive(Default)]
pub struct MPQAssetsPlugin;

impl Plugin for MPQAssetsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let task_pool = app
            .resources()
            .get::<IoTaskPool>()
            .expect("`IoTaskPool` resource not found.")
            .0
            .clone();


        let asset_server = {
            let AssetsServerSettings {
                asset_folder,
                stardat_name,
                broodat_name,
                patch_rt_name,
            } = &(*app
                .resources_mut()
                .get_or_insert_with(AssetsServerSettings::default));

            let fmt = |name: &str| format!("Failed to load {}", name);

            let mut source_io = UnifiedMPQAssetIO::new();

            for name in vec![stardat_name, broodat_name, patch_rt_name] {
                let dat = MPQAssetIO::new(&asset_folder, &name).expect(&fmt(&name));
                source_io.add_source(dat);
            }

            let file_asset_io = FileAssetIo::new(&asset_folder);
            source_io.add_source(file_asset_io);

            AssetServer::new(source_io, task_pool)
        };

        app.add_stage_before(
            stage::PRE_UPDATE,
            bevy::asset::stage::LOAD_ASSETS,
            SystemStage::parallel(),
        )
        .add_stage_after(
            stage::POST_UPDATE,
            bevy::asset::stage::ASSET_EVENTS,
            SystemStage::parallel(),
        )
        .add_resource(asset_server)
        .add_asset::<FlingyDat>()
        .add_asset::<SpritesDat>()
        .add_asset::<ImagesDat>()
        .add_asset::<TechDataDat>()
        .add_asset::<UpgradesDat>()
        .add_asset::<WeaponsDat>()
        .add_asset::<ImagesTbl>()
        .init_asset_loader::<DatAssetLoader>()
        .init_asset_loader::<TblAssetLoader>()
        .register_type::<HandleId>()
        .add_system_to_stage(stage::PRE_UPDATE, free_unused_assets_system.system())
        .add_startup_system(setup.system());

        app.add_system_to_stage(
            bevy::asset::stage::LOAD_ASSETS,
            filesystem_watcher_system.system(),
        );

        app.add_system_to_stage(
            bevy::asset::stage::ASSET_EVENTS,
            flingy_dat_asset_event_listener.system(),
        );

        app.add_system_to_stage(
            bevy::asset::stage::ASSET_EVENTS,
            sprites_dat_asset_event_listener.system(),
        );

        app.add_system_to_stage(
            bevy::asset::stage::ASSET_EVENTS,
            images_dat_asset_event_listener.system(),
        );

        app.add_system_to_stage(
            bevy::asset::stage::ASSET_EVENTS,
            tech_data_dat_asset_event_listener.system(),
        );

        app.add_system_to_stage(
            bevy::asset::stage::ASSET_EVENTS,
            upgrades_dat_asset_event_listener.system(),
        );

        app.add_system_to_stage(
            bevy::asset::stage::ASSET_EVENTS,
            weapons_dat_asset_event_listener.system(),
        );
    }
}

fn setup(asset_server: Res<AssetServer>) {
    macro_rules! load_assets {
        ($($path:literal),*) => {{
            $(asset_server.load_untyped($path);)*
        }};
    }

    load_assets!(
        "arr\\flingy.dat",
        "arr\\sprites.dat",
        "arr\\images.dat",
        "arr\\techdata.dat",
        "arr\\upgrades.dat",
        "arr\\weapons.dat"
    );
}

macro_rules! asset_event_listener {
    ($fn_name:ident, $asset:ident) => {
        fn $fn_name(
            commands: &mut Commands,
            mut event_reader: Local<EventReader<AssetEvent<$asset>>>,
            mut assets: ResMut<Assets<$asset>>,
            asset_events: Res<Events<AssetEvent<$asset>>>,
        ) {
            for asset_event in event_reader.iter(&asset_events) {
                match asset_event {
                    AssetEvent::<$asset>::Created { handle }
                    | AssetEvent::<$asset>::Modified { handle } => {
                        let asset = assets.remove(handle).unwrap();
                        commands.insert_resource(asset);
                    }
                    _ => {}
                }
            }
        }
    };
}

asset_event_listener!(flingy_dat_asset_event_listener, FlingyDat);
asset_event_listener!(sprites_dat_asset_event_listener, SpritesDat);
asset_event_listener!(images_dat_asset_event_listener, ImagesDat);
asset_event_listener!(tech_data_dat_asset_event_listener, TechDataDat);
asset_event_listener!(upgrades_dat_asset_event_listener, UpgradesDat);
asset_event_listener!(weapons_dat_asset_event_listener, WeaponsDat);
