use amethyst::{
    assets::Format,
    assets::{Asset, Handle},
    ecs::DenseVecStorage,
};
use boolinator::Boolinator;
use nom::{
    bytes::complete::take,
    combinator::{all_consuming, map},
    error::ParseError,
    multi::count,
    number::complete::{le_u8, le_u16, le_u32},
    sequence::tuple,
    Finish, IResult, InputIter, InputTake, Parser,
};
use struple::Struple;

#[derive(Clone, Debug, Struple)]
pub struct StarEditPlacementBox {
    width: u16,
    height: u16,
}

#[derive(Clone, Debug, Struple)]
pub struct Dimensions {
    left: u16,
    up: u16,
    right: u16,
    down: u16,
}

#[derive(Clone, Debug)]
pub struct UnitPointer(u16);

#[derive(Debug)]
pub struct Unit {
    /// Unit's main graphics object.
    ///
    /// Pointer to flingy.dat
    graphics: u8,

    /// Main subunit to the unit. Various turrets mostly.
    sub_unit_1: UnitPointer,

    /// Unit to transform into after Infestation. Exists only for units of
    /// ID 106-201 (buildings).
    infestation: Option<UnitPointer>,

    /// Construction graphics of the unit (used mostly with buildings).
    /// 0-Scourge = No graphics.
    ///
    /// Pointer to images.dat
    construction_animation: u32,

    /// Direction unit will face after it is created. Values start at 0
    /// (the unit will face the top of the screen) and go on clockwise through
    /// subsequent turning stages until 31 (unit will face a little left from
    /// the complete turn). Value of 32 means unit will face a random direction.
    unit_direction: u8,

    /// Enables Shields for the unit. Works for any unit, not only Protoss.
    /// Terran and zerg buildings with shields do NOT acquire full shield
    /// capacity during construction.
    are_shields_enabled: bool,

    /// Amount of Shield Points the unit has. Shields are reduced before the
    /// unit's Hit Points are affected.
    shield_amount: u16,

    /// Unit Hit Points (HP) or "life" amount. Values over "9999" will be
    /// accepted in-game, but they will not be displayed.
    hit_points: f32,

    /// The elevation level at which the unit moves. It can be used to make
    /// units moves like flyers, but still be attacked by ground weapons and
    /// act as ground units to specific special abilities (like Recall).
    /// Higher values puts the unit higher over terrain and other units.
    elevation_level: u8,

    /// Controls ground units movement: units with lower Rank will stop and
    /// wait to allow units with higher Rank to continue movement. Has no
    /// effects on air units. Also the order this unit is displayed in its
    /// folder in StarEdit.
    sub_label: u8,

    /// Order given to the unit if it is under computer control and does nothing.
    ///
    /// Pointer to orders.dat.
    comp_ai_idle: u8,

    /// Order given to the unit if it is under a human player's control and does
    /// nothing.
    ///
    /// Pointer to orders.dat.
    human_ai_idle: u8,

    /// Order executed after the unit has finished executing another order and
    /// returns to Idle state.
    ///
    /// Pointer to orders.dat.
    return_to_idle: u8,

    /// Order executed if the units is ordered to attack an enemy unit, also
    /// through the Right-Click action.
    ///
    /// Pointer to orders.dat.
    attack_unit: u8,

    /// Order executed if the unit is ordered to Attack Ground.
    ///
    /// Pointer to orders.dat.
    attack_move: u8,

    /// Weapon used for attacking "ground" units - those with the
    /// "Flying Target" advanced flag unchecked.
    ///
    /// Pointer to weapons.dat.
    ground_weapon: u8,

    /// Max number of times unit hits its target per Ground attack. This value
    /// is for statistics purposes only. Changing it only affects the value
    /// displayed in StarEdit.
    max_ground_hits: u8,

    /// Weapon used for attacking "air" or "flying" units - those with the
    /// "Flying Target" advanced flag checked.
    ///
    /// Pointer to weapons.dat
    air_weapon: u8,

    /// Max number of times unit hits its target per Air attack. This value is
    /// for statistics purposes only. Changing it only affects the value
    /// displayed in StarEdit.
    max_air_hits: u8,

    ai_internal: u8,

    special_ability_flags: u32,

