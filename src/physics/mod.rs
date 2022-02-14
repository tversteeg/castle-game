pub mod gravity;
pub mod position;
pub mod velocity;

use bevy::{
    core::FixedTimestep,
    math::Vec3,
    prelude::{App, Plugin, SystemSet},
};
use heron::prelude::Gravity;
use heron::PhysicsPlugin as HeronPhysicsPlugin;

/// How many times the physics get updated.
pub const TIME_STEP: f64 = 1.0 / 60.0;

/// The plugin to manage basic physics.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(HeronPhysicsPlugin::default())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(TIME_STEP))
                    .with_system(position::system)
                    .with_system(velocity::system)
                    .with_system(gravity::system),
            )
            .insert_resource(Gravity::from(Vec3::new(0.0, -9.2, 0.0)));
    }
}
