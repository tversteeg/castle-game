use crate::map::terrain::Terrain;
use bevy::{
    core::Time,
    prelude::{Component, Query, Res, Transform},
};
use bevy_inspector_egui::Inspectable;

/// Allow a unit to walk across the map.
#[derive(Debug, Component, Inspectable)]
pub struct Walk {
    /// How many meters per second the unit walks.
    speed: f32,
}

impl Walk {
    /// Create a new walk component with the specified speed in meters/second.
    pub fn new(speed: f32) -> Self {
        Self { speed }
    }
}

/// Let a unit walk over the ground with the specified speed.
pub fn system(mut query: Query<(&Walk, &mut Transform)>, terrain: Res<Terrain>, time: Res<Time>) {
    for (walk, mut transform) in query.iter_mut() {
        // Walk horizontally
        transform.translation.x += walk.speed * time.delta_seconds();

        // Follow the curve of the terrain vertically
        transform.translation.y = terrain.height_at_x(transform.translation.x);
    }
}
