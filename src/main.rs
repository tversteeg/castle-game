extern crate minifb;
extern crate blit;
extern crate image;
extern crate specs;

mod window;
mod world;
mod draw;
mod sprite;
mod geom;

use world::GameWorld;

const WIDTH: usize = 640;
const HEIGHT: usize = 320;

fn main() {
    let mut world = GameWorld::new((WIDTH, HEIGHT));

    world.main_loop();
}
