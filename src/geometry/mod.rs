pub mod polygon;

use self::polygon::{PolygonBundle, PolygonComponent};
use bevy::prelude::{App, Component, Plugin};
use bevy_inspector_egui::RegisterInspectable;

/// The plugin to register geometry types.
pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<PolygonComponent>()
            .register_inspectable::<PolygonBundle>();
    }
}
