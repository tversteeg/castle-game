use crate::geometry::transform::TransformBuilder;
use crate::projectile::Projectile;
use crate::{draw::colored_mesh::ColoredMeshBundle, inspector::Inspectable};
use bevy::prelude::{AssetServer, Assets, Mesh};
use bevy::{
    math::Vec2,
    prelude::{Bundle, Color, Commands, Component},
    sprite::{Sprite, SpriteBundle},
};
use bevy_rapier2d::prelude::{
    ActiveEvents, ColliderMassProps, ColliderShape, ColliderShapeComponent, RigidBodyCcd,
    RigidBodyType,
};
use bevy_rapier2d::{
    physics::{ColliderBundle, RigidBodyBundle},
    prelude::RigidBodyVelocity,
};

/// Unit struct for determining the projectile.
#[derive(Debug, Component, Inspectable)]
pub struct Arrow;

/// The arrow with other components.
#[derive(Bundle, Inspectable)]
pub struct ArrowBundle {
    /// Determine that it's an arrow.
    arrow: Arrow,
    /// Determine that it's a projectile.
    projectile: Projectile,
    /// The mesh itself for the arrow.
    #[bundle]
    #[inspectable(ignore)]
    mesh: ColoredMeshBundle,
    /// Physics.
    #[bundle]
    #[inspectable(ignore)]
    rigid_body: RigidBodyBundle,
    /// Detecting collisions.
    #[bundle]
    #[inspectable(ignore)]
    collider: ColliderBundle,
}

impl ArrowBundle {
    /// Shoot a new arrow.
    pub fn new(
        position: Vec2,
        velocity: RigidBodyVelocity,
        rotation: f32,
        asset_server: &AssetServer,
    ) -> Self {
        // Setup the physics
        let rigid_body = RigidBodyBundle {
            position: (position, rotation).into(),
            velocity: velocity.into(),
            ccd: RigidBodyCcd {
                ccd_enabled: true,
                ..Default::default()
            }
            .into(),
            body_type: RigidBodyType::Dynamic.into(),
            ..Default::default()
        };
        let collider = ColliderBundle {
            // TODO: add proper size
            shape: ColliderShape::cuboid(1.0, 1.0).into(),
            mass_properties: ColliderMassProps::Density(2.0).into(),
            // Register to collision events
            flags: ActiveEvents::CONTACT_EVENTS.into(),
            ..Default::default()
        };

        // Load the svg
        let mesh = ColoredMeshBundle::new(asset_server.load("weapons/bow.svg"))
            .with_position(position.x, position.y);

        Self {
            rigid_body,
            collider,
            mesh,
            arrow: Arrow,
            projectile: Projectile,
        }
    }
}
