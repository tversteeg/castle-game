extern crate blit;
extern crate direct_gui;
extern crate minifb;
extern crate specs;
extern crate line_drawing;
extern crate aabb2;
#[macro_use]
extern crate specs_derive;

mod draw;
mod physics;
mod terrain;
mod projectile;
mod ai;

use minifb::*;
use specs::{World, DispatcherBuilder, Join};
use std::time::{SystemTime, Duration};
use std::thread::sleep;
use direct_gui::*;
use direct_gui::controls::*;

use draw::*;
use physics::*;
use terrain::*;
use projectile::*;
use ai::*;

const WIDTH: usize = 640;
const HEIGHT: usize = 320;

const GRAVITY: f64 = 98.1;

macro_rules! load_resource {
    ($render:expr; sprite => $e:expr) => {{
        $render.add_buf_from_memory($e, include_bytes!(concat!("../resources/sprites/", $e, ".png.blit")))
    }};
    ($render:expr; mask => $e:expr) => {{
        $render.add_buf_from_memory($e, include_bytes!(concat!("../resources/masks/", $e, ".png.blit")))
    }};
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    // Setup game related things
    let mut world = World::new();

    // draw.rs
    world.register::<Sprite>();
    world.register::<MaskId>();

    // terrain.rs
    world.register::<TerrainMask>();

    // physics.rs
    world.register::<Position>();
    world.register::<Velocity>();
    world.register::<Rect>();
    world.register::<BoundingBox>();

    // ai.rs
    world.register::<Health>();
    world.register::<Walk>();
    world.register::<Destination>();

    // projectile.rs
    world.register::<Damage>();

    // Resources to `Fetch`
    world.add_resource(Terrain::new((WIDTH, HEIGHT)));
    world.add_resource(Gravity(GRAVITY));
    world.add_resource(DeltaTime::new(1.0 / 60.0));

    let mut render = Render::new((WIDTH, HEIGHT));

    render.draw_background_from_memory(include_bytes!("../resources/sprites/background.png.blit"));
    render.draw_terrain_from_memory(&mut *world.write_resource::<Terrain>(), include_bytes!("../resources/sprites/level.png.blit"));

    let projectile = load_resource!(render; sprite => "projectile1");
    let soldier = load_resource!(render; sprite => "soldier1");

    let projectile_mask = load_resource!(render; mask => "bighole1");

    world.create_entity()
        .with(Sprite::new(soldier))
        .with(Position::new(10.0, 200.0))
        .with(Velocity::new(0.0, 0.0))
        .with(Walk::new(Rect::new(1.0, 5.0, 3.0, 5.0), 10.0))
        .with(BoundingBox(Rect::new(0.0, 0.0, 10.0, 10.0)))
        .with(Destination(630.0))
        .with(Health(100.0))
        .build();

    let mut dispatcher = DispatcherBuilder::new()
        .add(ProjectileSystem, "projectile", &[])
        .add(ProjectileCollisionSystem, "projectile_collision", &["projectile"])
        .add(HealthSystem, "health", &["projectile_collision"])
        .add(WalkSystem, "walk", &[])
        .add(SpriteSystem, "sprite", &["projectile", "walk"])
        .build();

    // Setup minifb window related things
    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new("Castle Game - Click to shoot & ESC to exit.", WIDTH, HEIGHT, options).expect("Unable to open window");

    window.set_cursor_style(CursorStyle::Crosshair);

    // Setup the GUI system
    let mut gui = Gui::new((WIDTH as i32, HEIGHT as i32));

    let default_font = gui.default_font();
    let fps_ref = gui.register(Label::new(default_font).with_pos(2, 2).with_text("FPS"));

    // Game loop
    let mut time = SystemTime::now();
    let mut second = 0.0;
    let mut shot_time = 0.0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Calculate the deltatime
        {
            let mut delta = world.write_resource::<DeltaTime>();
            *delta = DeltaTime(time.elapsed().unwrap());
            time = SystemTime::now();

            // Update the title every second
            second += delta.to_seconds();
            if second > 0.5 {
                second -= 0.5;

                let fps = &format!("FPS {:.2}", 1.0 / delta.to_seconds());
                gui.get_mut::<Label>(fps_ref).unwrap().set_text(fps);
            }
        }

        shot_time -= world.read_resource::<DeltaTime>().to_seconds();
        // Handle mouse events
        window.get_mouse_pos(MouseMode::Discard).map(|mouse| {
            if shot_time <= 0.0 && window.get_mouse_down(MouseButton::Left) {
                shot_time = 0.3;

                let x = 630.0;
                let y = 190.0;
                let time = 2.0;

                let vx = ((mouse.0 as f64) - x) / time;
                let vy = (mouse.1 as f64 + 0.5 * -GRAVITY * time * time - y) / time;

                // Spawn a projectile
                world.create_entity()
                    .with(Sprite::new(projectile))
                    .with(MaskId(projectile_mask))
                    .with(BoundingBox(Rect::new(0.0, 0.0, 5.0, 5.0)))
                    .with(Damage(30.0))
                    .with(Position::new(x, y))
                    .with(Velocity::new(vx, vy))
                    .build();
            }
        });

        dispatcher.dispatch(&mut world.res);

        // Add/remove entities added in dispatch through `LazyUpdate`
        world.maintain();

        // Render the sprites & masks
        let sprites = world.read::<Sprite>();
        let terrain_masks = world.read::<TerrainMask>();
        for entity in world.entities().join() {
            if let Some(sprite) = sprites.get(entity) {
                render.draw_foreground(sprite).unwrap();
            }
            if let Some(mask) = terrain_masks.get(entity) {
                render.draw_mask_terrain(&mut *world.write_resource::<Terrain>(), mask).unwrap();

                let _ = world.entities().delete(entity);
            }
        }

        render.draw_final_buffer(&mut buffer, &*world.write_resource::<Terrain>());

        // Render the gui on the buffer
        gui.draw_to_buffer(&mut buffer);

        // Finally draw the buffer on the window
        window.update_with_buffer(&buffer).unwrap();

        sleep(Duration::from_millis(1));
    }
}
