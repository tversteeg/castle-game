use bevy::{
    core::{Name, Time, Timer},
    prelude::{AssetServer, Commands, Component, Query, Res},
};
use crate::inspector::Inspectable;

use crate::map::terrain::Terrain;

use super::{bundle::UnitBundle, faction::Faction, unit_type::UnitType};

/// Spawn an enemy unit once in a interval.
#[derive(Component, Inspectable)]
pub struct EnemySpawner {
    /// Interval at which the enemy will be spawned.
    #[inspectable(ignore)]
    timer: Timer,
    /// The unit type to spawn.
    unit_type: UnitType,
}

impl EnemySpawner {
    /// Spawn an enemy every amount of seconds.
    pub fn from_seconds(seconds: f32, unit_type: UnitType) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, true),
            unit_type,
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
            let unit = UnitBundle::new(spawner.unit_type, Faction::Enemy, &terrain, &asset_server);

            unit.spawn(&mut commands, &asset_server);
        }
    }
}

/// Setup the spawners.
pub fn setup(mut commands: Commands) {
    commands
        .spawn()
        .insert(EnemySpawner::from_seconds(5.0, UnitType::Soldier))
        .insert(Name::new("Enemy Melee Spawner"));

    commands
        .spawn()
        .insert(EnemySpawner::from_seconds(9.0, UnitType::Archer))
        .insert(Name::new("Enemy Archer Spawner"));
}
