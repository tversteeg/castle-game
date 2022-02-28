use bevy::prelude::{App, Msaa, Plugin};
use bevy_svg::prelude::SvgPlugin;

/// The plugin to manage rendering.
pub struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        // Smooth anti aliasing
        app.insert_resource(Msaa { samples: 4 })
            .add_plugin(SvgPlugin);
    }
}
