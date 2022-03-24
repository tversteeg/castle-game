#[cfg(feature = "inspector")]
mod inspector {
    pub use bevy_inspector_egui::{Inspectable, RegisterInspectable};

    use crate::{
        constants::Constants,
        map::terrain::Terrain,
        projectile::Projectile,
        unit::{
            closest::{ClosestAlly, ClosestEnemy},
            unit_type::UnitType,
        },
        weapon::Weapon,
    };
    use bevy::{
        prelude::{App, Entity, Plugin, With},
        sprite::Mesh2dHandle,
    };
    use bevy_inspector_egui::{
        widgets::{InspectorQuery, ResourceInspector},
        InspectableRegistry, InspectorPlugin as BevyInspectorEguiPlugin,
    };
    use bevy_inspector_egui_rapier::InspectableRapierPlugin;

    /// The inspector with all the subwindows.
    #[derive(Default, Inspectable)]
    pub struct Inspector {
        #[inspectable(label = "Constants", collapse)]
        constants: ResourceInspector<Constants>,
        #[inspectable(label = "Resources", collapse)]
        resources: Resources,
        #[inspectable(label = "Units", collapse)]
        units: InspectorQuery<Entity, With<UnitType>>,
        #[inspectable(label = "Weapons", collapse)]
        weapons: InspectorQuery<Entity, With<Weapon>>,
        #[inspectable(label = "Projectiles", collapse)]
        projectiles: InspectorQuery<Entity, With<Projectile>>,
    }

    /// Show these resources.
    #[derive(Default, Inspectable)]
    pub struct Resources {
        #[inspectable(label = "Closest Ally")]
        closest_ally: ResourceInspector<ClosestAlly>,
        #[inspectable(label = "Closest Enemy")]
        closest_enemy: ResourceInspector<ClosestEnemy>,
        #[inspectable(label = "Terrain", collapse)]
        terrain: ResourceInspector<Terrain>,
    }

    /// The plugin to the inspection of ECS items.
    pub struct InspectorPlugin;

    impl Plugin for InspectorPlugin {
        fn build(&self, app: &mut App) {
            app
                // Rapier structs
                .add_plugin(InspectableRapierPlugin)
                // The debug view
                .add_plugin(BevyInspectorEguiPlugin::<Inspector>::new());

            // Get the registry for inspectables to add our own implementation for custom types
            let mut inspectable_registry = app
                .world
                .get_resource_or_insert_with(InspectableRegistry::default);

            // "Implement" inspectable trait for mesh
            inspectable_registry
                .register_raw::<Mesh2dHandle, _>(crate::draw::mesh::inspector::mesh_inspectable);
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
