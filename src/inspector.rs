use crate::map::terrain::Terrain;
use crate::projectile::Projectile;
use crate::unit::closest::{ClosestAlly, ClosestEnemy};
use crate::unit::faction::Faction;
use bevy::prelude::{App, Entity, Plugin, With};

use bevy_inspector_egui::widgets::{
    InspectorQuery, ResourceInspector,
};
use bevy_inspector_egui::{
    Inspectable, InspectorPlugin as BevyInspectorEguiPlugin,
};

/// The inspector with all the subwindows.
#[derive(Default, Inspectable)]
pub struct Inspector {
    #[inspectable(label = "Resources", collapse)]
    resources: Resources,
    #[inspectable(label = "Units", collapse)]
    units: InspectorQuery<Entity, With<Faction>>,
    #[inspectable(label = "Projectiles", collapse)]
    projectiles: InspectorQuery<Entity, With<Projectile>>,
}

/// Show these resources.
#[derive(Default, Inspectable)]
pub struct Resources {
    closest_ally: ResourceInspector<ClosestAlly>,
    closest_enemy: ResourceInspector<ClosestEnemy>,
    terrain: ResourceInspector<Terrain>,
}

/// The plugin to the inspection of ECS items.
pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app
            // The debug view
            .add_plugin(BevyInspectorEguiPlugin::<Inspector>::new());
    }
}
