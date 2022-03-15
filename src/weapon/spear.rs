use crate::inspector::Inspectable;
use crate::{
    draw::colored_mesh::ColoredMeshBundle, geometry::transform::TransformBuilder,
    unit::faction::Faction,
};
use bevy::{
    core::Name,
    prelude::{AssetServer, Bundle, Component},
};

#[derive(Debug, Component, Inspectable)]
pub struct Spear;

/// Spear with mesh.
#[derive(Bundle)]
pub struct SpearBundle {
    /// The spear with the timer itself.
    pub spear: Spear,
    /// The faction of the unit holding the spear.
    faction: Faction,
    /// The mesh itself for the spear.
    #[bundle]
    pub mesh: ColoredMeshBundle,
    /// Name of the weapon.
    name: Name,
}

impl SpearBundle {
    /// Create a new bundle.
    pub fn new(faction: Faction, asset_server: &AssetServer) -> Self {
        Self {
            faction,
            spear: Spear,
            mesh: ColoredMeshBundle::new(asset_server.load("weapons/spear.svg"))
                .with_z_index(5.0)
                .with_rotation(match faction {
                    Faction::Ally => -20.0,
                    Faction::Enemy => 20.0,
                })
                .with_position(0.0, 1.0),
            name: Name::new("Spear"),
        }
    }
}

impl TransformBuilder for SpearBundle {
    fn transform_mut_ref(&'_ mut self) -> &'_ mut bevy::prelude::Transform {
        self.mesh.transform_mut_ref()
    }
}
