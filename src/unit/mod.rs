pub mod bundle;
pub mod closest;
pub mod faction;
pub mod health;
pub mod spawner;
pub mod unit_type;
pub mod walk;

use self::{
    bundle::UnitBundle,
    closest::{ClosestAlly, ClosestEnemy},
    faction::Faction,
    health::Health,
    spawner::EnemySpawner,
    unit_type::UnitType,
    walk::Walk,
};
use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::{widgets::ResourceInspector, RegisterInspectable};

/// The plugin to register units.
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Walk>()
            .register_inspectable::<Faction>()
            .register_inspectable::<Health>()
            .register_inspectable::<UnitType>()
            .register_inspectable::<EnemySpawner>()
            .register_inspectable::<UnitBundle>()
            .register_inspectable::<ResourceInspector<ClosestEnemy>>()
            .insert_resource(ClosestEnemy::default())
            .insert_resource(ClosestAlly::default())
            .add_system(walk::system)
            .add_system(bundle::recruit_event_listener)
            .add_system(spawner::system)
            .add_system(closest::system)
            .add_startup_system(spawner::setup);
    }
}
