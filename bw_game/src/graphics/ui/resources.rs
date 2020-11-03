use crate::assets::ProgressCounterMutRef;
use amethyst::{
    assets::Loader,
    assets::{Handle, ProgressCounter},
    prelude::*,
    ui::{FontAsset, TtfFormat},
};
use bw_assets::dat::{
    FlingyDatFormat, FlingyDatHandle, SpritesDatFormat, SpritesDatHandle, UnitsDatFormat,
    UnitsDatHandle, WeaponsDatFormat, WeaponsDatHandle,
};

#[derive(Clone)]
pub struct Fonts {
    pub reg: Handle<FontAsset>,
    pub bold: Handle<FontAsset>,
    pub reg_ext: Handle<FontAsset>,
}

pub fn load_fonts(world: &mut World, progress_counter: &mut ProgressCounter) {
    let mut progress_counter_newtype = ProgressCounterMutRef::new(progress_counter);

    let eurostile_reg: Handle<FontAsset> = world.read_resource::<Loader>().load(
        "fonts/Eurostile-Reg.ttf",
        TtfFormat,
        &mut progress_counter_newtype,
        &world.read_resource(),
    );
    let eurostile_bold: Handle<FontAsset> = world.read_resource::<Loader>().load(
        "fonts/Eurostile-Bol.ttf",
        TtfFormat,
        &mut progress_counter_newtype,
        &world.read_resource(),
    );
    let eurostile_reg_ext: Handle<FontAsset> = world.read_resource::<Loader>().load(
        "fonts/EurostileExt-Reg.ttf",
        TtfFormat,
        &mut progress_counter_newtype,
        &world.read_resource(),
    );

    let fonts = Fonts {
        reg: eurostile_reg,
        bold: eurostile_bold,
        reg_ext: eurostile_reg_ext,
    };

    world.insert(fonts);
}

#[derive(Debug, Clone)]
pub struct DatHandles {
    pub units_dat: UnitsDatHandle,
    pub flingy_dat: FlingyDatHandle,
    pub weapons_dat: WeaponsDatHandle,
    pub sprites_dat: SpritesDatHandle,
}

pub fn load_dats(world: &mut World, progress_counter: &mut ProgressCounter) -> DatHandles {
    let mut progress_counter_newtype = ProgressCounterMutRef::new(progress_counter);

    let units_dat: UnitsDatHandle = world.read_resource::<Loader>().load_from(
        "arr\\units.dat",
        UnitsDatFormat,
        "bw_assets",
        &mut progress_counter_newtype,
        &world.read_resource(),
    );

    let flingy_dat = world.read_resource::<Loader>().load_from(
        "arr\\flingy.dat",
        FlingyDatFormat,
        "bw_assets",
        &mut progress_counter_newtype,
        &world.read_resource(),
    );

    let weapons_dat = world.read_resource::<Loader>().load_from(
        "arr\\weapons.dat",
        WeaponsDatFormat,
        "bw_assets",
        &mut progress_counter_newtype,
        &world.read_resource(),
    );

    let sprites_dat = world.read_resource::<Loader>().load_from(
        "arr\\sprites.dat",
        SpritesDatFormat,
        "bw_assets",
        &mut progress_counter_newtype,
        &world.read_resource(),
    );

    DatHandles {
        units_dat,
        flingy_dat,
        weapons_dat,
        sprites_dat,
    }
}
