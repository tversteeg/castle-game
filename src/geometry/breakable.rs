use bevy::prelude::{Component, Entity, EventReader, EventWriter, Query};
use crate::inspector::Inspectable;
use bevy_rapier2d::{
    physics::IntoEntity,
    prelude::{ContactEvent, RigidBodyVelocityComponent},
};

/// Allow a polygon to break into multiple pieces when force is applied.
#[derive(Component, Inspectable)]
pub struct Breakable {
    /// The velocity on impact on which it breaks.
    impact_velocity: f32,
}

impl Default for Breakable {
    fn default() -> Self {
        Self {
            impact_velocity: 3.0,
        }
    }
}

/// The event that's fired when an entity needs to break.
pub struct BreakEvent {
    /// The speed with which the collision occurs.
    pub impact_velocity: f32,
    /// The entity which collides.
    pub entity: Entity,
}

/// Check collision events for when enough force is applied.
pub fn system(
    mut events: EventReader<ContactEvent>,
    query: Query<(Entity, &RigidBodyVelocityComponent, &Breakable)>,
    mut event_writer: EventWriter<BreakEvent>,
) {
    for event in events.iter() {
        if let ContactEvent::Started(collision_object_1, collision_object_2) = event {
            // Try to get the breakable entity from both sides of the collision
            if let Ok((entity, velocity, breakable)) = query
                .get(collision_object_1.entity())
                .or_else(|_| query.get(collision_object_2.entity()))
            {
                // "Calculate" the impact velocity
                // TODO: use better velocity calculation
                let impact_velocity = -velocity.0.linvel.y;

                // Check the velocity to see if we need to split it
                if impact_velocity >= breakable.impact_velocity {
                    // We need to split the object, trigger an event
                    event_writer.send(BreakEvent {
                        impact_velocity,
                        entity,
                    });
                }
            }
        }
    }
}
