use cgmath::Point2;
use collision::Discrete;
use specs::*;
use specs_derive::Component;

use super::*;

#[derive(Component, Debug, Eq, PartialEq)]
pub enum UnitState {
    // The path is clear and the unit can walk
    Walk,
    // There is a high ledge in front of the unit and it needs to climb it
    Climb,
    // There is another unit in front of this unit
    Wait,

    // The unit is fighting with an enemy unit
    Melee,
    // The unit is shooting at an enemy unit
    Shoot,
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Health(pub f64);

#[derive(Component, Debug, Copy, Clone)]
pub struct HealthBar {
    pub health: f64,
    pub max_health: f64,
    pub width: usize,
    pub pos: Point2<usize>,
    pub offset: (i32, i32),
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Walk {
    pub bounds: BoundingBox,
    pub speed: f64,
}

impl Walk {
    pub fn new(bounds: BoundingBox, speed: f64) -> Self {
        Walk { bounds, speed }
    }
}

pub struct WalkSystem;
impl<'a> System<'a> for WalkSystem {
    type SystemData = (
        Read<'a, DeltaTime>,
        Read<'a, Terrain>,
        ReadStorage<'a, Destination>,
        ReadStorage<'a, Walk>,
        WriteStorage<'a, UnitState>,
        WriteStorage<'a, WorldPosition>,
    );

    fn run(&mut self, (dt, terrain, dest, walk, mut state, mut pos): Self::SystemData) {
        let dt = dt.to_seconds();

        for (dest, walk, state, pos) in (&dest, &walk, &mut state, &mut pos).join() {
            // Don't walk when the unitstate is not saying that it can walk
            if *state != UnitState::Walk {
                continue;
            }

            let hit_box = walk.bounds + *pos.0;
            if let Some(hit) = terrain.rect_collides(hit_box) {
                if hit.1 == hit_box.min.y as i32 {
                    // Top edge of bounding box is hit, try to climb
                    *state = UnitState::Climb;
                    continue;
                }
            }

            pos.0.x += walk.speed * dt * (dest.0 - pos.0.x).signum();
        }
    }
}

pub struct HealthBarSystem;
impl<'a> System<'a> for HealthBarSystem {
    type SystemData = (
        ReadStorage<'a, Health>,
        ReadStorage<'a, WorldPosition>,
        WriteStorage<'a, HealthBar>,
    );

    fn run(&mut self, (health, pos, mut bar): Self::SystemData) {
        for (health, pos, bar) in (&health, &pos, &mut bar).join() {
            bar.health = health.0;
            bar.pos = pos.0.as_usize();
            bar.pos.x = (bar.pos.x as i32 + bar.offset.0) as usize;
            bar.pos.y = (bar.pos.y as i32 + bar.offset.1) as usize;
        }
    }
}

pub struct UnitFallSystem;
impl<'a> System<'a> for UnitFallSystem {
    type SystemData = (
        Read<'a, DeltaTime>,
        Read<'a, Terrain>,
        ReadStorage<'a, Walk>,
        WriteStorage<'a, WorldPosition>,
    );

    fn run(&mut self, (dt, terrain, walk, mut pos): Self::SystemData) {
        let dt = dt.to_seconds();

        for (walk, pos) in (&walk, &mut pos).join() {
            pos.0.y += GRAVITY * dt;

            // Move the units if they collide with the ground in a loop until they don't touch the ground anymore
            loop {
                let hit_box = walk.bounds + *pos.0;
                match terrain.rect_collides(hit_box) {
                    Some(_) => {
                        pos.0.y -= 1.0;
                    }
                    None => break,
                }
            }
        }
    }
}

pub struct UnitResumeWalkingSystem;
impl<'a> System<'a> for UnitResumeWalkingSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, WorldPosition>,
        ReadStorage<'a, BoundingBox>,
        ReadStorage<'a, Destination>,
        WriteStorage<'a, UnitState>,
    );

