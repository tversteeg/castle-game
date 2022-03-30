use super::{faction::Faction, health::Health, unit_type::UnitType, walk::Walk};
use crate::constants::Constants;
use crate::inspector::Inspectable;
use crate::{
    draw::colored_mesh::ColoredMeshBundle,
    geometry::{polygon::Polygon, transform::TransformBuilder},
    map::terrain::Terrain,
    ui::recruit_button::RecruitEvent,
    weapon::{bow::BowBundle, spear::SpearBundle},
};
use bevy::{
    core::Name,
    prelude::{AssetServer, BuildChildren, Bundle, Commands, EventReader, Res},
};
use bevy_rapier2d::physics::ColliderBundle;
use bevy_rapier2d::prelude::{ColliderShape, ColliderType};
use geo::{Coordinate, Rect};

/// Wrapper for a unit.
#[derive(Bundle, Inspectable)]
pub struct UnitBundle {
    /// To which side the unit belongs.
    faction: Faction,
    /// What type of unit it is.
    unit_type: UnitType,
    /// How much health the unit has.
    health: Health,
    /// How fast the unit walks.
    walk: Walk,
    /// The unit mesh.
    #[bundle]
    mesh: ColoredMeshBundle,
    /// The collider for detecting collisions, mainly with projectiles.
    #[bundle]
    #[inspectable(ignore)]
    collider: ColliderBundle,
    /// The name of the unit.
    name: Name,
}

impl UnitBundle {
    /// Construct a new unit.
    pub fn new(
        unit_type: UnitType,
        faction: Faction,
        terrain: &Terrain,
        asset_server: &AssetServer,
        constants: &Constants,
    ) -> Self {
        // The starting position
        let x = match faction {
            Faction::Ally => constants.terrain.ally_starting_position,
            Faction::Enemy => constants.terrain.enemy_starting_position,
        };
        let y = terrain.height_at_x(x);

        let mesh = ColoredMeshBundle::new(match (unit_type, faction) {
            (UnitType::Soldier, Faction::Ally) => asset_server.load("units/allies/character.svg"),
            (UnitType::Soldier, Faction::Enemy) => asset_server.load("units/enemies/character.svg"),
            (UnitType::Archer, Faction::Ally) => asset_server.load("units/allies/character.svg"),
            (UnitType::Archer, Faction::Enemy) => asset_server.load("units/enemies/character.svg"),
        })
        .with_position(x, y)
        .with_z_index(1.0);

        let health = Health::for_unit(unit_type, faction, constants);

        // How fast the unit walks
        let walk = Walk::for_unit(unit_type, faction, constants);

        // The collision shape for the unit
        let collider = ColliderBundle {
            shape: ColliderShape::cuboid(1.0, 2.0).into(),
            collider_type: ColliderType::Sensor.into(),
            ..Default::default()
        };

        let name = Name::new(format!("{} {}", faction.to_string(), unit_type.to_string()));

        Self {
            faction,
            unit_type,
            mesh,
            walk,
            name,
            health,
            collider,
        }
    }

    /// Spawn the unit.
    pub fn spawn(self, commands: &mut Commands, asset_server: &AssetServer, constants: &Constants) {
        let unit_type = self.unit_type;
        let faction = self.faction;
        commands.spawn_bundle(self).with_children(|parent| {
            match unit_type {
                // TODO: Something weird happens here
                UnitType::Soldier => parent.spawn_bundle(SpearBundle::new(faction, asset_server)),
                UnitType::Archer => {
                    parent.spawn_bundle(BowBundle::new(faction, asset_server, constants))
                }
            };
        });
    }
}

/// The system for recruiting new allied units.
pub fn recruit_event_listener(
    mut events: EventReader<RecruitEvent>,
    terrain: Res<Terrain>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    constants: Res<Constants>,
) {
    events
        .iter()
        // Spawn units based on what unit types have been send by the recruit button
        .for_each(|recruit_event| {
            let unit = UnitBundle::new(
                recruit_event.0,
                Faction::Ally,
                &terrain,
                &asset_server,
                &constants,
            );

            unit.spawn(&mut commands, &asset_server, &constants);
        });
}
