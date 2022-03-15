pub mod arrow;
pub mod event;
pub mod rock;
pub mod spawner;

use self::{arrow::Arrow, event::ProjectileSpawnEvent, rock::Rock};
use crate::inspector::RegisterInspectable;
use crate::{geometry::GeometrySystem, inspector::Inspectable};
use bevy::prelude::{App, Component, ParallelSystemDescriptorCoercion, Plugin};

/// Unit struct for determining the projectile.
#[derive(Component, Inspectable)]
pub struct Projectile;

/// The plugin to register projectiles.
pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Rock>()
            .register_inspectable::<Arrow>()
            .add_event::<ProjectileSpawnEvent>()
            .add_system(spawner::spawn_event_listener)
            .add_system(rock::break_event_listener.after(GeometrySystem::BreakEvent));
    }
}
