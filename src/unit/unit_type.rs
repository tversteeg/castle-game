use bevy::prelude::Component;
use crate::inspector::Inspectable;

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
}
