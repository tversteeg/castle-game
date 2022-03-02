pub mod definitions;
pub mod faction;
pub mod health;
pub mod human;
pub mod unit_type;
pub mod walk;

use self::{faction::Faction, health::Health, human::Human, unit_type::UnitType, walk::Walk};
use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::RegisterInspectable;

/// The plugin to register units.
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Walk>()
            .register_inspectable::<Faction>()
            .register_inspectable::<Human>()
            .register_inspectable::<Health>()
            .register_inspectable::<UnitType>()
            .add_system(walk::system)
            .add_system(definitions::recruit_event_listener);
    }
}
