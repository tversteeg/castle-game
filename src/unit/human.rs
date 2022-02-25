use bevy::prelude::Component;
use bevy_inspector_egui::Inspectable;

/// A human unit.
#[derive(Debug, Component, Inspectable)]
pub struct Human;
