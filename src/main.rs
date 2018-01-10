extern crate minifb;
extern crate blit;
extern crate specs;
#[macro_use]
extern crate specs_derive;

mod draw;
mod physics;

use minifb::*;
use specs::{World, DispatcherBuilder, Join};

use draw::{Render, Sprite, SpriteSystem};
use physics::*;

const WIDTH: usize = 640;
const HEIGHT: usize = 320;

fn main() {
    let mut render = Render::new((WIDTH, HEIGHT));

    let background = render.add_from_memory(include_bytes!("../resources/background.png.blit"));
    let level = render.add_from_memory(include_bytes!("../resources/level.png.blit"));

    let mut world = World::new();

    world.register::<Sprite>();
    world.register::<Position>();
    world.register::<Velocity>();

    world.add_resource(Gravity(0.05));

    world.create_entity()
        .with(Sprite::new(background))
        .with(Position::new(0.0, 0.0))
        .build();

    world.create_entity()
        .with(Sprite::new(level))
        .with(Position::new(0.0, 0.0))
        .build();

    let mut dispatcher = DispatcherBuilder::new()
        .add(ProjectileSystem, "projectile", &[])
        .add(SpriteSystem, "sprite", &["projectile"])
        .build();

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new("Castle Game - Press ESC to exit", WIDTH, HEIGHT, options).expect("Unable to open window");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        dispatcher.dispatch(&mut world.res);

        // Render the sprites
        let sprites = world.read::<Sprite>();
        for entity in world.entities().join() {
            if let Some(sprite) = sprites.get(entity) {
                render.draw(sprite).unwrap();
            }
        }

        window.update_with_buffer(&render.buffer).unwrap();
    }
}
