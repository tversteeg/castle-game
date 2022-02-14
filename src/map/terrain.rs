use crate::geometry::polygon::PolygonBundle;
use bevy::{
    core::Name,
    math::Vec2,
    prelude::{Assets, Color, Commands, Component, GlobalTransform, Mesh, ResMut},
    sprite::ColorMaterial,
};
use bevy_inspector_egui::Inspectable;
use geo::{LineString, Polygon};
use heron::RigidBody;
use rand::Rng;

/// The destructible ground.
#[derive(Component, Inspectable)]
pub struct Terrain {
    /// The vector mesh.
    #[inspectable(ignore)]
    shape: Polygon<f32>,
}

impl Terrain {
    /// Create a new randomly generated terrain.
    pub fn new(points: usize) -> Self {
        // Setup the random generator
        let mut rng = rand::thread_rng();

        // Generate the shape
        let vertices = (0..=points)
            .into_iter()
            .map(|index| {
                let x = -50.0 + (index as f32 / points as f32) * 100.0;
                // Generate a random height
                let y = rng.gen_range::<f32, _>(5.0..17.0);

                (x, y)
            })
            // Add the required edges to create a square
            .chain([(50.0, -5.0), (-50.0, -5.0)].into_iter())
            .collect::<Vec<_>>();

        let shape = Polygon::new(LineString::from(vertices), vec![]);

        Self { shape }
    }
}

/// Load the sprite.
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let terrain = Terrain::new(20);

    commands
        .spawn_bundle(PolygonBundle::new(
            terrain.shape,
            Color::GRAY,
            Vec2::new(0.0, 0.0),
            &mut meshes,
            &mut materials,
        ))
        .insert(Name::new("Terrain"))
        .insert(RigidBody::Static);
}
