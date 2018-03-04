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
mod unit;
mod turret;
mod level;
mod geom;
mod gui;

use minifb::*;
use specs::{World, DispatcherBuilder, Join};
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
use gui::*;
use unit::*;
use turret::*;

const WIDTH: usize = 1280;
const HEIGHT: usize = 540;

const GRAVITY: f64 = 98.1;

macro_rules! load_resource {
    ($resources:expr; $render:expr; sprite => $e:expr) => {{
        $resources.insert($e.to_string(), $render.add_buf_from_memory($e, include_bytes!(concat!("../resources/sprites/", $e, ".blit"))))
    }};
    ($resources:expr; $render:expr; mask => $e:expr) => {{
        $resources.insert($e.to_string(), $render.add_buf_from_memory($e, include_bytes!(concat!("../resources/masks/", $e, ".blit"))))
    }};
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    let mut render = Render::new((WIDTH, HEIGHT));

    let mut resources = HashMap::new();
    load_resource!(resources; render; sprite => "ally-melee1");
    load_resource!(resources; render; sprite => "ally-archer1");
    load_resource!(resources; render; sprite => "enemy-melee1");
    load_resource!(resources; render; sprite => "enemy-archer1");

    load_resource!(resources; render; sprite => "projectile1");

    load_resource!(resources; render; mask => "bighole1");

    // Setup game related things
    let mut world = World::new();

    // draw.rs
    world.register::<PixelParticle>();
    world.register::<MaskId>();
    world.register::<Sprite>();
    world.register::<Line>();

    // terrain.rs
    world.register::<TerrainMask>();
    world.register::<TerrainCollapse>();

    // physics.rs
    world.register::<WorldPosition>();
    world.register::<Point>();
    world.register::<BoundingBox>();
    world.register::<Velocity>();

    // ai.rs
    world.register::<Destination>();
    world.register::<Ally>();
    world.register::<Enemy>();
    world.register::<Melee>();

    // unit.rs
    world.register::<UnitState>();
    world.register::<Health>();
    world.register::<HealthBar>();
    world.register::<Walk>();

    // turret.rs
    world.register::<Turret>();
    world.register::<TurretOffset>();

    // projectile.rs
    world.register::<Projectile>();
    world.register::<ProjectileSprite>();
    world.register::<ProjectileBoundingBox>();
    world.register::<IgnoreCollision>();
    world.register::<Arrow>();
    world.register::<Damage>();

    // gui.rs
    world.register::<FloatingText>();

    // Resources to `Fetch`
    world.add_resource(Terrain::new((WIDTH, HEIGHT)));
    world.add_resource(Gravity(GRAVITY));
    world.add_resource(DeltaTime::new(1.0 / 60.0));
    world.add_resource(Images(resources));

    render.draw_background_from_memory(include_bytes!("../resources/sprites/background.blit"));
    render.draw_terrain_from_memory(&mut *world.write_resource::<Terrain>(), include_bytes!("../resources/sprites/level.blit"));

    place_turrets(&mut world, 1);

    let mut dispatcher = DispatcherBuilder::new()
        .add(ProjectileSystem, "projectile", &[])
        .add(ArrowSystem, "arrow", &["projectile"])
        .add(ProjectileCollisionSystem, "projectile_collision", &["projectile"])
        .add(ProjectileRemovalFromMaskSystem, "projectile_removal_from_mask", &["projectile"])
        .add(TerrainCollapseSystem, "terrain_collapse", &["projectile"])
        .add(WalkSystem, "walk", &[])
        .add(UnitFallSystem, "unit_fall", &["walk"])
        .add(UnitResumeWalkingSystem, "unit_resume_walking", &["walk"])
        .add(UnitCollideSystem, "unit_collide", &["walk"])
        .add(MeleeSystem, "melee", &["walk"])
        .add(HealthBarSystem, "health_bar", &["walk"])
        .add(TurretUnitSystem, "turret_unit", &["walk"])
        .add(TurretSystem, "turret", &["turret_unit"])
        .add(SpriteSystem, "sprite", &["projectile", "walk"])
        .add(ParticleSystem, "particle", &[])
        .add(FloatingTextSystem, "floating_text", &[])
        .build();

    // Setup minifb window related things
    let title = format!("Castle Game {} - Press ESC to exit.", env!("CARGO_PKG_VERSION"));
    let mut window = Window::new(&title, WIDTH, HEIGHT, WindowOptions::default()).expect("Unable to open window");

    // Setup the GUI system
    let mut gui = IngameGui::new((WIDTH as i32, HEIGHT as i32));

    // Game loop
    let mut time = SystemTime::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        {
            let mut delta = world.write_resource::<DeltaTime>();
            *delta = DeltaTime(time.elapsed().unwrap());
            time = SystemTime::now();
        }

        // Handle mouse events
        window.get_mouse_pos(MouseMode::Discard).map(|mouse| {
            gui.handle_mouse((mouse.0 as i32, mouse.1 as i32), window.get_mouse_down(MouseButton::Left));
        });

        dispatcher.dispatch(&mut world.res);

        // Add/remove entities added in dispatch through `LazyUpdate`
        world.maintain();

        // Render the sprites & masks
        {
            let sprites = world.read::<Sprite>();
            let lines = world.read::<Line>();
            let pixels = world.read::<PixelParticle>();
            let terrain_masks = world.read::<TerrainMask>();
            let health_bars = world.read::<HealthBar>();
            for entity in world.entities().join() {
                if let Some(sprite) = sprites.get(entity) {
                    render.draw_foreground(sprite).unwrap();
                }

                if let Some(line) = lines.get(entity) {
                    render.draw_foreground_line(line.p1, line.p2, line.color);
                }

                if let Some(pixel) = pixels.get(entity) {
                    render.draw_foreground_pixel(pixel.pos, pixel.color);
                }

                if let Some(bar) = health_bars.get(entity) {
                    render.draw_healthbar(bar.pos, bar.health / bar.max_health, bar.width);
                }

                if let Some(mask) = terrain_masks.get(entity) {
                    render.draw_mask_terrain(&mut *world.write_resource::<Terrain>(), mask).unwrap();

                    // Immediately remove the mask after drawing it
                    let _ = world.entities().delete(entity);
                }
            }

            render.draw_final_buffer(&mut buffer, &*world.write_resource::<Terrain>());
        }

        // Update the gui system and receive a possible event
        match gui.update() {
            GuiEvent::BuyArcherButton => {
                buy_archer(&mut world);
            },
            GuiEvent::BuySoldierButton => {
              buy_soldier(&mut world);  
            },
            _ => ()
        }

        // Render the floating text
        let floating_texts = world.read::<FloatingText>();

        // Render the gui on the buffer
        gui.render(&mut buffer);
        for entity in world.entities().join() {
            if let Some(text) = floating_texts.get(entity) {
                gui.draw_label(&mut buffer, &text.text, text.pos.as_i32());
            }
        }

        // Finally draw the buffer on the window
        window.update_with_buffer(&buffer).unwrap();

        sleep(Duration::from_millis(1));
    }
}
