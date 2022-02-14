pub mod breakable;
pub mod polygon;

use self::{
    breakable::Breakable,
    polygon::{PolygonBundle, PolygonComponent},
};
use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::RegisterInspectable;

/// The plugin to register geometry types.
pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<PolygonComponent>()
            .register_inspectable::<PolygonBundle>()
            .register_inspectable::<Breakable>()
            .add_system(breakable::system);
    }
}
