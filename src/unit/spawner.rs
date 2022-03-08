use bevy::{
    core::{Time, Timer},
    prelude::{AssetServer, Commands, Component, Handle, Mesh, Query, Res, ResMut},
};
use bevy_inspector_egui::Inspectable;

use crate::map::terrain::Terrain;

use super::faction::Faction;

/// Spawn an enemy unit once in a interval.
#[derive(Component, Inspectable)]
pub struct EnemySpawner {
    /// Interval at which the enemy will be spawned.
    #[inspectable(ignore)]
    timer: Timer,
    /// The function to spawn the unit.
    #[inspectable(ignore)]
    spawn_fn: fn(faction: Faction, terrain: &Terrain, commands: &mut Commands, mesh: Handle<Mesh>),
}

impl EnemySpawner {
    /// Spawn an enemy every amount of seconds.
    pub fn from_seconds(
        seconds: f32,
        spawn_fn: fn(
            faction: Faction,
            terrain: &Terrain,
            commands: &mut Commands,
            mesh: Handle<Mesh>,
        ),
    ) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, true),
            spawn_fn,
        }
    }
}

/// Count down the time and spawn an enemy.
pub fn system(
    time: Res<Time>,
    mut query: Query<&mut EnemySpawner>,
    terrain: Res<Terrain>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for mut spawner in query.iter_mut() {
        if spawner.timer.tick(time.delta()).just_finished() {
            // Spawn the unit
            (spawner.spawn_fn)(
                Faction::Enemy,
                &terrain,
                &mut commands,
                asset_server.load("units/enemies/character.svg"),
            );
        }
    }
}

/// Setup the spawners.
pub fn setup(mut commands: Commands) {
    commands.spawn().insert(EnemySpawner::from_seconds(
        5.0,
        super::definitions::spawn_melee_soldier,
    ));

    commands.spawn().insert(EnemySpawner::from_seconds(
        9.0,
        super::definitions::spawn_archer,
    ));
}
