use bevy::prelude::Component;
use crate::inspector::Inspectable;

/// To which side the unit belongs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Inspectable)]
pub enum Faction {
    /// We control this unit.
    Ally,
    /// The enemy AI controls this unit.
    Enemy,
}

impl Faction {
    /// The name as a string.
    pub fn to_string(self) -> &'static str {
        match self {
            Faction::Ally => "Ally",
            Faction::Enemy => "Enemy",
        }
    }
}
