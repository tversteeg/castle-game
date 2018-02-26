use specs::*;
use rand;
use rand::distributions::{IndependentSample, Range};
use collision::Discrete;

use super::*;

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub enum IgnoreCollision {
    Enemy,
    Ally
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Projectile;

#[derive(Component, Debug, Copy, Clone)]
pub struct ProjectileSprite(pub Sprite);

#[derive(Component, Debug, Copy, Clone)]
pub struct ProjectileBoundingBox(pub BoundingBox);

#[derive(Component, Debug, Copy, Clone)]
pub struct Arrow(pub f64);

#[derive(Component, Debug, Copy, Clone)]
pub struct Damage(pub f64);

pub struct ArrowSystem;
impl<'a> System<'a> for ArrowSystem {
    type SystemData = (ReadStorage<'a, WorldPosition>,
                       ReadStorage<'a, Velocity>,
                       WriteStorage<'a, Arrow>,
                       WriteStorage<'a, Line>);

    fn run(&mut self, (pos, vel, mut arrow, mut line): Self::SystemData) {
        for (pos, vel, arrow, line) in (&pos, &vel, &mut arrow, &mut line).join() {
            let rot = (vel.y as f64).atan2(vel.x as f64);

            line.p1.x = pos.0.x as usize;
            line.p1.y = pos.0.y as usize;

            line.p2.x = pos.0.x as usize - (rot.cos() * arrow.0) as usize;
            line.p2.y = pos.0.y as usize - (rot.sin() * arrow.0) as usize;
        }
    }
}

pub struct ProjectileSystem;
impl<'a> System<'a> for ProjectileSystem {
    type SystemData = (Entities<'a>,
                       Fetch<'a, DeltaTime>,
                       Fetch<'a, Gravity>,
                       Fetch<'a, Terrain>,
                       ReadStorage<'a, Projectile>,
                       ReadStorage<'a, MaskId>,
                       WriteStorage<'a, Line>,
                       WriteStorage<'a, Velocity>,
                       WriteStorage<'a, WorldPosition>,
                       Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, dt, grav, terrain, proj, mask, line, mut vel, mut pos, updater): Self::SystemData) {
        let grav = grav.0;
        let dt = dt.to_seconds();

        for (entity, _, vel, pos) in (&*entities, &proj, &mut vel, &mut pos).join() {
            let next: Point = Point::new(pos.0.x + vel.x * dt, pos.0.y + vel.y * dt);

            match terrain.line_collides(pos.0.as_i32(), next.as_i32()) {
                Some(point) => {
                    if let Some(mask) = mask.get(entity) {
                        // Create a crater if there is a mask for it
                        updater.insert(entities.create(), TerrainMask::new(mask.id, point, mask.size));
                    }

                    if let Some(line) = line.get(entity) {
                        // Keep drawing the line if there is one, this makes the arrows stay stuck
                        // in the ground
                        let mut line_copy = *line;
                        line_copy.p1.x = point.0 as usize;
                        line_copy.p1.y = point.1 as usize;
                        line_copy.p2.x += line_copy.p1.x - line.p1.x;
                        line_copy.p2.y += line_copy.p1.y - line.p1.y;
                        updater.insert(entities.create(), line_copy);
                    }

                    let _ = entities.delete(entity);
                },
                None => {
                    pos.0 = next;
                    vel.y += grav * dt;
                }
            }
        }
    }
}

pub struct ProjectileCollisionSystem;
impl<'a> System<'a> for ProjectileCollisionSystem {
    type SystemData = (Entities<'a>,
                       ReadStorage<'a, Projectile>,
                       ReadStorage<'a, WorldPosition>,
                       ReadStorage<'a, ProjectileBoundingBox>,
                       ReadStorage<'a, BoundingBox>,
                       ReadStorage<'a, Damage>,
                       ReadStorage<'a, IgnoreCollision>,
                       ReadStorage<'a, Ally>,
                       ReadStorage<'a, Enemy>,
                       WriteStorage<'a, Health>,
                       Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, proj, pos, proj_bb, bb, dmg, ignore, ally, enemy, mut health, updater): Self::SystemData) {
        for (proj, _, proj_pos, proj_bb, proj_dmg) in (&*entities, &proj, &pos, &proj_bb, &dmg).join() {
            let proj_aabb = proj_bb.0 + *proj_pos.0;
            for (target, target_pos, target_bb, target_health) in (&*entities, &pos, &bb, &mut health).join() {
                let ignore_e: Option<&IgnoreCollision> = ignore.get(proj);
                if let Some(ignore) = ignore_e {
                    if *ignore == IgnoreCollision::Ally {
                        let is_ally: Option<&Ally> = ally.get(target);
                        if let Some(_) = is_ally {
                            continue;
                        }
                    }
                    if *ignore == IgnoreCollision::Enemy {
                        let is_enemy: Option<&Enemy> = enemy.get(target);
                        if let Some(_) = is_enemy {
                            continue;
                        }
                    }
                }

                // When there is a collision with a unit
                let target_aabb = *target_bb + *target_pos.0;
                if proj_aabb.intersects(&*target_aabb) {
                    if reduce_unit_health(&entities, &target, target_health, proj_dmg.0) {
                        // The ally died
                        updater.insert(entities.create(), FloatingText {
                            text: "x".to_string(),
                            pos: target_pos.0,
                            time_alive: 2.0
                        });
                    }

                    let _ = entities.delete(proj);
                    let between = Range::new(-20.0, 20.0);
                    let mut rng = rand::thread_rng();

                    for _ in 0..4 {
                        let blood = entities.create();
                        updater.insert(blood, PixelParticle::new(0xDD0000, 10.0));
                        updater.insert(blood, *target_pos);
                        updater.insert(blood, Velocity::new(between.ind_sample(&mut rng), between.ind_sample(&mut rng)));
                    }
                }
            }
        }
    }
}

pub struct ProjectileRemovalFromMaskSystem;
impl<'a> System<'a> for ProjectileRemovalFromMaskSystem {
    type SystemData = (Entities<'a>,
                       ReadStorage<'a, TerrainMask>,
                       ReadStorage<'a, Line>);

    fn run(&mut self, (entities, mask, line): Self::SystemData) {
        for mask in mask.join() {
            let sx = (mask.size.0 / 2) as usize;
            let sy = (mask.size.1 / 2) as usize;

            for (entity, line) in (&*entities, &line).join() {
                // Check if the line's start point is inside the mask and remove it if that's the case
                let dx = (mask.pos.0 - line.p1.x as i32).abs() as usize;
                let dy = (mask.pos.1 - line.p1.y as i32).abs() as usize;
                if dx <= sx && dy <= sy {
                    let _ = entities.delete(entity);
                }
            }
        }
    }
}
