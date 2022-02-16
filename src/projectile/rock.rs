use crate::geometry::{
    breakable::{BreakEvent, Breakable},
    polygon::PolygonBundle,
    split::Split,
};
use bevy::{
    core::Name,
    math::{Vec2, Vec3},
    prelude::{
        Assets, Color, Commands, Component, DespawnRecursiveExt, Entity, EventReader, Mesh, Query,
        ResMut, Transform,
    },
    sprite::ColorMaterial,
};
use bevy_inspector_egui::Inspectable;
use geo::{prelude::BoundingRect, Coordinate, LineString, Polygon, Rect};
use heron::{RigidBody, Velocity};
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
    pub fn new() -> Self {
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
                let x = angle.sin() * rng.gen_range::<f32, _>(0.8..1.0);
                let y = angle.cos() * rng.gen_range::<f32, _>(0.8..1.0);

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
        velocity: Velocity,
        breakable: bool,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
    ) {
        if breakable {
            commands
                .spawn_bundle(PolygonBundle::new(
                    &self.shape,
                    Color::GRAY,
                    position,
                    meshes,
                    materials,
                ))
                .insert(self)
                .insert(velocity)
                .insert(RigidBody::Dynamic)
                .insert(Breakable::default())
                .insert(Name::new("Rock"));
        } else {
            commands
                .spawn_bundle(PolygonBundle::new(
                    &self.shape,
                    Color::GRAY,
                    position,
                    meshes,
                    materials,
                ))
                .insert(self)
                .insert(velocity)
                .insert(RigidBody::Dynamic)
                .insert(Name::new("Broken Rock"));
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
        let rock = Rock::new();
        rock.spawn(
            Vec2::new(y as f32 * 2.0, 30.0 + y as f32 * 10.0),
            Velocity::from_linear(Vec3::default()),
            true,
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    });
}

/// The system for breaking on hard impacts.
pub fn break_event_listener(
    mut events: EventReader<BreakEvent>,
    query: Query<(Entity, &Rock, &Transform, &Velocity)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    events
        .iter()
        .filter_map(|break_event| query.get(break_event.entity).ok())
        .for_each(|(entity, rock, transform, velocity)| {
            // Get the XY position of the previous rock
            let pos = transform.translation.truncate();

            // Remove the old rock
            commands.entity(entity).despawn_recursive();

            // Fracture the rock
            rock.split().into_iter().for_each(|new_rock| {
                // Only allow the rocks to split once
                new_rock.spawn(
                    pos,
                    *velocity,
                    false,
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                );
            });
        });
}
