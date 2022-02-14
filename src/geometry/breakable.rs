use super::polygon::PolygonComponent;
use bevy::prelude::{Component, EventReader, Query, With};
use bevy_inspector_egui::Inspectable;
use heron::{CollisionEvent, CollisionShape};

/// Allow a polygon to break into multiple pieces when force is applied.
#[derive(Component, Inspectable)]
pub struct Breakable;

/// Check collision events for when enough force is applied.
pub fn system(
    mut events: EventReader<CollisionEvent>,
    mut query: Query<(&PolygonComponent), With<Breakable>>,
) {
    for event in events.iter() {
        dbg!(event);
        if let CollisionEvent::Started(collision_object, _) = event {
            let entity = collision_object.collision_shape_entity();
        }
    }
}
