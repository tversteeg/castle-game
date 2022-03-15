use crate::{
    constants::Constants,
    inspector::Inspectable,
    map::terrain::Terrain,
    projectile::event::{ProjectileSpawnEvent},
    unit::{
        closest::{ClosestAlly, ClosestEnemy},
        faction::Faction,
        unit_type::UnitType,
    },
};
use bevy::{
    core::{Time, Timer},
    math::Vec2,
    prelude::{Component, EventWriter, GlobalTransform, Query, Res},
};

/// Fires an event when the enemy is near and the timer runs out.
#[derive(Debug, Component, Inspectable)]
pub struct Discharge {
    /// Interval at which the weapon will be discharged.
    #[inspectable(ignore)]
    timer: Timer,
    /// What type of unit, used to determine the distance.
    unit_type: UnitType,
}

impl Discharge {
    /// Setup the timer.
    pub fn new(unit_type: UnitType, faction: Faction, constants: &Constants) -> Self {
        let unit = constants.unit(unit_type, faction);

        Self {
            unit_type,
            timer: Timer::from_seconds(unit.weapon_delay, true),
        }
    }
}

/// Count down the time and discharge the weapon.
pub fn system(
    mut query: Query<(&mut Discharge, &Faction, &GlobalTransform)>,
    mut event_writer: EventWriter<ProjectileSpawnEvent>,
    time: Res<Time>,
    closest_enemy: Res<ClosestEnemy>,
    closest_ally: Res<ClosestAlly>,
    constants: Res<Constants>,
    terrain: Res<Terrain>,
) {
    for (mut discharge, faction, transform) in query.iter_mut() {
        if discharge.timer.tick(time.delta()).just_finished() {
            // The position of the enemy from this unit
            let enemy_position = match faction {
                Faction::Ally => closest_enemy.x_or_inf(),
                Faction::Enemy => closest_ally.x_or_inf(),
            };

            // Check the distance between this unit and it's next enemy
            let distance_to_next_enemy = (transform.translation.x - enemy_position).abs();

            // If the distance is smaller than the range, fire a projectile
            if distance_to_next_enemy
                <= constants
                    .unit(discharge.unit_type, *faction)
                    .minimum_weapon_distance
            {
                // Where the projectile will spawn
                let start_position = Vec2::new(transform.translation.x, transform.translation.y);

                // Where the projectile will fly to
                let target_position = Some(Vec2::new(
                    enemy_position,
                    terrain.height_at_x(enemy_position),
                ));

                // Spawn the projectile
                event_writer.send(ProjectileSpawnEvent {
                    start_position,
                    target_position,
                    projectile_type: discharge.unit_type.to_projectile_type(),
                })
            }
        }
    }
}
