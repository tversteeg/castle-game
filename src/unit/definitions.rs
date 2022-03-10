use super::{health::Health, unit_type::UnitType, walk::Walk};
use crate::{
    draw::colored_mesh::ColoredMeshBundle,
    geometry::{polygon::Polygon, transform::TransformBuilder},
    map::terrain::{Terrain, TERRAIN_WIDTH},
    ui::recruit_button::RecruitEvent,
    unit::faction::Faction,
    weapon::{bow::BowBundle, spear::SpearBundle},
};
use bevy::{
    core::Name,
    prelude::{AssetServer, BuildChildren, Commands, EventReader, Handle, Mesh, Res},
};
use geo::{Coordinate, Rect};

/// The starting position x coordinate for ally units.
pub const ALLY_STARTING_POSITION: f32 = 5.0;
/// The starting position x coordinate for enemy units.
pub const ENEMY_STARTING_POSITION: f32 = TERRAIN_WIDTH - 5.0;

/// Spawn a melee soldier.
pub fn spawn_melee_soldier(
    faction: Faction,
    terrain: &Terrain,
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    // The starting position
    let x = match faction {
        Faction::Ally => ALLY_STARTING_POSITION,
        Faction::Enemy => ENEMY_STARTING_POSITION,
    };
    let y = terrain.height_at_x(x);

    // Use a simple square for the collision shape
    let polygon: Polygon = Rect::new(Coordinate::zero(), Coordinate { x: 0.5, y: 1.8 })
        .to_polygon()
        .into();

    commands
        .spawn_bundle(
            ColoredMeshBundle::new(match faction {
                Faction::Ally => asset_server.load("units/allies/character.svg"),
                Faction::Enemy => asset_server.load("units/enemies/character.svg"),
            })
            .with_position(x, y)
            .with_z_index(1.0),
        )
        .insert(polygon)
        .insert(faction)
        .insert(Health::new(100.0))
        .insert(Walk::new(match faction {
            Faction::Ally => 1.0,
            Faction::Enemy => -1.0,
        }))
        .insert(Name::new(match faction {
            Faction::Ally => "Allied Melee Soldier",
            Faction::Enemy => "Enemy Melee Soldier",
        }))
        .with_children(|parent| {
            parent
                .spawn_bundle(
                    SpearBundle::new(asset_server.load("weapons/spear.svg"))
                        .with_rotation(-20.0)
                        .with_position(0.5, 1.0),
                )
                .insert(Name::new("Spear"));
        });
}

/// Spawn an archer, a unit which fires arrows from it's bow.
pub fn spawn_archer(
    faction: Faction,
    terrain: &Terrain,
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    // The starting position
    let x = match faction {
        Faction::Ally => ALLY_STARTING_POSITION,
        Faction::Enemy => ENEMY_STARTING_POSITION,
    };
    let y = terrain.height_at_x(x);

    // Use a simple square for the collision shape
    let polygon: Polygon = Rect::new(Coordinate::zero(), Coordinate { x: 0.5, y: 1.8 })
        .to_polygon()
        .into();

    commands
        .spawn_bundle(
            ColoredMeshBundle::new(match faction {
                Faction::Ally => asset_server.load("units/allies/character.svg"),
                Faction::Enemy => asset_server.load("units/enemies/character.svg"),
            })
            .with_position(x, y)
            .with_z_index(1.0),
        )
        .insert(polygon)
        .insert(faction)
        .insert(Walk::new(match faction {
            Faction::Ally => 1.0,
            Faction::Enemy => -1.0,
        }))
        .insert(Health::new(100.0))
        .insert(Name::new(match faction {
            Faction::Ally => "Allied Archer",
            Faction::Enemy => "Enemy Archer",
        }))
        .with_children(|parent| {
            parent
                .spawn_bundle(
                    BowBundle::new(2.0, asset_server.load("weapons/bow.svg"))
                        .with_rotation(-90.0)
                        .with_position(0.5, 1.0),
                )
                .insert(Name::new("Bow"));
        });
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
        .for_each(|recruit_event| match recruit_event.0 {
            UnitType::Soldier => {
                spawn_melee_soldier(Faction::Ally, &terrain, &mut commands, &asset_server)
            }
            UnitType::Archer => spawn_archer(Faction::Ally, &terrain, &mut commands, &asset_server),
        });
}
