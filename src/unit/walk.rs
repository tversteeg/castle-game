use crate::map::terrain::Terrain;
use bevy::{
    core::Time,
    prelude::{Component, Query, Res, Transform},
};
use crate::inspector::Inspectable;

use super::{
    closest::{ClosestAlly, ClosestEnemy},
    faction::Faction,
    unit_type::UnitType,
};

/// The distance at which the unit must stop before the next one.
pub const STOP_DISTANCE: f32 = 2.0;

/// Allow a unit to walk across the map.
#[derive(Debug, Component, Inspectable)]
pub struct Walk {
    /// How many meters per second the unit walks.
    speed: f32,
}

impl Walk {
    /// Construct the walk component.
    pub fn for_unit(unit_type: UnitType, faction: Faction) -> Self {
        let speed = match (unit_type, faction) {
            (UnitType::Soldier, Faction::Ally) => 1.5,
            (UnitType::Soldier, Faction::Enemy) => -1.7,
            (UnitType::Archer, Faction::Ally) => 1.2,
            (UnitType::Archer, Faction::Enemy) => -1.3,
        };

        Self { speed }
    }
}

/// Let a unit walk over the ground with the specified speed.
///
/// Stop walking when the closest is reached.
pub fn system(
    mut query: Query<(&Faction, &Walk, &mut Transform)>,
    terrain: Res<Terrain>,
    time: Res<Time>,
    closest_ally: Res<ClosestAlly>,
    closest_enemy: Res<ClosestEnemy>,
) {
    for (faction, walk, mut transform) in query.iter_mut() {
        // Determine whether we can walk
        let must_stop = match (faction, closest_ally.x, closest_enemy.x) {
            // We are an ally walking to an enemy
            (Faction::Ally, _closest_ally, Some(x)) => transform.translation.x >= x - STOP_DISTANCE,
            // We are an enemy walking to an ally
            (Faction::Enemy, Some(x), _closest_enemy) => {
                transform.translation.x <= x + STOP_DISTANCE
            }
            _ => false,
        };

        if !must_stop {
            // Walk horizontally
            transform.translation.x += walk.speed * time.delta_seconds();
        }

        // Follow the curve of the terrain vertically
        transform.translation.y = terrain.height_at_x(transform.translation.x);
    }
}
