use specs::*;
use std::time::Duration;

#[derive(Debug, Copy, Clone)]
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

impl Component for Position {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Copy, Clone)]
pub struct Velocity {
    x: f64,
    y: f64
}

impl Velocity {
    pub fn new(x: f64, y: f64) -> Self {
        Velocity { x, y }
    }
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
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
    type SystemData = (Fetch<'a, DeltaTime>,
                       Fetch<'a, Gravity>,
                       WriteStorage<'a, Velocity>,
                       WriteStorage<'a, Position>);

    fn run(&mut self, (dt, grav, mut vel, mut pos): Self::SystemData) {
        let grav = grav.0;
        let dt = dt.to_seconds();

        for (vel, pos) in (&mut vel, &mut pos).join() {
            pos.x += vel.x * dt;
            pos.y += vel.y * dt;

            vel.y += grav * dt;
        }
    }
}
