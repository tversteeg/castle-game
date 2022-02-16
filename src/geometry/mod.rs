pub mod breakable;
pub mod polygon;
pub mod split;

use self::{
    breakable::{BreakEvent, Breakable},
    polygon::{PolygonBundle, PolygonComponent},
};
use bevy::{
    diagnostic::{Diagnostic, Diagnostics},
    prelude::{App, Plugin, ResMut},
};
use bevy_inspector_egui::RegisterInspectable;

/// The plugin to register geometry types.
pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<PolygonComponent>()
            .register_inspectable::<PolygonBundle>()
            .register_inspectable::<Breakable>()
            .add_event::<BreakEvent>()
            .add_system(breakable::system);
    }
}
