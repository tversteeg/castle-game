mod camera;
mod geometry;
mod map;
mod physics;
mod projectile;
mod ui;
mod unit;

use crate::{
    camera::CameraPlugin, map::MapPlugin, physics::PhysicsPlugin, projectile::ProjectilePlugin,
    ui::UiPlugin,
};
use bevy::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use geometry::GeometryPlugin;

fn main() {
    App::new()
        // Setup the window
        .insert_resource(WindowDescriptor {
            width: 300.0,
            height: 300.0,
            title: "Castle Game".to_string(),
            vsync: true,
            ..Default::default()
        })
        // Default, needed for physics
        .add_plugins(DefaultPlugins)
        // Debug view
        .add_plugin(WorldInspectorPlugin::new())
        // Transitions
        .add_plugin(EasingsPlugin)
        // The physics engine
        .add_plugin(PhysicsPlugin)
        // The camera movements
        .add_plugin(CameraPlugin)
        // The UI with the FPS counter
        .add_plugin(UiPlugin)
        // The map
        .add_plugin(MapPlugin)
        // The projectiles
        .add_plugin(ProjectilePlugin)
        // The geometry
        .add_plugin(GeometryPlugin)
        // Close when Esc is pressed
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_startup_system(projectile::rock::setup)
        .run();
}
