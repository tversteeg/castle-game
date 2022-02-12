use bevy::{
    math::Vec2,
    prelude::{Component, Query, Transform},
};

/// World position of an object.
#[derive(Component)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Position {
    /// Instantiatie with zero for all values.
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl From<Vec2> for Position {
    fn from(vec: Vec2) -> Self {
        Self {
            x: vec.x as f64,
            y: vec.y as f64,
        }
    }
}

impl From<[f64; 2]> for Position {
    fn from(vec: [f64; 2]) -> Self {
        Self {
            x: vec[0],
            y: vec[1],
        }
    }
}

/// Place the graphics positions to our positions.
pub fn system(mut query: Query<(&mut Transform, &Position)>) {
    for (mut trans, pos) in query.iter_mut() {
        trans.translation.x = pos.x as f32;
        trans.translation.y = pos.y as f32;
    }
}
