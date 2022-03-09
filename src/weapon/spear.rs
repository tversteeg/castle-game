use crate::{draw::colored_mesh::ColoredMeshBundle, geometry::transform::TransformBuilder};
use bevy::prelude::{Bundle, Component, Handle, Mesh};
use bevy_inspector_egui::Inspectable;

#[derive(Debug, Component, Inspectable)]
pub struct Spear;

/// Spear with mesh.
#[derive(Bundle)]
pub struct SpearBundle {
    /// The spear with the timer itself.
    pub spear: Spear,
    /// The mesh itself for the spear.
    #[bundle]
    pub mesh: ColoredMeshBundle,
}

impl SpearBundle {
    /// Create a new bundle.
    pub fn new(mesh: Handle<Mesh>) -> Self {
        Self {
            spear: Spear,
            mesh: ColoredMeshBundle::new(mesh).with_z_index(5.0),
        }
    }
}

impl TransformBuilder for SpearBundle {
    fn transform_mut_ref(&'_ mut self) -> &'_ mut bevy::prelude::Transform {
        self.mesh.transform_mut_ref()
    }
}
