pub mod terrain;

use self::terrain::Terrain;
use crate::{constants::TerrainConstants, inspector::RegisterInspectable};
use bevy::prelude::{App, Plugin};

/// The plugin to manage the map.
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Terrain>()
            .insert_resource(Terrain::new(&TerrainConstants::default()))
            .add_startup_system(terrain::setup);
    }
}
