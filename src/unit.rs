use specs::*;
use collision::Discrete;

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
    Shoot
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Health(pub f64);

#[derive(Component, Debug, Copy, Clone)]
pub struct Walk {
    pub bounds: BoundingBox,
    pub speed: f64,
}

impl Walk {
    pub fn new(bounds: BoundingBox, speed: f64) -> Self {
        Walk { bounds, speed, }
    }
}

pub struct WalkSystem;
impl<'a> System<'a> for WalkSystem {
    type SystemData = (Fetch<'a, DeltaTime>,
                       Fetch<'a, Terrain>,
                       ReadStorage<'a, Destination>,
                       ReadStorage<'a, Walk>,
                       WriteStorage<'a, UnitState>,
                       WriteStorage<'a, WorldPosition>);

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

pub struct UnitFallSystem;
impl<'a> System<'a> for UnitFallSystem {
    type SystemData = (Fetch<'a, Terrain>,
                       ReadStorage<'a, Walk>,
                       WriteStorage<'a, WorldPosition>);

    fn run(&mut self, (terrain, walk, mut pos): Self::SystemData) {
        for (walk, pos) in (&walk, &mut pos).join() {
            pos.0.y += 1.0;

            // Move the units if they collide with the ground in a loop until they don't touch the ground anymore
            loop {
                let hit_box = walk.bounds + *pos.0;
                match terrain.rect_collides(hit_box) {
                    Some(_) => {
                        pos.0.y -= 1.0;
                    },
                    None => break
                }
            }
        }
    }
}

pub struct UnitCollideSystem;
impl<'a> System<'a> for UnitCollideSystem {
    type SystemData = (Entities<'a>,
                       ReadStorage<'a, Ally>,
                       ReadStorage<'a, WorldPosition>,
                       ReadStorage<'a, BoundingBox>,
                       WriteStorage<'a, UnitState>,
                       Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, ally, pos, bb, mut state, updater): Self::SystemData) {
        for (e1, pos1, bb1) in (&*entities, &pos, &bb).join() {
            // Get the bounding box of entity 1
            let aabb1 = *bb1 + *pos1.0;

            // Don't check if this unit is not walking
            if let Some(state) = state.get_mut(e1){
                if *state == UnitState::Wait {
                    // If it's waiting and not colliding anymore let it walk
                    let mut intersects = false;
                    for (e2, pos2, bb2) in (&*entities, &pos, &bb).join() {
                        // Don't collide with itself
                        if e1 == e2 {
                            continue;
                        }

                        // Get the bounding box of entity 2
                        let aabb2 = *bb2 + *pos2.0;

                        if aabb1.intersects(&*aabb2) {
                            intersects = true;
                            break;
                        }
                    }

                    // Make it walk again if there is no collision
                    if !intersects {
                        *state = UnitState::Walk;
                    }
                }

                // If it's not walking ignore it
                if *state != UnitState::Walk {
                    continue;
                }
            }

            for (e2, pos2, bb2) in (&*entities, &pos, &bb).join() {
                // Don't collide with itself
                if e1 == e2 {
                    continue;
                }

                // Get the bounding box of entity 2
                let aabb2 = *bb2 + *pos2.0;

                // Ignore the units if they don't collide
                if !aabb1.intersects(&*aabb2) {
                    continue;
                }

                let is_ally1 = ally.get(e1).is_some();
                let is_ally2 = ally.get(e2).is_some();

                // If they are both allies or both enemies
                if is_ally1 == is_ally2  {
                    // Let the first one wait
                    if let Some(state) = state.get_mut(e1) {
                        *state = UnitState::Wait;
                    }
                } else {
                    if let Some(state) = state.get_mut(e1) {
                        *state = UnitState::Melee;
                    }
                    if let Some(state) = state.get_mut(e2) {
                        *state = UnitState::Melee;
                    }
                }
                break;
            }
        }
    }
}

pub fn reduce_unit_health<'a>(entities: &'a EntitiesRes, unit: &'a Entity, health: &'a mut Health, dmg: f64) -> bool {
    health.0 -= dmg;
    if health.0 <= 0.0 {
        let _ = entities.delete(*unit);

        return true;
    }

    return false;
}
