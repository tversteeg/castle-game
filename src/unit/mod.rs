pub mod closest;
pub mod definitions;
pub mod faction;
pub mod health;
pub mod human;
pub mod spawner;
pub mod unit_type;
pub mod walk;

use self::{
    closest::{ClosestAlly, ClosestEnemy},
    faction::Faction,
    health::Health,
    human::Human,
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
            .register_inspectable::<Human>()
            .register_inspectable::<Health>()
            .register_inspectable::<UnitType>()
            .register_inspectable::<EnemySpawner>()
            .register_inspectable::<ResourceInspector<ClosestEnemy>>()
            .insert_resource(ClosestEnemy::default())
            .insert_resource(ClosestAlly::default())
            .add_system(walk::system)
            .add_system(definitions::recruit_event_listener)
            .add_system(spawner::system)
            .add_system(closest::system)
            .add_startup_system(spawner::setup);
    }
}
