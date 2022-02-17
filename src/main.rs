mod camera;
mod geometry;
mod log;
mod map;
mod physics;
mod projectile;
mod ui;
mod unit;

use crate::{
    camera::CameraPlugin, map::MapPlugin, physics::PhysicsPlugin, projectile::ProjectilePlugin,
    ui::UiPlugin,
};
use bevy::{
    ecs::schedule::ReportExecutionOrderAmbiguities,
    log::{Level, LogPlugin, LogSettings},
    prelude::*,
};
use bevy_easings::EasingsPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use geometry::GeometryPlugin;
use log::CustomLogPlugin;

fn main() {
    // Print pretty errors in wasm https://github.com/rustwasm/console_error_panic_hook
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        // Setup the window
        .insert_resource(WindowDescriptor {
            width: 800.0,
            height: 600.0,
            title: "Castle Game".to_string(),
            // The canvas ID when running in WASM
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#bevy_canvas".to_string()),
            ..Default::default()
        })
        // More verbose logging
        .insert_resource(LogSettings {
            level: Level::DEBUG,
            filter: "wgpu=error,bevy_render=info,winit=info,bevy_app=info,naga=info".to_string(),
        })
        // Tell us when some execution orders are ambiguous
        .insert_resource(ReportExecutionOrderAmbiguities)
        // Default, needed for physics, but use our own log plugin
        .add_plugins_with(DefaultPlugins, |group| group.disable::<LogPlugin>())
        // Our custom log plugin for tracing
        .add_plugin(CustomLogPlugin)
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
