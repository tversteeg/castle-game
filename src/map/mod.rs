pub mod terrain;

use self::terrain::Terrain;
use bevy::prelude::{App, Plugin};
use crate::inspector::RegisterInspectable;

/// The plugin to manage the map.
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Terrain>()
            .insert_resource(Terrain::new())
            .add_startup_system(terrain::setup);
    }
}
