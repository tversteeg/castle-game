use super::{health::Health, unit_type::UnitType, walk::Walk};
use crate::{
    draw::svg::AlliedCharacterSvg, geometry::polygon::Polygon, map::terrain::Terrain,
    ui::recruit_button::RecruitEvent, unit::faction::Faction,
};
use bevy::{
    core::Name,
    prelude::{
        Assets, Commands, ComputedVisibility, EventReader, GlobalTransform, Handle, Mesh, Res,
        ResMut, Transform, Visibility,
    },
    sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle},
};
use geo::{Coordinate, Rect};

/// The starting position x coordinate for ally units.
pub const ALLY_STARTING_POSITION: f32 = 5.0;
/// The starting position x coordinate for enemy units.
pub const ENEMY_STARTING_POSITION: f32 = 5.0;

/// Spawn a melee soldier.
pub fn spawn_melee_soldier(
    faction: Faction,
    terrain: &Terrain,
    commands: &mut Commands,
    mesh: Handle<Mesh>,
) {
    // The starting position
    let x = match faction {
        Faction::Ally => ALLY_STARTING_POSITION,
        Faction::Enemy => ENEMY_STARTING_POSITION,
    };
    let y = terrain.height_at_x(x);

    // Use a simple square for the drawing and collision shape
    let polygon: Polygon = Rect::new(Coordinate::zero(), Coordinate { x: 0.5, y: 1.8 })
        .to_polygon()
        .into();

    commands
        //.spawn_bundle(svg.bundle(x, y))
        .spawn()
        .insert(polygon)
        .insert(Walk::new(1.0))
        .insert(Health::new(100.0))
        .insert(Mesh2dHandle(mesh))
        .insert(Transform::from_xyz(x, y, 0.0))
        .insert(GlobalTransform::default())
        .insert(Visibility::default())
        .insert(ComputedVisibility::default())
        .insert(Name::new(match faction {
            Faction::Ally => "Allied Melee Soldier",
            Faction::Enemy => "Enemy Melee Soldier",
        }));
}

/// The system for recruiting new allied units.
pub fn recruit_event_listener(
    mut events: EventReader<RecruitEvent>,
    terrain: Res<Terrain>,
    allied_character: Res<AlliedCharacterSvg>,
    mut commands: Commands,
) {
    events
        .iter()
        // Spawn units based on what unit types have been send by the recruit button
        .for_each(|recruit_event| match recruit_event.0 {
            UnitType::Soldier => spawn_melee_soldier(
                Faction::Ally,
                &terrain,
                &mut commands,
                allied_character.0.clone(),
            ),
            UnitType::Archer => todo!(),
        });
}
