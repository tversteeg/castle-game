use specs::*;

use physics::*;
use terrain::*;

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

                        pos.x += walk.speed * dt * dest.0.signum();
                    },
                    None => break
                }
            }
        }
    }
}
