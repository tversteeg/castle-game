use std::time::Duration;

use bevy::{
    core::Time,
    prelude::{Commands, Component, Entity, Query, Res},
};
use crate::inspector::Inspectable;
use bevy_rapier2d::prelude::RigidBodyVelocityComponent;

/// The treshold of kinetic energy at which point the timer goes down.
const KINETIC_ENERGY_TRESHOLD: f32 = 1.0;

/// Remove this entity after not moving for the specified time.
#[derive(Debug, Component, Inspectable)]
pub struct RemoveAfterRestingFor {
    /// The time that already elapsed, will be reset when the object starts moving.
    elapsed: Duration,
    /// When elapsed exceeds this the entity will be removed.
    time: Duration,
}

impl RemoveAfterRestingFor {
    /// Create the component with the time as seconds.
    pub fn from_secs(seconds: f32) -> Self {
        Self {
            elapsed: Duration::ZERO,
            time: Duration::from_secs_f32(seconds),
        }
    }
}

/// Check if the object is resting and remove it if isn't for the specified time.
pub fn system(
    mut query: Query<(
        Entity,
        &mut RemoveAfterRestingFor,
        &RigidBodyVelocityComponent,
    )>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut resting, velocity) in query.iter_mut() {
        if velocity.pseudo_kinetic_energy() <= KINETIC_ENERGY_TRESHOLD {
            // Subtract the time
            resting.elapsed += time.delta();

            // Remove the entity if the time elapsed
            if resting.elapsed > resting.time {
                commands.entity(entity).despawn();
            }
        } else if !resting.elapsed.is_zero() {
            // Reset the time
            resting.elapsed = Duration::ZERO;
        }
    }
}
