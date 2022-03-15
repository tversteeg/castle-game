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
use crate::inspector::RegisterInspectable;
use bevy::{
    core::FixedTimestep,
    prelude::{App, CoreStage, Plugin, StageLabel, SystemStage},
};

/// The label used for the slow stage.
const SLOW_STAGE_LABEL: &str = "unit_slow_stage";

/// A stage that's updated not every frame.
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct SlowUpdateStage;

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
            .insert_resource(ClosestEnemy::default())
            .insert_resource(ClosestAlly::default())
            .add_system(walk::system)
            .add_system(bundle::recruit_event_listener)
            .add_system(spawner::system)
            // Check the closest distance with a different interval
            .add_stage_after(
                CoreStage::Update,
                SlowUpdateStage,
                SystemStage::parallel()
                    // Run the system twice per second
                    .with_run_criteria(FixedTimestep::step(0.5).with_label(SLOW_STAGE_LABEL))
                    .with_system(closest::system),
            )
            .add_startup_system(spawner::setup);
    }
}
