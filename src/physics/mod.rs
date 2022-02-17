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
        app.add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            .add_plugin(RapierRenderPlugin)
            .add_startup_system(setup);
    }
}

/// Configure the rapier physics engine.
fn setup(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vector2::new(0.0, -9.2);
}
