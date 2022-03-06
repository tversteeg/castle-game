use bevy::prelude::Component;
use bevy_inspector_egui::Inspectable;

/// The different types of units.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Component, Inspectable)]
pub enum UnitType {
    Soldier,
    Archer,
}

impl UnitType {
    /// The label for the recruit button.
    pub fn recruit_button_label(&self) -> &'static str {
        match self {
            UnitType::Soldier => "Soldier",
            UnitType::Archer => "Archer",
        }
    }
}
