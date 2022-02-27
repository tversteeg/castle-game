pub mod arrow;
pub mod rock;

use self::{arrow::Arrow, rock::Rock};
use crate::geometry::GeometrySystem;
use bevy::prelude::{App, Component, ParallelSystemDescriptorCoercion, Plugin};
use bevy_inspector_egui::RegisterInspectable;

#[derive(Component)]
pub struct Projectile;

/// The plugin to register projectiles.
pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Rock>()
            .register_inspectable::<Arrow>()
            .add_system(rock::break_event_listener.after(GeometrySystem::BreakEvent));
    }
}
