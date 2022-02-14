pub mod terrain;

use self::terrain::Terrain;
use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::RegisterInspectable;

/// The plugin to manage the map.
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Terrain>()
            .add_startup_system(terrain::setup);
    }
}
