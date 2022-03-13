pub mod breakable;
pub mod polygon;
pub mod split;
pub mod transform;

use self::{
    breakable::{BreakEvent, Breakable},
    polygon::{Polygon, PolygonShapeBundle},
};
use bevy::prelude::{App, ParallelSystemDescriptorCoercion, Plugin, SystemLabel};
use crate::inspector::RegisterInspectable;

/// For prioritizing systems in relation to our systems.
#[derive(Debug, Clone, Hash, PartialEq, Eq, SystemLabel)]
pub enum GeometrySystem {
    BreakEvent,
}

/// The plugin to register geometry types.
pub struct GeometryPlugin;

impl Plugin for GeometryPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Polygon>()
            .register_inspectable::<PolygonShapeBundle>()
            .register_inspectable::<Breakable>()
            .add_event::<BreakEvent>()
            .add_system(breakable::system.label(GeometrySystem::BreakEvent));
    }
}
