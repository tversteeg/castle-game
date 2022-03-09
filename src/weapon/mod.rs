pub mod bow;
pub mod spear;

use self::{bow::Bow, spear::Spear};
use bevy::prelude::{App, Plugin};
use bevy_inspector_egui::RegisterInspectable;

/// The plugin to manage the different weapons.
pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Bow>()
            .register_inspectable::<Spear>();
    }
}
