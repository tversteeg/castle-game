mod camera;
mod map;
mod physics;
mod projectile;
mod ui;
mod unit;

use crate::{camera::CameraPlugin, physics::PhysicsPlugin, ui::UiPlugin};
use bevy::prelude::*;
use bevy_easings::EasingsPlugin;

fn main() {
    App::new()
        // Setup the window
        .insert_resource(WindowDescriptor {
            width: 300.0,
            height: 300.0,
            title: "Castle Game".to_string(),
            ..Default::default()
        })
        // Default, needed for physics
        .add_plugins(DefaultPlugins)
        // Transitions
        .add_plugin(EasingsPlugin)
        // The physics engine
        .add_plugin(PhysicsPlugin)
        // The camera movements
        .add_plugin(CameraPlugin)
        // The UI with the FPS counter
        .add_plugin(UiPlugin)
        .add_startup_system(map::setup_ground.system())
        .add_startup_system(setup)
        // Close when Esc is pressed
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

/// Setup the basic bundles and objects.
fn setup(mut commands: Commands) {
    for x in 0..1000 {
        projectile::arrow::spawn(
            [0.0, 5.0 + x as f64].into(),
            [1.0, 0.0].into(),
            &mut commands,
        );
    }
}
