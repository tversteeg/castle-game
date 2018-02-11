extern crate blit;
extern crate direct_gui;
extern crate minifb;
extern crate specs;
extern crate line_drawing;
extern crate rand;
extern crate cgmath;
extern crate collision;
#[macro_use]
extern crate specs_derive;

mod draw;
mod physics;
mod terrain;
mod projectile;
mod ai;
mod level;
mod geom;

use minifb::*;
use specs::{World, DispatcherBuilder, Join};
use direct_gui::*;
use direct_gui::controls::*;
use std::time::{SystemTime, Duration};
use std::thread::sleep;
use std::collections::HashMap;

use draw::*;
use physics::*;
use terrain::*;
use projectile::*;
use ai::*;
use level::*;
use geom::*;

const WIDTH: usize = 640;
const HEIGHT: usize = 320;

const GRAVITY: f64 = 98.1;

macro_rules! load_resource {
    ($resources:expr; $render:expr; sprite => $e:expr) => {{
        $resources.insert($e.to_string(), $render.add_buf_from_memory($e, include_bytes!(concat!("../resources/sprites/", $e, ".png.blit"))))
    }};
    ($resources:expr; $render:expr; mask => $e:expr) => {{
        $resources.insert($e.to_string(), $render.add_buf_from_memory($e, include_bytes!(concat!("../resources/masks/", $e, ".png.blit"))))
    }};
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    let mut render = Render::new((WIDTH, HEIGHT));

    let mut resources = HashMap::new();
    load_resource!(resources; render; sprite => "projectile1");
    load_resource!(resources; render; sprite => "ally-soldier1");
    load_resource!(resources; render; sprite => "enemy-soldier1");
    load_resource!(resources; render; mask => "bighole1");

    // Setup game related things
    let mut world = World::new();

    // draw.rs
    world.register::<PixelParticle>();
    world.register::<MaskId>();
    world.register::<Sprite>();

    // terrain.rs
    world.register::<TerrainMask>();
    world.register::<TerrainCollapse>();

    // physics.rs
    world.register::<Point>();
    world.register::<BoundingBox>();
    world.register::<Velocity>();

    // ai.rs
    world.register::<Health>();
    world.register::<Walk>();
    world.register::<Destination>();
    world.register::<Ally>();
    world.register::<Enemy>();
    world.register::<Turret>();
    world.register::<Melee>();

    // projectile.rs
    world.register::<Damage>();

    // Resources to `Fetch`
    world.add_resource(Terrain::new((WIDTH, HEIGHT)));
    world.add_resource(Gravity(GRAVITY));
    world.add_resource(DeltaTime::new(1.0 / 60.0));
    world.add_resource(Images(resources));

    render.draw_background_from_memory(include_bytes!("../resources/sprites/background.png.blit"));
    render.draw_terrain_from_memory(&mut *world.write_resource::<Terrain>(), include_bytes!("../resources/sprites/level.png.blit"));

    place_turrets(&mut world, 1);

    let mut dispatcher = DispatcherBuilder::new()
        .add(ProjectileSystem, "projectile", &[])
        .add(ProjectileCollisionSystem, "projectile_collision", &["projectile"])
        .add(TerrainCollapseSystem, "terrain_collapse", &["projectile"])
        .add(WalkSystem, "walk", &[])
        .add(MeleeSystem, "melee", &["walk"])
        .add(TurretSystem, "turret", &[])
        .add(SpriteSystem, "sprite", &["projectile", "walk"])
        .add(ParticleSystem, "particle", &[])
        .build();

    // Setup minifb window related things
    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let title = format!("Castle Game {} - Press ESC to exit.", env!("CARGO_PKG_VERSION"));
    let mut window = Window::new(&title, WIDTH, HEIGHT, options).expect("Unable to open window");

    window.set_cursor_style(CursorStyle::Crosshair);

    // Setup the GUI system
    let mut gui = Gui::new((WIDTH as i32, HEIGHT as i32));

    let default_font = gui.default_font();
    let fps_ref = gui.register(Label::new(default_font).with_pos(2, 2).with_text("FPS"));

    // Game loop
    let mut time = SystemTime::now();
    let mut second = 0.0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut cs = ControlState {
            ..ControlState::default()
        };

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

        // Handle mouse events
        window.get_mouse_pos(MouseMode::Discard).map(|mouse| {
            cs.mouse_pos = (mouse.0 as i32, mouse.1 as i32);
            cs.mouse_down = window.get_mouse_down(MouseButton::Left);
        });

        dispatcher.dispatch(&mut world.res);

        // Add/remove entities added in dispatch through `LazyUpdate`
        world.maintain();

        // Render the sprites & masks
        let sprites = world.read::<Sprite>();
        let pixels = world.read::<PixelParticle>();
        let terrain_masks = world.read::<TerrainMask>();
        for entity in world.entities().join() {
            if let Some(sprite) = sprites.get(entity) {
                render.draw_foreground(sprite).unwrap();
            }

            if let Some(pixel) = pixels.get(entity) {
                render.draw_foreground_pixel(pixel.pos, pixel.color);
            }

            if let Some(mask) = terrain_masks.get(entity) {
                render.draw_mask_terrain(&mut *world.write_resource::<Terrain>(), mask).unwrap();

                let _ = world.entities().delete(entity);
            }
        }

        render.draw_final_buffer(&mut buffer, &*world.write_resource::<Terrain>());

        // Render the gui on the buffer
        gui.update(&cs);
        gui.draw_to_buffer(&mut buffer);

        // Finally draw the buffer on the window
        window.update_with_buffer(&buffer).unwrap();

        sleep(Duration::from_millis(1));
    }
}