    /// Range at which the Carrier will launch Interceptors and Reaver Scarabs.
    /// Also determines the range at which melee units and Medics will pickup
    /// targets.

    /// Value of 0 makes the game use the Weapons.dat for range data.
    /// 1 range unit here equals 2 range units in weapons.dat.
    target_acquisition_range: u8,

    /// Range (in matrices) indicating how much Fog-Of-War will the unit clear
    /// up.
    sight_range: u8,

    /// Researching this upgrade will improve the unit's Armor by 1.
    ///
    /// Pointer to upgrades.dat
    armour_upgrade: u8,

    /// Used to calculate the "Explosive" and "Concussive" weapons damage:
    /// Explosive (50% to Small, 75% to Medium),
    /// Concussive (50% to Medium, 25% to Large).
    /// "Independent" - the unit will lose 1 HP every second attack it takes
    /// (no matter by what unit or weapon and regardless of its Armor.
    //// Spell effects may vary, e.g. Plague works normally, but Irradiate
    /// doesn't).
    unit_size: u8,

    /// Unit's basic Armor level. Armor is subtracted from damage caused by
    /// every attack from another unit. If Armor is higher than the attack
    /// damage, the unit will act as if was of "Independent" Unit Size.
    armour: u8,

    /// Determines what actions may, or may not be taken by the unit if it is
    /// given an order through the Right-Click action.
    right_click_action: u8,

    /// Sound played after the unit is trained/built.
    ///
    /// 0=No sound.
    ///
    /// Pointer to sfxdata.dat
    ready_sound: Option<u16>,

    /// First of the "What" sounds - played when you select the unit.
    /// 0 = No sound.
    ///
    /// Pointer to sfxdata.dat
    what_sound_start: u16,

    /// Last of the "What" sounds - played when you select the unit.
    ///
    /// 0=No sound.
    ///
    /// Pointer to sfxdata.dat
    what_sound_end: u16,

    /// First of the "Annoyed" sounds - played when you click multiple times
    /// on the same unit.
    ///
    /// 0=No sound.
    ///
    /// Pointer to sfxdata.dat
    annoyed_sound_start: Option<u16>,

    /// Last of the "Annoyed" sounds - played when you click multiple times on
    /// the same unit.
    ///
    /// 0=No sound.
    annoyed_sound_end: Option<u16>,

    /// First of the "Yes" sounds - played when you give the unit an order.
    ///
    /// 0=No sound.
    ///
    /// Pointer to sfxdata.dat
    yes_sound_start: Option<u16>,

    /// Last of the "Yes" sounds - played when you give the unit an order.  
    ///
    /// 0=No sound.
    ///
    /// Pointer to sfxdata.dat
    yes_sound_end: Option<u16>,

    /// Width of the green placement rectangle in StarEdit, in pixels.
    /// Used instead of "Unit Dimensions" in StarEdit, but only if the unit
    /// is a "Building".
    star_edit_placement_box: StarEditPlacementBox,

    /// Horizontal distance in pixels at which an addon will be placed.
    /// Exists only for units of ID 106-201 (buildings).
    addon_horizontal: Option<u16>,

    /// Vertical distance in pixels at which an addon will be placed. Exists
    /// only for units of ID 106-201 (buildings).
    addon_vertical: Option<u16>,

    /// Dimensions of the unit. Measured in pixels.
    dimensions: Dimensions,

    /// Unit's Idle and Talking portraits.
    ///
    /// Pointer to portdata.dat
    portrait: u16,

    /// Amount of minerals needed to train/build the unit.
    mineral_cost: u16,

    /// Amount of Vespene Gas needed to train/build the unit.
    vespense_cost: u16,

    /// Amount of time it takes to train/build the unit, in 1/24ths of a second
    /// (at Fastest speed). A value of 0 will crash the game.
    build_time: u16,

    star_edit_group_flags: u8,

    /// Amount of Supply/Psi/Control the unit adds to the total pool. Halves
    /// are rounded down for the display, but calculated normally.
    supply_provided: u8,

    /// Amount of Supply/Psi/Control required to train/build the unit. Halves
    /// are rounded down for the display, but calculated normally.
    supply_required: u8,

