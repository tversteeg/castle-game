pub mod colored_mesh;
pub mod mesh;
pub mod svg;

use self::svg::SvgAssetLoader;
use bevy::prelude::{AddAsset, App, Msaa, Plugin};
use colored_mesh::ColoredMeshPlugin;

/// The plugin to manage rendering.
pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        // Smooth anti aliasing
        app.insert_resource(Msaa { samples: 4 })
            .init_asset_loader::<SvgAssetLoader>()
            .add_plugin(ColoredMeshPlugin)
            .add_startup_system(svg::setup);
    }
}
