use bevy::prelude::Component;
use bevy_inspector_egui::Inspectable;

/// When the health of a unit is zero, it dies.
#[derive(Debug, Component, Inspectable)]
pub struct Health {
    /// The health of the unit.
    hp: f32,
}

impl Health {
    /// Construct the health component.
    pub fn new(hp: f32) -> Self {
        Self { hp }
    }
}
