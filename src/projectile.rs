use specs::*;
use collision::Discrete;

use physics::*;
use draw::*;
use terrain::*;
use ai::Health;
use geom::*;

#[derive(Component, Debug, Copy, Clone)]
pub struct Damage(pub f64);

pub struct ProjectileSystem;
impl<'a> System<'a> for ProjectileSystem {
    type SystemData = (Entities<'a>,
                       Fetch<'a, DeltaTime>,
                       Fetch<'a, Gravity>,
                       Fetch<'a, Terrain>,
                       WriteStorage<'a, Velocity>,
                       WriteStorage<'a, Point>,
                       ReadStorage<'a, MaskId>,
                       Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, dt, grav, terrain, mut vel, mut pos, mask, updater): Self::SystemData) {
        let grav = grav.0;
        let dt = dt.to_seconds();

        for (entity, vel, pos, mask) in (&*entities, &mut vel, &mut pos, &mask).join() {
            let next: Point = Point::new(pos.x + vel.x * dt, pos.y + vel.y * dt);

            match terrain.line_collides(pos.as_i32(), next.as_i32()) {
                Some(point) => {
                    let _ = entities.delete(entity);

                    let crater = entities.create();
                    updater.insert(crater, TerrainMask::new(mask.0, point));

                    /*
                    // TODO replace with proper size
                    let terrain_rect = Aabb2::new(point.0 as f64, point.1 as f64, 10.0, 10.0);

                    let collapse = entities.create();
                    updater.insert(collapse, TerrainCollapse(terrain_rect));
                    */
                },
                None => {
                    *pos = next;
                    vel.y += grav * dt;
                }
            }
        }
    }
}

pub struct ProjectileCollisionSystem;
impl<'a> System<'a> for ProjectileCollisionSystem {
    type SystemData = (Entities<'a>,
                       ReadStorage<'a, Point>,
                       ReadStorage<'a, BoundingBox>,
                       ReadStorage<'a, Damage>,
                       WriteStorage<'a, Health>,
                       Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, pos, bb, dmg, mut health, updater): Self::SystemData) {
        for (proj, proj_pos, proj_bb, proj_dmg) in (&*entities, &pos, &bb, &dmg).join() {
            let proj_aabb = *proj_bb + *proj_pos;
            for (target, target_pos, target_bb, target_health) in (&*entities, &pos, &bb, &mut health).join() {
                let target_aabb = *target_bb + *target_pos;
                if proj_aabb.intersects(&*target_aabb) {
                    target_health.0 -= proj_dmg.0;
                    if target_health.0 <= 0.0 {
                        let _ = entities.delete(target);
                    }

                    let _ = entities.delete(proj);

                    let blood = entities.create();
                    updater.insert(blood, PixelParticle::new(0xFF0000, 10.0));
                    updater.insert(blood, *target_pos);
                    updater.insert(blood, Velocity::new(-10.0, -10.0));
                }
            }
        }
    }
}
