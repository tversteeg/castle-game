use crate::physics::velocity::Velocity;
use bevy::prelude::{Component, Query};

#[derive(Component)]
pub struct Gravity(f64);

impl Default for Gravity {
    fn default() -> Self {
        Self(-9.2)
    }
}

/// Increase the velocity with the gravity.
pub fn system(mut query: Query<(&mut Velocity, &Gravity)>) {
    for (mut vel, grav) in query.iter_mut() {
        vel.y += grav.0;
    }
}
