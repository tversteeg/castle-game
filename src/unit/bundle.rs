use super::{faction::Faction, health::Health, unit_type::UnitType, walk::Walk};
use crate::{
    draw::colored_mesh::ColoredMeshBundle,
    geometry::{polygon::Polygon, transform::TransformBuilder},
    map::terrain::{Terrain, ALLY_STARTING_POSITION, ENEMY_STARTING_POSITION},
    ui::recruit_button::RecruitEvent,
};
use bevy::{
    core::Name,
    prelude::{AssetServer, Bundle, Commands, EventReader, Res},
};
use bevy_inspector_egui::Inspectable;
use geo::{Coordinate, Rect};

/// Wrapper for a unit.
#[derive(Bundle, Inspectable)]
pub struct UnitBundle {
    faction: Faction,
    unit_type: UnitType,
    hitbox: Polygon,
    health: Health,
    walk: Walk,
    name: Name,
    #[bundle]
    #[inspectable(ignore)]
    mesh: ColoredMeshBundle,
}

impl UnitBundle {
    /// Construct a new unit.
    pub fn new(
        unit_type: UnitType,
        faction: Faction,
        terrain: &Terrain,
        _commands: &mut Commands,
        asset_server: &AssetServer,
    ) -> Self {
        // The starting position
        let x = match faction {
            Faction::Ally => ALLY_STARTING_POSITION,
            Faction::Enemy => ENEMY_STARTING_POSITION,
        };

        let y = terrain.height_at_x(x);

        // Use a simple square for the collision shape
        let hitbox: Polygon = Rect::new(Coordinate::zero(), Coordinate { x: 0.5, y: 1.8 })
            .to_polygon()
            .into();

        let mesh = ColoredMeshBundle::new(match (unit_type, faction) {
            (UnitType::Soldier, Faction::Ally) => asset_server.load("units/allies/character.svg"),
            (UnitType::Soldier, Faction::Enemy) => asset_server.load("units/enemies/character.svg"),
            (UnitType::Archer, Faction::Ally) => asset_server.load("units/allies/character.svg"),
            (UnitType::Archer, Faction::Enemy) => asset_server.load("units/enemies/character.svg"),
        })
        .with_position(x, y)
        .with_z_index(1.0);

        let health = Health::for_unit(unit_type, faction);

        // How fast the unit walks
        let walk = Walk::for_unit(unit_type, faction);

        let name = Name::new(format!("{} {}", faction.to_string(), unit_type.to_string()));

        Self {
            faction,
            unit_type,
            mesh,
            walk,
            name,
            health,
            hitbox,
        }
    }

    /// Spawn the unit.
    pub fn spawn(self, commands: &mut Commands) {
        commands.spawn_bundle(self);
    }
}

/// The system for recruiting new allied units.
pub fn recruit_event_listener(
    mut events: EventReader<RecruitEvent>,
    terrain: Res<Terrain>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    events
        .iter()
        // Spawn units based on what unit types have been send by the recruit button
        .for_each(|recruit_event| {
            let unit = UnitBundle::new(
                recruit_event.0,
                Faction::Ally,
                &terrain,
                &mut commands,
                &asset_server,
            );

            unit.spawn(&mut commands);
        });
}
