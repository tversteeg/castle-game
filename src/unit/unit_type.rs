use crate::{inspector::Inspectable, projectile::event::ProjectileType};
use bevy::prelude::Component;

/// The different types of units.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Component, Inspectable)]
pub enum UnitType {
    Soldier,
    Archer,
}

impl UnitType {
    /// The name as a string.
    pub fn to_string(self) -> &'static str {
        match self {
            UnitType::Soldier => "Soldier",
            UnitType::Archer => "Archer",
        }
    }

    /// What type of projectile type this unit spawns.
    pub fn to_projectile_type(self) -> ProjectileType {
        match self {
            UnitType::Soldier => ProjectileType::Direct,
            UnitType::Archer => ProjectileType::Arrow,
        }
    }
}
