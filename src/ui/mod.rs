use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{App, Plugin},
};

pub mod fps;

/// The plugin to handle camera movements.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Get the FPS
        app.add_plugin(FrameTimeDiagnosticsPlugin::default());

        // Show the FPS text
        app.add_system(fps::system);
        app.add_startup_system(fps::setup);
    }
}
