pub mod resting;
pub mod rotation;

use self::{resting::RemoveAfterRestingFor, rotation::RotateToVelocityUntilContact};
use crate::inspector::RegisterInspectable;
use bevy::prelude::{App, Plugin, ResMut};
use bevy_rapier2d::{
    na::Vector2,
    physics::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    render::RapierRenderPlugin,
};

/// The plugin to manage basic physics.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<RemoveAfterRestingFor>()
            .register_inspectable::<RotateToVelocityUntilContact>()
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugin(RapierRenderPlugin)
            .add_startup_system(setup)
            .add_system(resting::system)
            .add_system(rotation::system)
            .add_system(rotation::contact_event_listener);
    }
}

/// Configure the rapier physics engine.
fn setup(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vector2::new(0.0, -9.2);
}
