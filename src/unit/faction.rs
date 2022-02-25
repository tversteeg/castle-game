use bevy::prelude::Component;
use bevy_inspector_egui::Inspectable;

/// To which side the unit belongs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, Inspectable)]
pub enum Faction {
    /// We control this unit.
    Ally,
    /// The enemy AI controls this unit.
    Enemy,
}
