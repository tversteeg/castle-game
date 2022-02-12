use crate::physics::{position::Position, TIME_STEP};
use bevy::{
    math::Vec2,
    prelude::{Component, Query},
};

/// Offset that gets added to world position every second.
#[derive(Component)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
}

impl Velocity {
    /// Instantiatie with zero for all values.
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl From<Vec2> for Velocity {
    fn from(vec: Vec2) -> Self {
        Self {
            x: vec.x as f64,
            y: vec.y as f64,
        }
    }
}

impl From<[f64; 2]> for Velocity {
    fn from(vec: [f64; 2]) -> Self {
        Self {
            x: vec[0],
            y: vec[1],
        }
    }
}

/// Add the velocities to the positions.
pub fn system(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.x * TIME_STEP;
        pos.y += vel.y * TIME_STEP;
    }
}