    /// Amount of loading space the unit takes up in a transport.
    /// If it is higher than the transport's loading space, the unit cannot be
    /// loaded. It DOES NOT resize the unit's wireframe when inside the transport.
    space_required: u8,

    /// Amount of loading space the unit has. Used with dropships. This IS NOT
    /// the number of units the transporting unit may carry as different unit
    /// may take up different amount of loading space.
    space_provided: u8,

    /// Amount of points given for training/building the unit, counted to the
    /// total score after the end of a game.
    build_score: u16,

    /// Amount of points given for destroying the unit, counted to the total
    /// score after the end of a game. It is also used by the AI for targeting
    /// purposes. Units with a higher destroy score will be targeted first.
    destroy_score: u16,

    /// If this property is different from 0, the unit's name will be read from
    /// the strings stored within the map (CHK) that is currently loaded,
    /// instead of the stat_txt.tbl file.
    unit_map_string: u16,

    /// Makes the unit available only while playing BroodWar expansion set.
    is_broodwar_only: bool,

    star_edit_availability_flags: u16,
}

pub struct UnitsDat(Vec<Unit>);

pub struct UnitsDatAsset(Option<UnitsDat>);

impl UnitsDatAsset {
    pub fn take(&mut self) -> Option<UnitsDat> {
        self.0.take()
    }
}

pub type UnitsDatHandle = Handle<UnitsDatAsset>;

impl Asset for UnitsDatAsset {
    const NAME: &'static str = "bw_assets::dat::UnitsDatAsset";
    type Data = Self;
    type HandleStorage = DenseVecStorage<UnitsDatHandle>;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UnitsDatFormat;

impl Format<UnitsDatAsset> for UnitsDatFormat {
    fn name(&self) -> &'static str {
        "UnitsDatFormat"
    }

    fn import_simple(&self, b: Vec<u8>) -> amethyst::Result<UnitsDatAsset> {
        let (_, unit_dat) = parse_unit_dat(&b).finish().map_err(|err| {
            amethyst::error::format_err!(
                "failed to load units.dat asset: {} at position {}",
                err.code.description(),
                b.len() - err.input.len()
            )
        })?;

        Ok(UnitsDatAsset(Some(unit_dat)))
    }
}

const UNIT_COUNT: usize = 106;
const BUILDING_COUNT: usize = 96;
const BLOCK_SIZE: usize = 228;

fn parse_u8_boolean(b: &[u8]) -> IResult<&[u8], bool> {
    map(le_u8, |x| x != 0)(b)
}

fn take_block(b: &[u8], size: u8) -> IResult<&[u8], ()> {
    map(count_total(take(size)), |_| ())(b)
}

fn take_u8_block(b: &[u8]) -> IResult<&[u8], ()> {
    take_block(b, 1u8)
}

fn take_u16_block(b: &[u8]) -> IResult<&[u8], ()> {
    take_block(b, 2u8)
}

fn u32_to_f32(x: u32, decimal_places: u32) -> f32 {
    let whole = (x >> decimal_places) as f32;
    let decimals = (x & ((1 << decimal_places) - 1)) as f32 / 10f32.powf(decimal_places as f32);

    whole + decimals
}

fn count_building_block<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq + InputIter + InputTake,
    F: Parser<I, O, E> + Clone,
    E: ParseError<I>,
{
    count(f, BUILDING_COUNT)
}

fn count_unit_block<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq + InputIter + InputTake,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    count(f, UNIT_COUNT)
}

pub fn count_total<I, O, E, F>(f: F) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + PartialEq,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    count(f, BLOCK_SIZE)
}

fn parse_unit_pointer(b: &[u8]) -> IResult<&[u8], UnitPointer> {
    map(le_u16, UnitPointer)(b)
}

