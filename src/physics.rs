use specs::*;

#[derive(Debug, Copy, Clone)]
pub struct Position {
    x: f32,
    y: f32
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
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
    x: f32,
    y: f32
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Velocity { x, y }
    }

    pub fn as_i32(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
}

impl Component for Velocity {
    type Storage = VecStorage<Self>;
}

pub struct Gravity(pub f32);

pub struct ProjectileSystem;

impl<'a> System<'a> for ProjectileSystem {
    type SystemData = (Fetch<'a, Gravity>,
                       ReadStorage<'a, Velocity>,
                       WriteStorage<'a, Position>);

    fn run(&mut self, (grav, vel, mut pos): Self::SystemData) {
        let grav = grav.0;

        for (vel, pos) in (&vel, &mut pos).join() {
            pos.x += vel.x;
            pos.y += vel.y;

            pos.y += grav;
        }
    }
}
