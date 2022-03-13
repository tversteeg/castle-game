pub mod bow;
pub mod spear;

use self::{bow::Bow, spear::Spear};
use crate::inspector::RegisterInspectable;
use bevy::prelude::{App, Plugin};

/// The plugin to manage the different weapons.
pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Bow>()
            .register_inspectable::<Spear>();
    }
}
