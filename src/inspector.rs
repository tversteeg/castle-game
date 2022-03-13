#[cfg(feature = "inspector")]
mod inspector {
    pub use bevy_inspector_egui::{Inspectable, RegisterInspectable};

    use crate::map::terrain::Terrain;
    use crate::projectile::Projectile;
    use crate::unit::closest::{ClosestAlly, ClosestEnemy};
    use crate::unit::faction::Faction;
    use bevy::prelude::{App, Entity, Plugin, With};
    use bevy_inspector_egui::{
        widgets::{InspectorQuery, ResourceInspector},
        world_inspector::WorldInspectorPlugin,
        InspectorPlugin as BevyInspectorEguiPlugin,
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
                .add_plugin(BevyInspectorEguiPlugin::<Inspector>::new())
                .add_plugin(WorldInspectorPlugin::new());
        }
    }
}

#[cfg(not(feature = "inspector"))]
mod inspector {
    pub use mock_inspector_derive::Inspectable;

    use bevy::prelude::{App, Plugin};
    use bevy_egui::EguiPlugin;

    /// The inspectable trait to not do anything.
    pub trait Inspectable {}

    /// The register trait to not do anything.
    pub trait RegisterInspectable {
        fn register_inspectable<T: 'static>(&mut self) -> &mut Self {
            self
        }
    }

    impl RegisterInspectable for App {}

    /// The plugin to the inspection of ECS items.
    ///
    /// Doesn't do anything now.
    pub struct InspectorPlugin;

    impl Plugin for InspectorPlugin {
        fn build(&self, app: &mut App) {
            app.add_plugin(EguiPlugin);
        }
    }
}

pub use inspector::*;
