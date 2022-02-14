use crate::geometry::polygon::PolygonBundle;
use bevy::{
    core::Name,
    math::Vec2,
    prelude::{Assets, Color, Commands, Component, Mesh, ResMut},
    sprite::ColorMaterial,
};
use bevy_inspector_egui::Inspectable;
use geo::{LineString, Polygon};
use heron::RigidBody;
use rand::Rng;
use std::f32::consts::TAU;

/// A rock projectile that can split into smaller rocks on impact.
#[derive(Component, Inspectable)]
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
                let x = angle.sin() * rng.gen_range::<f32, _>(0.5..1.0);
                let y = angle.cos() * rng.gen_range::<f32, _>(0.5..1.0);

                (x, y)
            })
            .collect::<Vec<_>>();

        // Create the polygon
        let shape = Polygon::new(LineString::from(vertices), vec![]);

        Self { shape }
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

        commands
            .spawn_bundle(PolygonBundle::new(
                rock.shape,
                Color::GRAY,
                Vec2::new(0.0, 10.0 + y as f32),
                &mut meshes,
                &mut materials,
            ))
            .insert(Name::new("Rock"))
            .insert(RigidBody::Dynamic);
    });
}
