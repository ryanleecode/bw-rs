use crate::make_dat;

use super::flingy::FlingyPointer;
use std::ops::Index;
use struple::Struple;

pub use unit_id::*;

mod unit_id;

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
#[derive(Debug)]
pub struct Unit {
    /// Unit's main graphics object.
    ///
    /// Pointer to flingy.dat
    pub graphics: FlingyPointer,

    /// Main subunit to the unit. Various turrets mostly.
    pub sub_unit_1: Option<UnitId>,

    /// Unit to transform into after Infestation. Exists only for units of
    /// ID 106-201 (buildings).
    pub infestation: Option<UnitId>,

    /// Construction graphics of the unit (used mostly with buildings).
    /// 0-Scourge = No graphics.
    ///
    /// Pointer to images.dat
    pub construction_animation: u32,

    /// Direction unit will face after it is created. Values start at 0
    /// (the unit will face the top of the screen) and go on clockwise through
    /// subsequent turning stages until 31 (unit will face a little left from
    /// the complete turn). Value of 32 means unit will face a random direction.
    pub unit_direction: u8,

    /// Enables Shields for the unit. Works for any unit, not only Protoss.
    /// Terran and zerg buildings with shields do NOT acquire full shield
    /// capacity during construction.
    pub are_shields_enabled: bool,

    /// Amount of Shield Points the unit has. Shields are reduced before the
    /// unit's Hit Points are affected.
    pub shield_amount: u16,

    /// Unit Hit Points (HP) or "life" amount. Values over "9999" will be
    /// accepted in-game, but they will not be displayed.
    pub hit_points: f32,

    /// The elevation level at which the unit moves. It can be used to make
    /// units moves like flyers, but still be attacked by ground weapons and
    /// act as ground units to specific special abilities (like Recall).
    /// Higher values puts the unit higher over terrain and other units.
    pub elevation_level: u8,

    /// Controls ground units movement: units with lower Rank will stop and
    /// wait to allow units with higher Rank to continue movement. Has no
    /// effects on air units. Also the order this unit is displayed in its
    /// folder in StarEdit.
    pub sub_label: u8,

    /// Order given to the unit if it is under computer control and does nothing.
    ///
    /// Pointer to orders.dat.
    pub comp_ai_idle: u8,

    /// Order given to the unit if it is under a human player's control and does
    /// nothing.
    ///
    /// Pointer to orders.dat.
    pub human_ai_idle: u8,

    /// Order executed after the unit has finished executing another order and
    /// returns to Idle state.
    ///
    /// Pointer to orders.dat.
    pub return_to_idle: u8,

    /// Order executed if the units is ordered to attack an enemy unit, also
    /// through the Right-Click action.
    ///
    /// Pointer to orders.dat.
    pub attack_unit: u8,

    /// Order executed if the unit is ordered to Attack Ground.
    ///
    /// Pointer to orders.dat.
    pub attack_move: u8,

    /// Weapon used for attacking "ground" units - those with the
    /// "Flying Target" advanced flag unchecked.
    ///
    /// Pointer to weapons.dat.
    pub ground_weapon: u8,

    /// Max number of times unit hits its target per Ground attack. This value
    /// is for statistics purposes only. Changing it only affects the value
    /// displayed in StarEdit.
    pub max_ground_hits: u8,

    /// Weapon used for attacking "air" or "flying" units - those with the
    /// "Flying Target" advanced flag checked.
    ///
    /// Pointer to weapons.dat
    pub air_weapon: u8,

    /// Max number of times unit hits its target per Air attack. This value is
    /// for statistics purposes only. Changing it only affects the value
    /// displayed in StarEdit.
    pub max_air_hits: u8,

    pub ai_internal: u8,

    pub special_ability_flags: u32,

    /// Range at which the Carrier will launch Interceptors and Reaver Scarabs.
    /// Also determines the range at which melee units and Medics will pickup
    /// targets.

    /// Value of 0 makes the game use the Weapons.dat for range data.
    /// 1 range unit here equals 2 range units in weapons.dat.
    pub target_acquisition_range: u8,

    /// Range (in matrices) indicating how much Fog-Of-War will the unit clear
    /// up.
    pub sight_range: u8,

    /// Researching this upgrade will improve the unit's Armor by 1.
    ///
    /// Pointer to upgrades.dat
    pub armour_upgrade: u8,

    /// Used to calculate the "Explosive" and "Concussive" weapons damage:
    /// Explosive (50% to Small, 75% to Medium),
    /// Concussive (50% to Medium, 25% to Large).
    /// "Independent" - the unit will lose 1 HP every second attack it takes
    /// (no matter by what unit or weapon and regardless of its Armor.
    //// Spell effects may vary, e.g. Plague works normally, but Irradiate
    /// doesn't).
    pub unit_size: u8,

