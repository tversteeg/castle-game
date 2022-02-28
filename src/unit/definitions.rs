use super::{health::Health, walk::Walk};
use crate::{geometry::polygon::Polygon, map::terrain::Terrain, unit::faction::Faction};
use bevy::{
    core::Name,
    prelude::{AssetServer, Assets, Commands, Mesh, Res, ResMut},
    sprite::{ColorMaterial},
};
use bevy_svg::prelude::{Origin, Svg2dBundle};
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
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<ColorMaterial>,
    asset_server: &AssetServer,
) {
    // The starting position
    let x = match faction {
        Faction::Ally => ALLY_STARTING_POSITION,
        Faction::Enemy => ENEMY_STARTING_POSITION,
    };

    // Get the starting height
    let _y = terrain.height_at_x(x);

    // Use a simple square for the drawing and collision shape
    let polygon: Polygon = Rect::new(Coordinate::zero(), Coordinate { x: 0.5, y: 1.8 })
        .to_polygon()
        .into();

    // Load the unit vector graphics
    let svg = asset_server.load("units/allies/character.svg");

    commands
        .spawn_bundle(Svg2dBundle {
            svg,
            origin: Origin::TopLeft,
            ..Default::default()
        })
        .insert(polygon)
        // Load the asset handle for the sprite
        //.insert(asset_server.load::<Image, &'static str>("units/ally/melee.png"))
        .insert(Walk::new(1.0))
        .insert(Health::new(100.0))
        .insert(Name::new(match faction {
            Faction::Ally => "Allied Melee Soldier",
            Faction::Enemy => "Enemy Melee Soldier",
        }));
}

/// Temp setup.
///
/// TOOD: remove
pub fn setup(
    terrain: Res<Terrain>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    spawn_melee_soldier(
        Faction::Ally,
        &terrain,
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
    );
}
