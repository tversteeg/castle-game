use crate::inspector::Inspectable;
use bevy::prelude::{Commands, Component, Entity, EventReader, Query, With};
use bevy_rapier2d::{
    physics::IntoEntity,
    prelude::{
        ContactEvent, RigidBodyPositionComponent,
        RigidBodyVelocityComponent, Rotation,
    },
};


/// Tell the physics engine to lock the rotation to the velocity until the first contact event.
///
/// The value is the rotation offset.
#[derive(Debug, Default, Component, Inspectable)]
pub struct RotateToVelocityUntilContact(f32);

impl RotateToVelocityUntilContact {
    /// Instantiate the component with a rotation offset.
    pub fn with_rotation_offset(offset_rad: f32) -> Self {
        Self(offset_rad)
    }
}

/// Remove the rotate lock component on collision.
pub fn contact_event_listener(
    mut events: EventReader<ContactEvent>,
    query: Query<Entity, With<RotateToVelocityUntilContact>>,
    mut commands: Commands,
) {
    for event in events.iter() {
        if let ContactEvent::Started(collision_object_1, collision_object_2) = event {
            // Remove the component from both entities
            if let Ok(entity) = query.get(collision_object_1.entity()) {
                commands
                    .entity(entity)
                    .remove::<RotateToVelocityUntilContact>();
            }
            if let Ok(entity) = query.get(collision_object_2.entity()) {
                commands
                    .entity(entity)
                    .remove::<RotateToVelocityUntilContact>();
            }
        }
    }
}

/// Let the physics object rotate towards it's velocity.
pub fn system(
    mut query: Query<(
        &mut RigidBodyPositionComponent,
        &RigidBodyVelocityComponent,
        &RotateToVelocityUntilContact,
    )>,
) {
    for (mut position, velocity, rotation_offset) in query.iter_mut() {
        // Calculate the rotation based on the velocity
        let rotation = velocity.linvel.y.atan2(velocity.linvel.x) + rotation_offset.0;

        position.position.rotation = Rotation::new(rotation).into();
    }
}
