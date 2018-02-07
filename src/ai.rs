use specs::*;
use rand;
use rand::distributions::{IndependentSample, Range};

use physics::*;
use terrain::*;
use draw::*;
use projectile::*;

#[derive(Component, Debug, Copy, Clone)]
pub struct Health(pub f64);

#[derive(Component, Debug, Copy, Clone)]
pub struct Walk {
    pub bounds: Rect,
    pub speed: f64
}

impl Walk {
    pub fn new(bounds: Rect, speed: f64) -> Self {
        Walk { bounds, speed }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Destination(pub f64);

#[derive(Component, Debug)]
pub struct Ally;

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct Turret {
    pub delay: f64,
    pub max_strength: f64,
    pub flight_time: f64,
    pub strength_variation: f64,

    delay_left: f64
}

impl Turret {
    pub fn new(delay: f64, max_strength: f64, strength_variation: f64, flight_time: f64) -> Self {
        Turret {
            delay, max_strength, flight_time, strength_variation,
            delay_left: 0.0
        }
    }
}

impl Default for Turret {
    fn default() -> Self {
        Turret {
            delay: 5.0,
            max_strength: 210.0,
            flight_time: 3.0,
            strength_variation: 10.0,

            delay_left: 0.0
        }
    }
}

#[derive(Component, Debug)]
pub struct Melee(pub Damage);

pub struct WalkSystem;
impl<'a> System<'a> for WalkSystem {
    type SystemData = (Fetch<'a, DeltaTime>,
                       Fetch<'a, Terrain>,
                       ReadStorage<'a, Walk>,
                       ReadStorage<'a, Destination>,
                       WriteStorage<'a, Position>);

    fn run(&mut self, (dt, terrain, walk, dest, mut pos): Self::SystemData) {
        let dt = dt.to_seconds();

        for (walk, dest, pos) in (&walk, &dest, &mut pos).join() {
            pos.y += 1.0;

            loop {
                let hit_box = walk.bounds + *pos;
                match terrain.rect_collides(hit_box) {
                    Some(hit) => {
                        pos.y -= 1.0;

                        if hit.1 == hit_box.y as i32 {
                            // Top edge of bounding box is hit, don't walk anymore
                            break;
                        }

                        pos.x += walk.speed * dt * (dest.0 - pos.x).signum();
                    },
                    None => break
                }
            }
        }
    }
}

pub struct MeleeSystem;
impl<'a> System<'a> for MeleeSystem {
    type SystemData = (ReadStorage<'a, Ally>,
                       ReadStorage<'a, Enemy>,
                       ReadStorage<'a, Position>,
                       ReadStorage<'a, BoundingBox>,
                       WriteStorage<'a, Melee>,
                       WriteStorage<'a, Health>);

    fn run(&mut self, (ally, enemy, pos, bb, melee, mut health): Self::SystemData) {

    }
}

pub struct TurretSystem;
impl<'a> System<'a> for TurretSystem {
    type SystemData = (Entities<'a>,
                       Fetch<'a, DeltaTime>,
                       Fetch<'a, Gravity>,
                       ReadStorage<'a, Ally>,
                       ReadStorage<'a, Enemy>,
                       ReadStorage<'a, Position>,
                       ReadStorage<'a, Sprite>,
                       ReadStorage<'a, MaskId>,
                       ReadStorage<'a, BoundingBox>,
                       ReadStorage<'a, Damage>,
                       ReadStorage<'a, Walk>,
                       WriteStorage<'a, Turret>,
                       Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, dt, grav, ally, enemy, pos, sprite, mask, bb, dmg, walk, mut turret, updater): Self::SystemData) {
        let dt = dt.to_seconds();
        let grav = grav.0;

        for (tpos, _, sprite, mask, bb, dmg, turret) in (&pos, &enemy, &sprite, &mask, &bb, &dmg, &mut turret).join() {
            turret.delay_left -= dt;
            if turret.delay_left > 0.0 {
                continue;
            }

            // Find the nearest ally to shoot
            let mut closest = Position::new(-1000.0, -1000.0);
            let mut dist = tpos.distance_to(&closest);

            for (apos, _, walk) in (&pos, &ally, &walk).join() {
                let mut pos = *apos;
                pos.x += walk.speed * turret.flight_time;

                let dist_to = tpos.distance_to(&pos);
                if dist_to < dist {
                    dist = dist_to;
                    closest = pos;
                }
            }

            let between = Range::new(-turret.strength_variation, turret.strength_variation);
            let mut rng = rand::thread_rng();

            let time = turret.flight_time;
            let vx = (closest.x - tpos.x) / time + between.ind_sample(&mut rng);
            let vy = (closest.y + 0.5 * -grav * time * time - tpos.y) / time + between.ind_sample(&mut rng);

            if (vx * vx + vy * vy).sqrt() < turret.max_strength {
                // Shoot the turret

                let projectile = entities.create();
                updater.insert(projectile, Position::new(tpos.x, tpos.y));
                updater.insert(projectile, Velocity::new(vx, vy));
                updater.insert(projectile, *sprite);
                updater.insert(projectile, *mask);
                updater.insert(projectile, *bb);
                updater.insert(projectile, *dmg);

                turret.delay_left = turret.delay;
            }
        }
    }
}