    /// Unit's basic Armor level. Armor is subtracted from damage caused by
    /// every attack from another unit. If Armor is higher than the attack
    /// damage, the unit will act as if was of "Independent" Unit Size.
    pub armour: u8,

    /// Determines what actions may, or may not be taken by the unit if it is
    /// given an order through the Right-Click action.
    pub right_click_action: u8,

    /// Sound played after the unit is trained/built.
    ///
    /// 0=No sound.
    ///
    /// Pointer to sfxdata.dat
    pub ready_sound: Option<u16>,

    /// First of the "What" sounds - played when you select the unit.
    /// 0 = No sound.
    ///
    /// Pointer to sfxdata.dat
    pub what_sound_start: u16,

    /// Last of the "What" sounds - played when you select the unit.
    ///
    /// 0=No sound.
    ///
    /// Pointer to sfxdata.dat
    pub what_sound_end: u16,

    /// First of the "Annoyed" sounds - played when you click multiple times
    /// on the same unit.
    ///
    /// 0=No sound.
    ///
    /// Pointer to sfxdata.dat
    pub annoyed_sound_start: Option<u16>,

    /// Last of the "Annoyed" sounds - played when you click multiple times on
    /// the same unit.
    ///
    /// 0=No sound.
    pub annoyed_sound_end: Option<u16>,

    /// First of the "Yes" sounds - played when you give the unit an order.
    ///
    /// 0=No sound.
    ///
    /// Pointer to sfxdata.dat
    pub yes_sound_start: Option<u16>,

    /// Last of the "Yes" sounds - played when you give the unit an order.  
    ///
    /// 0=No sound.
    ///
    /// Pointer to sfxdata.dat
    pub yes_sound_end: Option<u16>,

    /// Width of the green placement rectangle in StarEdit, in pixels.
    /// Used instead of "Unit Dimensions" in StarEdit, but only if the unit
    /// is a "Building".
    pub star_edit_placement_box: StarEditPlacementBox,

    /// Horizontal distance in pixels at which an addon will be placed.
    /// Exists only for units of ID 106-201 (buildings).
    pub addon_horizontal: Option<u16>,

    /// Vertical distance in pixels at which an addon will be placed. Exists
    /// only for units of ID 106-201 (buildings).
    pub addon_vertical: Option<u16>,

    /// Dimensions of the unit. Measured in pixels.
    pub dimensions: Dimensions,

    /// Unit's Idle and Talking portraits.
    ///
    /// Pointer to portdata.dat
    pub portrait: u16,

    /// Amount of minerals needed to train/build the unit.
    pub mineral_cost: u16,

    /// Amount of Vespene Gas needed to train/build the unit.
    pub vespense_cost: u16,

    /// Amount of time it takes to train/build the unit, in 1/24ths of a second
    /// (at Fastest speed). A value of 0 will crash the game.
    pub build_time: u16,

    pub star_edit_group_flags: u8,

    /// Amount of Supply/Psi/Control the unit adds to the total pool. Halves
    /// are rounded down for the display, but calculated normally.
    pub supply_provided: u8,

    /// Amount of Supply/Psi/Control required to train/build the unit. Halves
    /// are rounded down for the display, but calculated normally.
    pub supply_required: u8,

    /// Amount of loading space the unit takes up in a transport.
    /// If it is higher than the transport's loading space, the unit cannot be
    /// loaded. It DOES NOT resize the unit's wireframe when inside the transport.
    pub space_required: u8,

    /// Amount of loading space the unit has. Used with dropships. This IS NOT
    /// the number of units the transporting unit may carry as different unit
    /// may take up different amount of loading space.
    pub space_provided: u8,

    /// Amount of points given for training/building the unit, counted to the
    /// total score after the end of a game.
    pub build_score: u16,

    /// Amount of points given for destroying the unit, counted to the total
    /// score after the end of a game. It is also used by the AI for targeting
    /// purposes. Units with a higher destroy score will be targeted first.
    pub destroy_score: u16,

    /// If this property is different from 0, the unit's name will be read from
    /// the strings stored within the map (CHK) that is currently loaded,
    /// instead of the stat_txt.tbl file.
    pub unit_map_string: u16,

    /// Makes the unit available only while playing BroodWar expansion set.
    pub is_broodwar_only: bool,

    pub star_edit_availability_flags: u16,
}

make_dat!(UnitsDat, Unit, UnitId);
