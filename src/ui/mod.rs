pub mod recruit_button;
pub mod spawn_bar;
pub mod theme;

use self::recruit_button::{RecruitButton, RecruitEvent};
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{App, Plugin},
};
use bevy_inspector_egui::RegisterInspectable;

/// The plugin to handle camera movements.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Get the FPS
        app.register_inspectable::<RecruitButton>()
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_event::<RecruitEvent>()
            // Added by inspector plugin, enable this when removing the inspector
            //.add_plugin(EguiPlugin)
            // Show the bottom spawn bar
            .add_system(spawn_bar::system)
            // Count down the recruit times
            .add_system(recruit_button::system)
            // Load the buttons
            .add_startup_system(recruit_button::setup);
    }
}