fn parse_unit_dat(b: &[u8]) -> IResult<&[u8], UnitsDat> {
    let (remaining, graphic_col) = count_total(le_u8)(b)?;

    let (remaining, sub_unit_1_col) = count_total(parse_unit_pointer)(remaining)?;

    // The next u16 is sub_unit_2 which is unused by the game.
    let (remaining, _) = take_u16_block(remaining)?;

    let (remaining, infestation_col) = count_building_block(parse_unit_pointer)(remaining)?;
    let (remaining, construction_animation_col) = count_total(le_u32)(remaining)?;
    let (remaining, unit_direction_col) = count_total(le_u8)(remaining)?;
    let (remaining, are_shields_enabled_col) = count_total(parse_u8_boolean)(remaining)?;
    let (remaining, shield_amount_col) = count_total(le_u16)(remaining)?;
    let (remaining, hit_points_col) = count_total(map(le_u32, |x| u32_to_f32(x, 8)))(remaining)?;
    let (remaining, elevation_level_col) = count_total(le_u8)(remaining)?;

    // unknown block
    let (remaining, _) = take_u8_block(remaining)?;

    let (remaining, sub_label_col) = count_total(le_u8)(remaining)?;
    let (remaining, comp_ai_idle_col) = count_total(le_u8)(remaining)?;
    let (remaining, human_ai_idle_col) = count_total(le_u8)(remaining)?;
    let (remaining, return_to_idle_col) = count_total(le_u8)(remaining)?;
    let (remaining, attack_unit_col) = count_total(le_u8)(remaining)?;
    let (remaining, attack_move_col) = count_total(le_u8)(remaining)?;
    let (remaining, ground_weapon_col) = count_total(le_u8)(remaining)?;
    let (remaining, max_ground_hits_col) = count_total(le_u8)(remaining)?;
    let (remaining, air_weapon_col) = count_total(le_u8)(remaining)?;
    let (remaining, max_air_hits_col) = count_total(le_u8)(remaining)?;
    let (remaining, ai_internal_col) = count_total(le_u8)(remaining)?;
    let (remaining, special_ability_flags_col) = count_total(le_u32)(remaining)?;
    let (remaining, target_acquisition_range_col) = count_total(le_u8)(remaining)?;
    let (remaining, sight_range_col) = count_total(le_u8)(remaining)?;
    let (remaining, armour_upgrade_col) = count_total(le_u8)(remaining)?;
    let (remaining, unit_size_col) = count_total(le_u8)(remaining)?;
    let (remaining, armour_col) = count_total(le_u8)(remaining)?;
    let (remaining, right_click_action_col) = count_total(le_u8)(remaining)?;
    let (remaining, ready_sound_col) = count_unit_block(le_u16)(remaining)?;
    let (remaining, what_sound_start_col) = count_total(le_u16)(remaining)?;
    let (remaining, what_sound_end_col) = count_total(le_u16)(remaining)?;
    let (remaining, annoyed_sound_start_col) = count_unit_block(le_u16)(remaining)?;
    let (remaining, annoyed_sound_end_col) = count_unit_block(le_u16)(remaining)?;
    let (remaining, yes_sound_start_col) = count_unit_block(le_u16)(remaining)?;
    let (remaining, yes_sound_end_col) = count_unit_block(le_u16)(remaining)?;


    let (remaining, star_edit_placement_col) = count_total(map(
        tuple((le_u16, le_u16)),
        StarEditPlacementBox::from_tuple,
    ))(remaining)?;

    let (remaining, addon_horizontal_col) = count_building_block(le_u16)(remaining)?;
    let (remaining, addon_vertical_col) = count_building_block(le_u16)(remaining)?;

    let (remaining, dimensions_col) = count_total(map(
        tuple((le_u16, le_u16, le_u16, le_u16)),
        Dimensions::from_tuple,
    ))(remaining)?;

    let (remaining, portrait_col) = count_total(le_u16)(remaining)?;
    let (remaining, mineral_cost_col) = count_total(le_u16)(remaining)?;
    let (remaining, vespense_cost_col) = count_total(le_u16)(remaining)?;
    let (remaining, build_time_col) = count_total(le_u16)(remaining)?;

    // The u16 after build time is unknown and useless
    let (remaining, _) = take_u16_block(remaining)?;

    let (remaining, star_edit_group_flags_col) = count_total(le_u8)(remaining)?;
    let (remaining, supply_provided_col) = count_total(le_u8)(remaining)?;
    let (remaining, supply_required_col) = count_total(le_u8)(remaining)?;
    let (remaining, space_required_col) = count_total(le_u8)(remaining)?;
    let (remaining, space_provided_col) = count_total(le_u8)(remaining)?;
    let (remaining, build_score_col) = count_total(le_u16)(remaining)?;
    let (remaining, destroy_score_col) = count_total(le_u16)(remaining)?;
    let (remaining, unit_map_string_col) = count_total(le_u16)(remaining)?;
    let (remaining, is_broodwar_only_col) = count_total(parse_u8_boolean)(remaining)?;

    let (remaining, star_edit_availability_flags_col) = count_total(le_u16)(remaining)?;

    all_consuming(take(0u8))(remaining)?;

    let units =
        (0..BLOCK_SIZE)
            .map(|i| Unit {
                graphics: graphic_col[i],
                sub_unit_1: sub_unit_1_col[i].clone(),
                infestation: (i >= UNIT_COUNT && i < UNIT_COUNT + BUILDING_COUNT)
                    .and_option_from(|| infestation_col.get(i).map(ToOwned::to_owned)),
                construction_animation: construction_animation_col[i],
                unit_direction: unit_direction_col[i],
                are_shields_enabled: are_shields_enabled_col[i],
                shield_amount: shield_amount_col[i],
                hit_points: hit_points_col[i],
                elevation_level: elevation_level_col[i],
                sub_label: sub_label_col[i],
                comp_ai_idle: comp_ai_idle_col[i],
                human_ai_idle: human_ai_idle_col[i],
                return_to_idle: return_to_idle_col[i],
                attack_unit: attack_unit_col[i],
                attack_move: attack_move_col[i],
                ground_weapon: ground_weapon_col[i],
                max_ground_hits: max_ground_hits_col[i],
                air_weapon: air_weapon_col[i],
                max_air_hits: max_air_hits_col[i],
                ai_internal: ai_internal_col[i],
                special_ability_flags: special_ability_flags_col[i],
                target_acquisition_range: target_acquisition_range_col[i],
                sight_range: sight_range_col[i],
                armour_upgrade: armour_upgrade_col[i],
                unit_size: unit_size_col[i],
                armour: armour_col[i],
                right_click_action: right_click_action_col[i],
                ready_sound: (i < UNIT_COUNT)
                    .and_option_from(|| ready_sound_col.get(i).map(ToOwned::to_owned)),
                what_sound_start: what_sound_start_col[i],
                what_sound_end: what_sound_end_col[i],
                annoyed_sound_start: (i < UNIT_COUNT)
                    .and_option_from(|| annoyed_sound_start_col.get(i).map(ToOwned::to_owned)),
                annoyed_sound_end: (i < UNIT_COUNT)
                    .and_option_from(|| annoyed_sound_end_col.get(i).map(ToOwned::to_owned)),
                yes_sound_start: (i < UNIT_COUNT)
                    .and_option_from(|| yes_sound_start_col.get(i).map(ToOwned::to_owned)),
                yes_sound_end: (i < UNIT_COUNT)
                    .and_option_from(|| yes_sound_end_col.get(i).map(ToOwned::to_owned)),
                star_edit_placement_box: star_edit_placement_col[i].clone(),
                addon_horizontal: (i >= UNIT_COUNT && i < UNIT_COUNT + BUILDING_COUNT)
                    .and_option_from(|| {
                        addon_horizontal_col
                            .get(i - UNIT_COUNT)
                            .map(ToOwned::to_owned)
                    }),
                addon_vertical: (i >= UNIT_COUNT && i < UNIT_COUNT + BUILDING_COUNT)
                    .and_option_from(|| {
                        addon_vertical_col
                            .get(i - UNIT_COUNT)
                            .map(ToOwned::to_owned)
                    }),
                dimensions: dimensions_col[i].clone(),
                portrait: portrait_col[i],
                mineral_cost: mineral_cost_col[i],
                vespense_cost: vespense_cost_col[i],
                build_time: build_time_col[i],
                star_edit_group_flags: star_edit_group_flags_col[i],
                supply_provided: supply_provided_col[i],
                supply_required: supply_required_col[i],
                space_required: space_required_col[i],
                space_provided: space_provided_col[i],
                build_score: build_score_col[i],
                destroy_score: destroy_score_col[i],
                unit_map_string: unit_map_string_col[i],
                is_broodwar_only: is_broodwar_only_col[i],
                star_edit_availability_flags: star_edit_availability_flags_col[i],
            })
            .collect::<Vec<_>>();

    Ok((remaining, UnitsDat(units)))
}
