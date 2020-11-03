use amethyst::{
    assets::Format,
    assets::{Asset, Handle},
    ecs::DenseVecStorage,
};
use nom::{
    bytes::complete::take,
    combinator::all_consuming,
    combinator::map,
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u16, le_u32},
    Finish, IResult, Parser,
};

use super::flingy::FlingyPointer;

#[derive(Debug)]
pub struct Weapon {
    label: u16,
    graphics: FlingyPointer,
    target_flags: u16,
    minimum_range: u32,
    maximum_range: u32,
    damage_upgrade: u8,
    weapon_type: u8,
    weapon_behavior: u8,
    remove_after: u8,
    weapon_effect: u8,
    inner_splash_radius: u16,
    medium_splash_radius: u16,
    outer_splash_radius: u16,
    damage_amount: u16,
    damage_bonus: u16,
    weapon_cooldown: u8,
    damage_factor: u8,
    attack_angle: u8,
    launch_spin: u8,
    forward_offset: u8,
    upward_offset: u8,
    target_error_message: u16,
    icon: u16,
}

pub struct WeaponsDat(Vec<Weapon>);

pub struct WeaponsDatAsset(Option<WeaponsDat>);

impl WeaponsDatAsset {
    pub fn take(&mut self) -> Option<WeaponsDat> {
        self.0.take()
    }
}

pub type WeaponsDatHandle = Handle<WeaponsDatAsset>;

impl Asset for WeaponsDatAsset {
    const NAME: &'static str = "bw_assets::dat::WeaponsDatAsset";
    type Data = Self;
    type HandleStorage = DenseVecStorage<WeaponsDatHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WeaponsDatFormat;

impl Format<WeaponsDatAsset> for WeaponsDatFormat {
    fn name(&self) -> &'static str {
        "WeaponsDatFormat"
    }

    fn import_simple(&self, b: Vec<u8>) -> amethyst::Result<WeaponsDatAsset> {
        let (_, weapons_dat) = parse_weapons_dat(&b).finish().map_err(|err| {
            amethyst::error::format_err!(
                "failed to load weapons.dat asset: {} at position {}",
                err.code.description(),
                b.len() - err.input.len()
            )
        })?;

        Ok(WeaponsDatAsset(Some(weapons_dat)))
    }
}

const BLOCK_SIZE: usize = 130;

pub fn count_total<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    count(f, BLOCK_SIZE)
}

fn parse_weapons_dat(b: &[u8]) -> IResult<&[u8], WeaponsDat> {
    let (remaining, label_col) = count_total(le_u16)(b)?;
    let (remaining, flingy_pointer_col) = count_total(map(le_u32, FlingyPointer::new))(remaining)?;

    // unused block
    let (remaining, _) = count_total(le_u8)(remaining)?;

    let (remaining, target_flags_col) = count_total(le_u16)(remaining)?;
    let (remaining, minimum_range_col) = count_total(le_u32)(remaining)?;
    let (remaining, maximum_range_col) = count_total(le_u32)(remaining)?;
    let (remaining, damage_upgrade_col) = count_total(le_u8)(remaining)?;
    let (remaining, weapon_type_col) = count_total(le_u8)(remaining)?;
    let (remaining, weapon_behavior_col) = count_total(le_u8)(remaining)?;
    let (remaining, remove_after_col) = count_total(le_u8)(remaining)?;
    let (remaining, weapon_effect_col) = count_total(le_u8)(remaining)?;

    let (remaining, inner_splash_radius_col) = count_total(le_u16)(remaining)?;
    let (remaining, medium_splash_radius_col) = count_total(le_u16)(remaining)?;
    let (remaining, outer_splash_radius_col) = count_total(le_u16)(remaining)?;

    let (remaining, damage_amount_col) = count_total(le_u16)(remaining)?;
    let (remaining, damage_bonus_col) = count_total(le_u16)(remaining)?;
    let (remaining, weapon_cooldown_col) = count_total(le_u8)(remaining)?;
    let (remaining, damage_factor_col) = count_total(le_u8)(remaining)?;
    let (remaining, attack_angle_col) = count_total(le_u8)(remaining)?;
    let (remaining, launch_spin_col) = count_total(le_u8)(remaining)?;
    let (remaining, forward_offset_col) = count_total(le_u8)(remaining)?;
    let (remaining, upward_offset_col) = count_total(le_u8)(remaining)?;

    let (remaining, target_error_message_col) = count_total(le_u16)(remaining)?;
    let (remaining, icon_col) = count_total(le_u16)(remaining)?;

    all_consuming(take(0u8))(remaining)?;

    let weapons = (0..BLOCK_SIZE)
        .map(|i| Weapon {
            label: label_col[i],
            graphics: flingy_pointer_col[i].clone(),
            target_flags: target_flags_col[i],
            minimum_range: minimum_range_col[i],
            maximum_range: maximum_range_col[i],
            damage_upgrade: damage_upgrade_col[i],
            weapon_type: weapon_type_col[i],
            weapon_behavior: weapon_behavior_col[i],
            remove_after: remove_after_col[i],
            weapon_effect: weapon_effect_col[i],
            inner_splash_radius: inner_splash_radius_col[i],
            medium_splash_radius: medium_splash_radius_col[i],
            outer_splash_radius: outer_splash_radius_col[i],
            damage_amount: damage_amount_col[i],
            damage_bonus: damage_bonus_col[i],
            weapon_cooldown: weapon_cooldown_col[i],
            damage_factor: damage_factor_col[i],
            attack_angle: attack_angle_col[i],
            launch_spin: launch_spin_col[i],
            forward_offset: forward_offset_col[i],
            upward_offset: upward_offset_col[i],
            target_error_message: target_error_message_col[i],
            icon: icon_col[i],
        })
        .collect::<Vec<_>>();

    Ok((remaining, WeaponsDat(weapons)))
}
