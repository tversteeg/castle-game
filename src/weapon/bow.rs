use crate::{draw::colored_mesh::ColoredMeshBundle, geometry::transform::TransformBuilder};
use bevy::{
    core::Timer,
    prelude::{Bundle, Component, Handle, Mesh},
};
use bevy_inspector_egui::Inspectable;

#[derive(Debug, Component, Inspectable)]
pub struct Bow {
    /// Interval at which an arrow will be shoot.
    #[inspectable(ignore)]
    timer: Timer,
}

impl Bow {
    /// Spawn a bow which shoots an arrow every amount of seconds.
    pub fn from_seconds(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, true),
        }
    }
}

/// Bow with mesh.
#[derive(Bundle)]
pub struct BowBundle {
    /// The bow with the timer itself.
    pub bow: Bow,
    /// The mesh itself for the bow.
    #[bundle]
    pub mesh: ColoredMeshBundle,
}

impl BowBundle {
    /// Create a new bundle.
    pub fn new(shoot_interval_seconds: f32, mesh: Handle<Mesh>) -> Self {
        Self {
            bow: Bow::from_seconds(shoot_interval_seconds),
            mesh: ColoredMeshBundle::new(mesh).with_z_index(5.0),
        }
    }
}

impl TransformBuilder for BowBundle {
    fn transform_mut_ref(&'_ mut self) -> &'_ mut bevy::prelude::Transform {
        self.mesh.transform_mut_ref()
    }
}
