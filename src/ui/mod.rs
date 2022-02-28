pub mod spawn_bar;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{App, Plugin},
};

/// The plugin to handle camera movements.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Get the FPS
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            // Added by inspector plugin, enable this when removing the inspector
            //.add_plugin(EguiPlugin)
            // Show the bottom spawn bar
            .add_system(spawn_bar::system);
    }
}
