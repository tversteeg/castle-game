use super::{
    closest::{ClosestAlly, ClosestEnemy},
    faction::Faction,
    unit_type::UnitType,
};
use crate::{constants::Constants, inspector::Inspectable, map::terrain::Terrain};
use bevy::{
    core::Time,
    prelude::{Component, Query, Res, Transform},
};

/// Allow a unit to walk across the map.
#[derive(Debug, Component, Inspectable)]
pub struct Walk {
    /// How many meters per second the unit walks.
    speed: f32,
}

impl Walk {
    /// Construct the walk component.
    pub fn for_unit(unit_type: UnitType, faction: Faction, constants: &Constants) -> Self {
        let speed = constants.unit(unit_type, faction).walking_speed;

        Self { speed }
    }
}

/// Let a unit walk over the ground with the specified speed.
///
/// Stop walking when the closest is reached.
pub fn system(
    mut query: Query<(&Faction, &UnitType, &Walk, &mut Transform)>,
    terrain: Res<Terrain>,
    time: Res<Time>,
    closest_ally: Res<ClosestAlly>,
    closest_enemy: Res<ClosestEnemy>,
    constants: Res<Constants>,
) {
    for (faction, unit_type, walk, mut transform) in query.iter_mut() {
        // Get the constant stop distance for the unit
        let stop_distance = constants.unit(*unit_type, *faction).stop_distance;

        // Determine whether we can walk
        let must_stop = match (faction, closest_ally.x, closest_enemy.x) {
            // We are an ally walking to an enemy
            (Faction::Ally, _closest_ally, Some(x)) => transform.translation.x >= x - stop_distance,
            // We are an enemy walking to an ally
            (Faction::Enemy, Some(x), _closest_enemy) => {
                transform.translation.x <= x + stop_distance
            }
            _ => false,
        };

        if !must_stop {
            // Walk horizontally
            transform.translation.x += walk.speed * time.delta_seconds();

            // Follow the curve of the terrain vertically
            transform.translation.y = terrain.height_at_x(transform.translation.x);
        }
    }
}
