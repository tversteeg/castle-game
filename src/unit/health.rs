use crate::{constants::Constants, inspector::Inspectable};
use bevy::prelude::Component;

use super::{faction::Faction, unit_type::UnitType};

/// When the health of a unit is zero, it dies.
#[derive(Debug, Component, Inspectable)]
pub struct Health {
    /// The health of the unit.
    hp: f32,
}

impl Health {
    /// Construct the health component.
    pub fn for_unit(unit_type: UnitType, faction: Faction, constants: &Constants) -> Self {
        let hp = constants.unit(unit_type, faction).hp;

        Self { hp }
    }
}
