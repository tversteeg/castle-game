use super::polygon::PolygonComponent;
use bevy::prelude::{Component, Entity, EventReader, EventWriter, Query, With};
use bevy_inspector_egui::Inspectable;
use heron::{CollisionEvent, CollisionShape, Velocity};

/// Allow a polygon to break into multiple pieces when force is applied.
#[derive(Component, Inspectable)]
pub struct Breakable {
    /// The velocity on impact on which it breaks.
    impact_velocity: f32,
}

impl Default for Breakable {
    fn default() -> Self {
        Self {
            impact_velocity: 5.0,
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
    mut events: EventReader<CollisionEvent>,
    query: Query<(Entity, &Velocity, &Breakable)>,
    mut event_writer: EventWriter<BreakEvent>,
) {
    for event in events.iter() {
        if let CollisionEvent::Started(collision_object, _) = event {
            let entity = collision_object.rigid_body_entity();
            if let Ok((entity, velocity, breakable)) = query.get(entity) {
                // "Calculate" the impact velocity
                // TODO: use better velocity calculation
                let impact_velocity = velocity.linear.length();

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
