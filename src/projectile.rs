use specs::*;
use rand;
use rand::distributions::{IndependentSample, Range};
use collision::Discrete;

use physics::*;
use draw::*;
use terrain::*;
use ai::*;
use geom::*;

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

            line.p2.x = pos.0.x as usize + (rot.cos() * arrow.0) as usize;
            line.p2.y = pos.0.y as usize + (rot.sin() * arrow.0) as usize;
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
                       WriteStorage<'a, Velocity>,
                       WriteStorage<'a, WorldPosition>,
                       Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, dt, grav, terrain, proj, mask, mut vel, mut pos, updater): Self::SystemData) {
        let grav = grav.0;
        let dt = dt.to_seconds();

        for (entity, _, vel, pos) in (&*entities, &proj, &mut vel, &mut pos).join() {
            let next: Point = Point::new(pos.0.x + vel.x * dt, pos.0.y + vel.y * dt);

            match terrain.line_collides(pos.0.as_i32(), next.as_i32()) {
                Some(point) => {
                    let _ = entities.delete(entity);

                    let e_mask: Option<&MaskId> = mask.get(entity);
                    if let Some(mask) = e_mask {
                        let crater = entities.create();
                        updater.insert(crater, TerrainMask::new(mask.0, point));
                    }

                    /*
                    // TODO replace with proper size
                    let terrain_rect = Aabb2::new(point.0 as f64, point.1 as f64, 10.0, 10.0);

                    let collapse = entities.create();
                    updater.insert(collapse, TerrainCollapse(terrain_rect));
                    */
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

                let target_aabb = *target_bb + *target_pos.0;
                if proj_aabb.intersects(&*target_aabb) {
                    target_health.0 -= proj_dmg.0;
                    if target_health.0 <= 0.0 {
                        let _ = entities.delete(target);
                    }

                    let _ = entities.delete(proj);
                    let between = Range::new(-20.0, 20.0);
                    let mut rng = rand::thread_rng();

                    let blood = entities.create();
                    updater.insert(blood, PixelParticle::new(0xDD0000, 10.0));
                    updater.insert(blood, *target_pos);
                    updater.insert(blood, Velocity::new(between.ind_sample(&mut rng), between.ind_sample(&mut rng)));
                }
            }
        }
    }
}
