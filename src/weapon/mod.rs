pub mod bow;
pub mod discharge;
pub mod spear;

use self::{bow::Bow, discharge::Discharge, spear::Spear};
use crate::inspector::RegisterInspectable;
use bevy::prelude::{App, Plugin};

/// The plugin to manage the different weapons.
pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Bow>()
            .register_inspectable::<Spear>()
            .register_inspectable::<Discharge>()
            .add_system(discharge::system);
    }
}
