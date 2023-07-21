
use crate::{constants::ProjectileConstants, inspector::Inspectable, unit::health::Health};
use bevy::prelude::{Commands, Component, Entity, EventReader, EventWriter, Query, With};
use bevy_rapier2d::{
    physics::IntoEntity,
    prelude::{ContactEvent, RigidBodyVelocityComponent},
};

/// Do damage when a certain velocity is exceeded.
#[derive(Debug, Component, Inspectable)]
pub struct DamageOnImpact {
    /// The damage done.
    pub damage: f32,
    /// The minimum velocity needed to trigger the damage done.
    pub min_velocity: f32,
}

impl DamageOnImpact {
    /// Create this component from the projectile constants.
    pub fn from_constants(constants: &ProjectileConstants) -> Self {
        Self {
            damage: constants.damage,
            min_velocity: constants.min_velocity_for_damage,
        }
    }
}

/// When damage is being done to an object.
#[derive(Debug)]
pub struct HitEvent {
    /// The damage that will be done to the object.
    pub damage: f32,
    /// The entity that will be hit with the damage.
    pub target: Entity,
}

/// The system for breaking on hard impacts.
pub fn event_listener(
    mut events: EventReader<ContactEvent>,
    projectile_query: Query<(Entity, &RigidBodyVelocityComponent, &DamageOnImpact)>,
    target_query: Query<Entity, With<Health>>,
    mut commands: Commands,
    mut event_writer: EventWriter<HitEvent>,
) {
    for event in events.iter() {
        if let ContactEvent::Started(collision_object_1, collision_object_2) = event {
            // Get the collision between projectiles and targets with health
            if let Ok((projectile_entity, velocity, damage_on_impact)) = projectile_query
                .get(collision_object_1.entity())
                .or_else(|_| projectile_query.get(collision_object_2.entity()))
            {
                // Only trigger the collision when the velocity is big enough
                if velocity.linvel.magnitude() >= damage_on_impact.min_velocity {
                    if let Ok(target_entity) = target_query
                        .get(collision_object_1.entity())
                        .or_else(|_| target_query.get(collision_object_2.entity()))
                    {
                        // Raise the event
                        event_writer.send(HitEvent {
                            damage: damage_on_impact.damage,
                            target: target_entity,
                        });

                        // Remove the projectile
                        commands.entity(projectile_entity).despawn();
                    }
                }
            }
        }
    }
}
