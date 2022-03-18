use crate::constants::Constants;
use crate::inspector::Inspectable;

use crate::unit::unit_type::UnitType;
use crate::{
    draw::colored_mesh::ColoredMeshBundle, geometry::transform::TransformBuilder,
    unit::faction::Faction,
};

use bevy::{
    core::Name,
    prelude::{AssetServer, Bundle, Component},
};

use super::discharge::Discharge;
use super::Weapon;

/// Unit struct for determining the weapon.
#[derive(Debug, Component, Inspectable)]
pub struct Bow;

/// Bow with mesh.
#[derive(Bundle, Inspectable)]
pub struct BowBundle {
    /// Determine that it's a bow.
    bow: Bow,
    /// Determine that it's a weapon.
    weapon: Weapon,
    /// Timer for firing the bow.
    discharge: Discharge,
    /// The faction of the unit holding the bow.
    faction: Faction,
    /// The mesh itself for the bow.
    #[bundle]
    #[inspectable(ignore)]
    mesh: ColoredMeshBundle,
    /// Name of the weapon.
    name: Name,
}

impl BowBundle {
    /// Create a new bundle.
    pub fn new(faction: Faction, asset_server: &AssetServer, constants: &Constants) -> Self {
        Self {
            faction,
            discharge: Discharge::new(UnitType::Archer, faction, constants),
            bow: Bow,
            weapon: Weapon,
            mesh: ColoredMeshBundle::new(asset_server.load("weapons/bow.svg"))
                .with_z_index(5.0)
                .with_rotation(match faction {
                    Faction::Ally => -20.0,
                    Faction::Enemy => 20.0,
                })
                .with_position(
                    match faction {
                        Faction::Ally => 0.5,
                        Faction::Enemy => -0.5,
                    },
                    1.0,
                ),
            name: Name::new("Bow"),
        }
    }
}

impl TransformBuilder for BowBundle {
    fn transform_mut_ref(&'_ mut self) -> &'_ mut bevy::prelude::Transform {
        self.mesh.transform_mut_ref()
    }
}
