use bevy::prelude::Component;
use crate::inspector::Inspectable;

use super::{faction::Faction, unit_type::UnitType};

/// When the health of a unit is zero, it dies.
#[derive(Debug, Component, Inspectable)]
pub struct Health {
    /// The health of the unit.
    hp: f32,
}

impl Health {
    /// Construct the health component.
    pub fn for_unit(unit_type: UnitType, faction: Faction) -> Self {
        let hp = match (unit_type, faction) {
            (UnitType::Soldier, Faction::Ally) => 100.0,
            (UnitType::Soldier, Faction::Enemy) => 100.0,
            (UnitType::Archer, Faction::Ally) => 100.0,
            (UnitType::Archer, Faction::Enemy) => 100.0,
        };

        Self { hp }
    }
}
