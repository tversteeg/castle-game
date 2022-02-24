use crate::{
    geometry::{
        breakable::{BreakEvent, Breakable},
        polygon::{PolygonBundle, ToColliderShape},
        split::Split,
    },
    physics::resting::RemoveAfterRestingFor,
};
use bevy::{
    core::Name,
    math::Vec2,
    prelude::{
        Assets, Color, Commands, Component, DespawnRecursiveExt, Entity, EventReader, Mesh, Query,
        ResMut, Transform,
    },
    sprite::ColorMaterial,
};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::{
    physics::{ColliderBundle, RigidBodyBundle, RigidBodyPositionSync},
    prelude::{
        ActiveEvents, ColliderMassProps, RigidBodyCcd, RigidBodyType, RigidBodyVelocity,
        RigidBodyVelocityComponent,
    },
};
use geo::{prelude::Area, LineString, Polygon};
use itertools::Itertools;
use rand::Rng;
use std::f32::consts::TAU;

/// A rock projectile that can split into smaller rocks on impact.
#[derive(Debug, Component, Inspectable)]
pub struct Rock {
    /// The rock's shape.
    #[inspectable(ignore)]
    shape: Polygon<f32>,
}

impl Rock {
    /// Generate a new rock.
    pub fn new(size: f32) -> Self {
        // Setup the random generator
        let mut rng = rand::thread_rng();

        // Generate a random polygon
        let edges: usize = rng.gen_range(5..10);

        // The increments in angles between each vertex
        let angle_increment = TAU / edges as f32;

        // Convert the edges to vertices
        let vertices = (0..edges)
            .into_iter()
            .map(|index| {
                // Get the absolute angle
                let angle = angle_increment * index as f32;

                // Generate the X & Y coordinates by taking the offset from the center and randomly
                // moving it a distance
                let x = angle.sin() * size * rng.gen_range::<f32, _>(0.6..1.0);
                let y = angle.cos() * size * rng.gen_range::<f32, _>(0.6..1.0);

                (x, y)
            })
            .collect::<Vec<_>>();

        // Create the polygon
        let shape = Polygon::new(LineString::from(vertices), vec![]);

        Self { shape }
    }

    /// Spawn the rock.
    pub fn spawn(
        self,
        position: Vec2,
        rotation: f32,
        velocity: RigidBodyVelocity,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) {
        // Get the area of the new rock
        let area = self.shape.unsigned_area();

        if area < 0.3 {
            // Don't spawn very small pieces
            return;
        }

        // Setup the rendering shape
        let polygon_bundle =
            PolygonBundle::new(&self.shape, Color::GRAY, position, meshes, materials);

        // Setup the physics
        let mut rigid_body_bundle = RigidBodyBundle {
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
        let collider_bundle = ColliderBundle {
            shape: self.shape.to_collider_shape().into(),
            mass_properties: ColliderMassProps::Density(2.0).into(),
            // Register to collision events
            flags: ActiveEvents::CONTACT_EVENTS.into(),
            // TODO
            // restitution: 0.1,
            // friction: 0.3,
            ..Default::default()
        };

        // If we are big chunk break even further
        if area > 2.0 {
            commands
                .spawn()
                .insert(self)
                .insert_bundle(polygon_bundle)
                .insert_bundle(rigid_body_bundle)
                .insert_bundle(collider_bundle)
                // Sync with bevy transform
                .insert(RigidBodyPositionSync::Discrete)
                .insert(Breakable::default())
                // Remove after a longer while
                .insert(RemoveAfterRestingFor::from_secs(3.0))
                .insert(Name::new("Rock"));
        } else {
            // Disable CCD for parts that can't break further
            rigid_body_bundle.ccd.0.ccd_enabled = false;

            commands
                .spawn()
                .insert(self)
                .insert_bundle(polygon_bundle)
                .insert_bundle(rigid_body_bundle)
                .insert_bundle(collider_bundle)
                // Sync with bevy transform
                .insert(RigidBodyPositionSync::Discrete)
                // Remove after a short while
                .insert(RemoveAfterRestingFor::from_secs(1.0))
                .insert(Name::new("Rock Fragment"));
        }
    }

    /// Fracture the rock.
    pub fn split(&self) -> Vec<Self> {
        self.shape.split().map(|shape| Self { shape }).collect()
    }
}

/// Load the rocks.
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    (0..20).into_iter().for_each(|y| {
        let rock = Rock::new(1.0 + y as f32 / 50.0);
        rock.spawn(
            Vec2::new(y as f32 * 2.0, 15.0 + y as f32 * 3.0),
            0.0,
            RigidBodyVelocity::default(),
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    });
}

/// The system for breaking on hard impacts.
pub fn break_event_listener(
    mut events: EventReader<BreakEvent>,
    query: Query<(Entity, &Rock, &Transform, &RigidBodyVelocityComponent)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    events
        .iter()
        // Remove duplicate entities
        // TODO: find if there's a better way of not throwing duplicate events in the first place
        .dedup_by(|event1, event2| event1.entity.id() == event2.entity.id())
        // Get the entities
        .filter_map(|break_event| query.get(break_event.entity).ok())
        .for_each(|(entity, rock, transform, velocity)| {
            // Get the XY position of the previous rock
            let position = transform.translation.truncate();

            // Remove the old rock
            commands.entity(entity).despawn_recursive();

            // Fracture the rock
            rock.split().into_iter().for_each(|new_rock| {
                // Only allow the rocks to split once
                new_rock.spawn(
                    position,
                    // TODO: fix rotation
                    transform.rotation.to_axis_angle().1,
                    velocity.0,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                );
            });
        });
}
