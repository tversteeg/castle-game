use specs::*;
use std::time::Duration;

use draw::*;
use terrain::*;

#[derive(Component, Debug, Copy, Clone)]
pub struct Position {
    x: f64,
    y: f64
}

impl Position {
    pub fn new(x: f64, y: f64) -> Self {
        Position { x, y }
    }

    pub fn as_i32(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Velocity {
    x: f64,
    y: f64
}

impl Velocity {
    pub fn new(x: f64, y: f64) -> Self {
        Velocity { x, y }
    }
}

pub struct DeltaTime(pub Duration);

impl DeltaTime {
    pub fn new(time: f64) -> Self {
        DeltaTime(Duration::from_millis((time * 1000.0) as u64))
    }

    pub fn to_seconds(&self) -> f64 {
        self.0.as_secs() as f64 + self.0.subsec_nanos() as f64 * 1e-9
    }
}

pub struct Gravity(pub f64);

pub struct ProjectileSystem;

impl<'a> System<'a> for ProjectileSystem {
    type SystemData = (Entities<'a>,
                       Fetch<'a, DeltaTime>,
                       Fetch<'a, Gravity>,
                       Fetch<'a, Terrain>,
                       WriteStorage<'a, Velocity>,
                       WriteStorage<'a, Position>,
                       ReadStorage<'a, MaskId>,
                       Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, dt, grav, terrain, mut vel, mut pos, mask, updater): Self::SystemData) {
        let grav = grav.0;
        let dt = dt.to_seconds();

        for (entity, vel, pos, mask) in (&*entities, &mut vel, &mut pos, &mask).join() {
            let next = Position::new(pos.x + vel.x * dt, pos.y + vel.y * dt);

            match terrain.line_collides(pos.as_i32(), next.as_i32()) {
                Some(point) => {
                    let _ = entities.delete(entity);

                    let crater = entities.create();
                    updater.insert(crater, TerrainMask::new(mask.0, point));
                },
                None => {
                    *pos = next;
                    vel.y += grav * dt;
                }
            }
        }
    }
}
