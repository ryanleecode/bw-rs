use crate::assets::ProgressCounterMutRef;
use amethyst::{
    assets::Loader,
    assets::{Handle, ProgressCounter},
    prelude::*,
    ui::{FontAsset, TtfFormat},
};
use bw_assets::dat::{UnitDatAsset, UnitDatFormat, UnitDatHandle};

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
    pub unit_dat: Handle<UnitDatAsset>,
}

pub fn load_dats(world: &mut World, progress_counter: &mut ProgressCounter) -> DatHandles {
    let mut progress_counter_newtype = ProgressCounterMutRef::new(progress_counter);

    let units_dat: UnitDatHandle = world.read_resource::<Loader>().load_from(
        "arr\\units.dat",
        UnitDatFormat,
        "bw_assets",
        &mut progress_counter_newtype,
        &world.read_resource(),
    );

    DatHandles {
        unit_dat: units_dat,
    }
}
