use crate::{
    color::Palette,
    geometry::polygon::{Polygon, PolygonShapeBundle, ToColliderShape},
};
use bevy::{
    core::Name,
    math::Vec2,
    prelude::{Assets, Commands, Mesh, Res, ResMut},
    utils::tracing,
};
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::{
    physics::{ColliderBundle, RigidBodyBundle},
    prelude::{ActiveEvents, RigidBodyType},
};
use geo::{prelude::BoundingRect, Coordinate, LineString, Rect};
use noise::{Fbm, NoiseFn, Seedable};

/// How many height points should be calculated for the terrain.
pub const HEIGHT_POINTS: usize = 100;
/// Total width of the terrain.
pub const TERRAIN_WIDTH: f32 = 300.0;
/// Minimum height of the terrain.
pub const TERRAIN_MIN_HEIGHT: f32 = 6.0;
/// Maximum height of the terrain.
pub const TERRAIN_MAX_HEIGHT: f32 = 10.0;
/// The scale of the noise, will influence which X points will be get as sample.
pub const NOISE_SCALE: f64 = 0.01;

/// The destructible ground.
#[derive(Inspectable)]
pub struct Terrain {
    /// The vector mesh.
    shape: Polygon,
    /// The heights of the top for detecting collisions.
    #[inspectable(ignore)]
    top_coordinates: LineString<f32>,
    /// The bounding box for the top, to improve collision detection speed.
    #[inspectable(ignore)]
    top_coordinates_bounding_box: Rect<f32>,
}

impl Terrain {
    /// Create a new randomly generated terrain.
    #[tracing::instrument(name = "generating terrain")]
    pub fn new() -> Self {
        // Setup the noise generator
        let noise = Fbm::new().set_seed(fastrand::u32(..));

        // Generate the top shape
        let top_coordinates = (0..=HEIGHT_POINTS)
            .into_iter()
            .map(|index| {
                let x = (index as f32 / HEIGHT_POINTS as f32) * TERRAIN_WIDTH;

                // Generate a random height
                let y = noise.get([x as f64 * NOISE_SCALE, 0.0]) as f32
                    * (TERRAIN_MAX_HEIGHT - TERRAIN_MIN_HEIGHT)
                    + TERRAIN_MIN_HEIGHT;

                (x, y)
            })
            .collect::<Vec<_>>();

        // Add the required edges to create a square
        let vertices = top_coordinates
            .iter()
            .map(|coord| *coord)
            .chain([(TERRAIN_WIDTH, -5.0), (0.0, -5.0)].into_iter())
            .collect::<Vec<_>>();

        // Create the polygon
        let shape = Polygon::new(LineString::from(vertices), vec![]);

        // Create the top shape
        let top_coordinates =
            LineString(top_coordinates.into_iter().map(Coordinate::from).collect());

        // Get the bounding box for quick calculations
        let top_coordinates_bounding_box = top_coordinates
            .bounding_rect()
            .expect("Could not create bounding box for top coordinates of terrain");

        Self {
            shape,
            top_coordinates,
            top_coordinates_bounding_box,
        }
    }

    /// Get the terrain height at the horizontal position.
    pub fn height_at_x(&self, x: f32) -> f32 {
        if x <= self.top_coordinates_bounding_box.min().x
            || x > self.top_coordinates_bounding_box.max().x
        {
            // The base terrain height out of bounds is always 0
            0.0
        } else {
            // Find the line matching our x coordinate
            let line = self
                .top_coordinates
                .lines()
                .find(|line| x >= line.start.x && x < line.end.x)
                .expect("Could not find line within bounding box for collision");

            // Return the line height at that point
            line.start.y + line.slope() * (x - line.start.x)
        }
    }

    /// Check whether a point collides with the ground.
    pub fn collides(&self, x: f32, y: f32) -> bool {
        if y <= self.top_coordinates_bounding_box.min().y {
            // No collision when it's too high or outside of the horizontal bounds
            false
        } else if y > self.top_coordinates_bounding_box.max().y {
            // Always collide when in horizontal bounds and lower than the ground's collision box
            true
        } else {
            y >= self.height_at_x(x)
        }
    }
}

/// Load the sprite.
pub fn setup(terrain: Res<Terrain>, mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands
        .spawn_bundle(PolygonShapeBundle::new(
            terrain.shape.clone(),
            Palette::C11.into(),
            Vec2::ZERO,
            &mut meshes,
        ))
        .insert(Name::new("Terrain Polygon"));

    commands
        .spawn_bundle(RigidBodyBundle {
            body_type: RigidBodyType::Static.into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            // TODO: use heightmap collider
            shape: terrain.shape.to_collider_shape().into(),
            // Register to collision events
            flags: (ActiveEvents::INTERSECTION_EVENTS | ActiveEvents::CONTACT_EVENTS).into(),
            // TODO
            // restitution: 0.2,
            // friction: 0.4,
            ..Default::default()
        })
        .insert(Name::new("Terrain Rigid Body"));
}
