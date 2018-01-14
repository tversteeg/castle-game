use specs::*;

use physics::*;
use terrain::*;

#[derive(Component)]
pub struct Health(i32);

pub struct UnitSystem;
impl<'a> System<'a> for UnitSystem {
    type SystemData = (Entities<'a>,
                       Fetch<'a, DeltaTime>,
                       Fetch<'a, Gravity>,
                       Fetch<'a, Terrain>,
                       WriteStorage<'a, Health>);

    fn run(&mut self, (entities, dt, grav, terrain, mut health): Self::SystemData) {

    }
}

#[derive(Component)]
pub struct Walk(pub Rect);

#[derive(Component)]
pub struct Destination(pub f64);

pub struct WalkSystem;
impl<'a> System<'a> for WalkSystem {
    type SystemData = (Fetch<'a, DeltaTime>,
                       Fetch<'a, Gravity>,
                       Fetch<'a, Terrain>,
                       ReadStorage<'a, Walk>,
                       ReadStorage<'a, Destination>,
                       WriteStorage<'a, Position>);

    fn run(&mut self, (dt, grav, terrain, bounds, dest, mut pos): Self::SystemData) {

    }
}