    fn run(&mut self, (entities, pos, bb, dest, mut state): Self::SystemData) {
        for (e1, pos1, bb1, dest1) in (&*entities, &pos, &bb, &dest).join() {
            // A unit can only resume walking when it's waiting or fighting
            if let Some(state1) = state.get_mut(e1) {
                if *state1 != UnitState::Wait && *state1 != UnitState::Melee {
                    continue;
                }
            }

            // Get the bounding box of entity 1
            let aabb1 = *bb1 + *pos1.0;

            // If it's waiting or fighting and not colliding anymore let it walk
            let mut intersects = false;
            for (e2, pos2, bb2, dest2) in (&*entities, &pos, &bb, &dest).join() {
                // Don't collide with itself
                if e1 == e2 {
                    continue;
                }

                // Get the bounding box of entity 2
                let aabb2 = *bb2 + *pos2.0;

                // If they bounding boxes of both units intersect
                if aabb1.intersects(&*aabb2) {
                    let dist1 = (dest1.0 - pos1.0.x).abs();
                    let dist2 = (dest2.0 - pos2.0.x).abs();

                    // If this is not the unit closest to it's destination
                    if dist1 > dist2 {
                        intersects = true;
                        break;
                    }
                }
            }

            // Make it walk again if there is no collision
            if !intersects {
                if let Some(state1) = state.get_mut(e1) {
                    *state1 = UnitState::Walk;
                }
            }
        }
    }
}

pub struct UnitCollideSystem;
impl<'a> System<'a> for UnitCollideSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Ally>,
        ReadStorage<'a, WorldPosition>,
        ReadStorage<'a, BoundingBox>,
        ReadStorage<'a, Destination>,
        WriteStorage<'a, UnitState>,
    );

    fn run(&mut self, (entities, ally, pos, bb, dest, mut state): Self::SystemData) {
        for (e1, pos1, bb1, dest1) in (&*entities, &pos, &bb, &dest).join() {
            // Get the bounding box of entity 1
            let aabb1 = *bb1 + *pos1.0;

            // Don't check if this unit is not walking
            if let Some(state1) = state.get_mut(e1) {
                // If it's not walking ignore it
                if *state1 != UnitState::Walk {
                    continue;
                }
            }

            // Check for a collision if this unit is walking
            for (e2, pos2, bb2, dest2) in (&*entities, &pos, &bb, &dest).join() {
                // Don't collide with itself
                if e1 == e2 {
                    continue;
                }

                // Join a melee
                let is_melee = if let Some(state) = state.get_mut(e2) {
                    *state == UnitState::Melee
                } else {
                    // Unit doesn't have a unit state?
                    panic!("Unit doesn't have a unit state");
                };

                let aabb2 = if is_melee {
                    // Get the half bounding box of entity 2
                    bb2.to_half_width() + *pos2.0
                } else {
                    // Get the full bounding box of entity 2
                    *bb2 + *pos2.0
                };

                // Ignore the units if they don't collide
                if !aabb1.intersects(&*aabb2) {
                    continue;
                }

                let is_ally1 = ally.get(e1).is_some();
                let is_ally2 = ally.get(e2).is_some();

                if is_ally1 == is_ally2 {
                    // If they are both allies or both enemies let one of them wait
                    let dist1 = (dest1.0 - pos1.0.x).abs();
                    let dist2 = (dest2.0 - pos2.0.x).abs();
                    // Let the unit wait which is furthest away from the destination
                    if dist1 > dist2 {
                        if let Some(state) = state.get_mut(e1) {
                            *state = UnitState::Wait;
                        }
                        break;
                    } else if let Some(state) = state.get_mut(e2) {
                        *state = UnitState::Wait;
                    }
                } else {
                    // If they are an ally and an enemy let them fight
                    if let Some(state) = state.get_mut(e1) {
                        *state = UnitState::Melee;
                    }
                    if let Some(state) = state.get_mut(e2) {
                        *state = UnitState::Melee;
                    }
                    break;
                }
            }
        }
    }
}

pub fn reduce_unit_health<'a>(
    entities: &'a Entities,
    unit: &'a Entity,
    health: &'a mut Health,
    dmg: f64,
) -> bool {
    health.0 -= dmg;
    if health.0 <= 0.0 {
        let _ = entities.delete(*unit);

        return true;
    }

    return false;
}
