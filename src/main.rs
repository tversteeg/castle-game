mod camera;
mod color;
mod draw;
mod geometry;
mod inspector;
mod log;
mod map;
mod physics;
mod projectile;
mod ui;
mod unit;
mod weapon;

use crate::color::Palette;
use crate::geometry::GeometryPlugin;
use crate::inspector::InspectorPlugin;
use crate::log::CustomLogPlugin;
use crate::{
    camera::CameraPlugin, draw::DrawPlugin, map::MapPlugin, physics::PhysicsPlugin,
    projectile::ProjectilePlugin, ui::UiPlugin, unit::UnitPlugin, weapon::WeaponPlugin,
};
use bevy::{
    log::{Level, LogPlugin, LogSettings},
    prelude::*,
};
use bevy_easings::EasingsPlugin;

fn main() {
    // Print pretty errors in wasm https://github.com/rustwasm/console_error_panic_hook
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        // The background color
        .insert_resource(ClearColor(Palette::C21.into()))
        // Setup the window
        .insert_resource(WindowDescriptor {
            width: 1200.0,
            height: 800.0,
            title: "Castle Game".to_string(),
            // The canvas ID when running in WASM
            #[cfg(target_arch = "wasm32")]
            canvas: Some("#bevy_canvas".to_string()),
            ..Default::default()
        })
        // More verbose logging
        .insert_resource(LogSettings {
            level: Level::DEBUG,
            filter: "wgpu=error".to_string(),
        })
        // Default, needed for physics, but use our own log plugin
        .add_plugins_with(DefaultPlugins, |group| group.disable::<LogPlugin>())
        // Our custom log plugin for tracing
        .add_plugin(CustomLogPlugin)
        // Debug inspector
        .add_plugin(InspectorPlugin)
        // Transitions
        .add_plugin(EasingsPlugin)
        // Rendering
        .add_plugin(DrawPlugin)
        // The physics engine
        .add_plugin(PhysicsPlugin)
        // The units
        .add_plugin(UnitPlugin)
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
        // The weapons
        .add_plugin(WeaponPlugin)
        // Close when Esc is pressed
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_startup_system(projectile::rock::setup)
        .run();
}
