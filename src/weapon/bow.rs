use crate::{
    draw::colored_mesh::ColoredMeshBundle, geometry::transform::TransformBuilder,
    unit::faction::Faction,
};
use bevy::{
    core::{Name, Timer},
    prelude::{AssetServer, Bundle, Component, Handle, Mesh},
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
    bow: Bow,
    /// The mesh itself for the bow.
    #[bundle]
    mesh: ColoredMeshBundle,
    /// Name of the weapon.
    name: Name,
}

impl BowBundle {
    /// Create a new bundle.
    pub fn new(faction: Faction, asset_server: &AssetServer) -> Self {
        Self {
            bow: Bow::from_seconds(5.0),
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
