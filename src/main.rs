extern crate minifb;
extern crate blit;
extern crate specs;

mod draw;
mod sprite;
mod geom;

use minifb::*;
use blit::BlitBuffer;
use specs::{World, RunNow};

use draw::RenderSystem;
use sprite::Sprite;
use geom::Position;

const WIDTH: usize = 640;
const HEIGHT: usize = 320;

fn main() {
    let mut world = World::new();

    world.register::<Sprite>();
    world.register::<Position>();

    let bytes = include_bytes!("../resources/background.png.blit");
    world.create_entity()
        .with(Sprite::new(BlitBuffer::load_from_memory(bytes).unwrap()))
        .with(Position::new(0.0, 0.0))
        .build();

    let bytes = include_bytes!("../resources/level.png.blit");
    world.create_entity()
        .with(Sprite::new(BlitBuffer::load_from_memory(bytes).unwrap()))
        .with(Position::new(0.0, 0.0))
        .build();

    let mut render_system = RenderSystem::new((WIDTH, HEIGHT));

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new("Castle Game - Press ESC to exit", WIDTH, HEIGHT, options).expect("Unable to open window");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        /*
        window.get_mouse_pos(MouseMode::Discard).map(|mouse| {
            let screen_pos = ((mouse.1 as usize) * WIDTH) + mouse.0 as usize;
        });
        */

        window.get_keys().map(|keys| {
            for t in keys {
                match t {
                    Key::W => println!("holding w!"),
                    Key::T => println!("holding t!"),
                    _ => (),
                }
            }
        });

        render_system.run_now(&world.res);

        window.update_with_buffer(render_system.raw_buffer()).unwrap();
    }
}
