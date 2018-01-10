extern crate minifb;
extern crate blit;
extern crate specs;
#[macro_use]
extern crate specs_derive;

mod draw;
mod physics;

use minifb::*;
use specs::{World, DispatcherBuilder, Join};
use std::time::{SystemTime, Duration};
use std::thread::sleep;

use draw::{Render, Sprite, SpriteSystem};
use physics::*;

const WIDTH: usize = 640;
const HEIGHT: usize = 320;

const GRAVITY: f64 = 98.1;

fn main() {
    let mut render = Render::new((WIDTH, HEIGHT));

    let background = render.add_from_memory(include_bytes!("../resources/background.png.blit"));
    let level = render.add_from_memory(include_bytes!("../resources/level.png.blit"));
    let projectile = render.add_from_memory(include_bytes!("../resources/projectile1.png.blit"));

    let mut world = World::new();

    world.register::<Sprite>();
    world.register::<Position>();
    world.register::<Velocity>();

    world.add_resource(Gravity(GRAVITY));
    world.add_resource(DeltaTime::new(1.0 / 60.0));

    world.create_entity()
        .with(Sprite::new(background))
        .with(Position::new(0.0, 0.0))
        .build();

    world.create_entity()
        .with(Sprite::new(level))
        .with(Position::new(0.0, (HEIGHT - render.sprite_size(level).unwrap().1) as f64))
        .build();

    let mut dispatcher = DispatcherBuilder::new()
        .add(ProjectileSystem, "projectile", &[])
        .add(SpriteSystem, "sprite", &["projectile"])
        .build();

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new("Castle Game", WIDTH, HEIGHT, options).expect("Unable to open window");

    window.set_cursor_style(CursorStyle::Crosshair);

    let mut time = SystemTime::now();
    let mut second = 0.0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Calculate the deltatime
        {
            let mut delta = world.write_resource::<DeltaTime>();
            *delta = DeltaTime(time.elapsed().unwrap());
            time = SystemTime::now();

            // Update the title every second
            second += delta.to_seconds();
            if second > 1.0 {
                second -= 1.0;

                let title = &format!("Castle Game - Press ESC to exit, FPS: {:.2}", 1.0 / delta.to_seconds());
                window.set_title(title);
            }
        }

        // Handle mouse events
        window.get_mouse_pos(MouseMode::Discard).map(|mouse| {
            if second > 0.9 && window.get_mouse_down(MouseButton::Left) {
                let x = 630.0;
                let y = 200.0;
                let time = 3.0;

                let vx = ((mouse.0 as f64) - x) / time;
                let vy = (mouse.1 as f64 + 0.5 * -GRAVITY * time * time - y) / time;

                world.create_entity()
                    .with(Sprite::new(projectile))
                    .with(Position::new(x, y))
                    .with(Velocity::new(vx, vy))
                    .build();
            }
        });

        dispatcher.dispatch(&mut world.res);

        // Render the sprites
        let sprites = world.read::<Sprite>();
        for entity in world.entities().join() {
            if let Some(sprite) = sprites.get(entity) {
                render.draw(sprite).unwrap();
            }
        }

        window.update_with_buffer(&render.buffer).unwrap();

        sleep(Duration::from_millis(1));
    }
}
