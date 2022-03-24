use crate::{constants::Constants, projectile::event::ProjectileType};

use super::{arrow::ArrowBundle, event::ProjectileSpawnEvent};
use bevy::{
    math::Vec2,
    prelude::{AssetServer, Commands, EventReader, Res},
};
use bevy_rapier2d::prelude::RigidBodyVelocity;

/// The system for breaking on hard impacts.
pub fn spawn_event_listener(
    mut events: EventReader<ProjectileSpawnEvent>,
    mut commands: Commands,
    constants: Res<Constants>,
    asset_server: Res<AssetServer>,
) {
    for event in events.iter() {
        match event.projectile_type {
            ProjectileType::Direct => todo!(),
            ProjectileType::Arrow => {
                // Calculate the velocity if applicable
                let velocity = if let Some(target_position) = event.target_position {
                    shoot_velocity(event.start_position, target_position, &constants)
                } else {
                    // TODO: do something
                    Vec2::default()
                };

                // Spawn the arrow
                commands.spawn_bundle(ArrowBundle::new(
                    event.start_position,
                    RigidBodyVelocity {
                        linvel: velocity.into(),
                        angvel: 0.0,
                    },
                    &asset_server,
                    &constants,
                ));
            }
            ProjectileType::Rock => todo!(),
        }
    }
}

/// Calculate the velocity needed for shooting from point A to point B.
fn shoot_velocity(a: Vec2, b: Vec2, constants: &Constants) -> Vec2 {
    // X velocity, a constant
    let vx = (b.x - a.x) / constants.arrow.flight_time;
    // Y velocity, calculate the arch
    let vy = (b.y
        + 0.5
            * -constants.world.gravity
            * constants.arrow.flight_time
            * constants.arrow.flight_time
        - a.y)
        / constants.arrow.flight_time;

    Vec2::new(vx, vy)
}