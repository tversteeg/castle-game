use crate::{
    constants::Constants, draw::colored_mesh::ColoredMeshBundle,
    geometry::transform::TransformBuilder, inspector::Inspectable,
    physics::resting::RemoveAfterRestingFor, projectile::Projectile,
};
use bevy::{
    core::Name,
    math::Vec2,
    prelude::{AssetServer, Bundle, Component},
};
use bevy_rapier2d::{
    physics::{ColliderBundle, RigidBodyBundle, RigidBodyPositionSync},
    prelude::{
        ActiveEvents, ColliderMassProps, ColliderShape, RigidBodyCcd, RigidBodyType,
        RigidBodyVelocity,
    },
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
    /// Remove the component after resting for a specific time.
    remove_after_resting_for: RemoveAfterRestingFor,
    /// Sync with bevy transform.
    #[inspectable(ignore)]
    position_sync: RigidBodyPositionSync,
    /// The mesh itself for the arrow.
    #[bundle]
    mesh: ColoredMeshBundle,
    /// Physics.
    #[bundle]
    #[inspectable(ignore)]
    rigid_body: RigidBodyBundle,
    /// Detecting collisions.
    #[bundle]
    #[inspectable(ignore)]
    collider: ColliderBundle,
    /// Name of the entity.
    name: Name,
}

impl ArrowBundle {
    /// Shoot a new arrow.
    pub fn new(
        position: Vec2,
        velocity: RigidBodyVelocity,
        rotation: f32,
        asset_server: &AssetServer,
        constants: &Constants,
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
            shape: ColliderShape::cuboid(0.05, 0.5).into(),
            mass_properties: ColliderMassProps::Density(2.0).into(),
            // Register to collision events
            flags: ActiveEvents::CONTACT_EVENTS.into(),
            ..Default::default()
        };

        // Load the svg
        let mesh = ColoredMeshBundle::new(asset_server.load("projectiles/arrow.svg"))
            .with_position(position.x, position.y);

        // When to remove the arrow
        let remove_after_resting_for =
            RemoveAfterRestingFor::from_secs(constants.arrow.remove_after_resting_for);

        let name = Name::new("Arrow");

        Self {
            rigid_body,
            collider,
            mesh,
            name,
            remove_after_resting_for,
            arrow: Arrow,
            projectile: Projectile,
            position_sync: RigidBodyPositionSync::Discrete,
        }
    }
}
