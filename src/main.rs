mod ai;
mod audio;
mod draw;
mod geom;
mod gui;
mod level;
mod physics;
mod projectile;
mod terrain;
mod turret;
mod unit;

use minifb::*;
use rust_embed::RustEmbed;
use specs::{DispatcherBuilder, Join, World, WorldExt};
use std::{
    collections::HashMap,
    thread,
    time::{Duration, SystemTime},
};

use ai::*;
use audio::Audio;
use draw::*;
use geom::*;
use gui::*;
use level::*;
use physics::*;
use projectile::*;
use terrain::*;
use turret::*;
use unit::*;

const WIDTH: usize = 1280;
const HEIGHT: usize = 540;

const GRAVITY: f64 = 98.1;

#[derive(RustEmbed)]
#[folder = "$OUT_DIR/sprites/"]
struct SpriteFolder;

impl SpriteFolder {
    fn load_sprite(render: &mut Render, resources: &mut HashMap<String, usize>, name: &str) {
        let mut file = name.to_owned();
        file.push_str(".blit");

        let buf = Self::get(&*file).unwrap();

        resources.insert(name.to_string(), render.add_buf_from_memory(name, &buf));
    }

    fn load_anim(render: &mut Render, resources: &mut HashMap<String, usize>, name: &str) {
        let mut file = name.to_owned();
        file.push_str(".anim");

        let buf = Self::get(&*file).unwrap();

        resources.insert(
            name.to_string(),
            render.add_anim_buf_from_memory(name, &buf),
        );
    }
}

#[derive(RustEmbed)]
#[folder = "$OUT_DIR/masks/"]
struct MaskFolder;

impl MaskFolder {
    fn load_sprite(render: &mut Render, resources: &mut HashMap<String, usize>, name: &str) {
        let mut file = name.to_owned();
        file.push_str(".blit");

        let buf = Self::get(&*file).unwrap();

        resources.insert(name.to_string(), render.add_buf_from_memory(name, &buf));
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; (WIDTH * HEIGHT) as usize];

    let mut render = Render::new((WIDTH, HEIGHT));

    let mut resources = HashMap::new();

    SpriteFolder::load_anim(&mut render, &mut resources, "ally-archer1");
    SpriteFolder::load_sprite(&mut render, &mut resources, "ally-melee1");
    SpriteFolder::load_sprite(&mut render, &mut resources, "enemy-melee1");
    SpriteFolder::load_sprite(&mut render, &mut resources, "enemy-archer1");
    SpriteFolder::load_sprite(&mut render, &mut resources, "projectile1");

    MaskFolder::load_sprite(&mut render, &mut resources, "bighole1");

    // Setup game related things
    let mut world = World::new();

    // draw.rs
    world.register::<PixelParticle>();
    world.register::<MaskId>();
    world.register::<Anim>();
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
    world.insert(Terrain::new((WIDTH, HEIGHT)));
    world.insert(Gravity(GRAVITY));
    world.insert(DeltaTime::new(1.0 / 60.0));
    world.insert(Images(resources));
    world.insert(Audio::new());

    render.draw_background_from_memory(&SpriteFolder::get("background.blit").unwrap());
    render.draw_terrain_from_memory(
        &mut *world.write_resource::<Terrain>(),
        &SpriteFolder::get("level.blit").unwrap(),
    );

    place_turrets(&mut world, 1);

    let mut dispatcher = DispatcherBuilder::new()
        .with(ProjectileSystem, "projectile", &[])
        .with(ArrowSystem, "arrow", &["projectile"])
        .with(
            ProjectileCollisionSystem,
            "projectile_collision",
            &["projectile"],
        )
        .with(
            ProjectileRemovalFromMaskSystem,
            "projectile_removal_from_mask",
            &["projectile"],
        )
        .with(TerrainCollapseSystem, "terrain_collapse", &["projectile"])
        .with(WalkSystem, "walk", &[])
        .with(UnitFallSystem, "unit_fall", &["walk"])
        .with(UnitResumeWalkingSystem, "unit_resume_walking", &["walk"])
        .with(UnitCollideSystem, "unit_collide", &["walk"])
        .with(MeleeSystem, "melee", &["walk"])
        .with(HealthBarSystem, "health_bar", &["walk"])
        .with(TurretUnitSystem, "turret_unit", &["walk"])
        .with(TurretSystem, "turret", &["turret_unit"])
        .with(SpriteSystem, "sprite", &["projectile", "walk"])
        .with(AnimSystem, "anim", &["projectile", "walk"])
        .with(ParticleSystem, "particle", &[])
        .with(FloatingTextSystem, "floating_text", &[])
        .build();

    // Setup minifb window related things
    let title = format!(
        "Castle Game {} - Press ESC to exit.",
        env!("CARGO_PKG_VERSION")
    );
    let options = WindowOptions {
        borderless: false,
        title: true,
        scale: Scale::X2,
        scale_mode: ScaleMode::AspectRatioStretch,
        ..Default::default()
    };
    let mut window = Window::new(&title, WIDTH, HEIGHT, options).expect("Unable to open window");

    // Setup the GUI system
    let mut gui = IngameGui::new((WIDTH as i32, HEIGHT as i32));

    {
        // Start the audio
        let mut audio = world.write_resource::<Audio>();
        audio.run();
    }

    // Game loop
    let mut time = SystemTime::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Calculate the delta-time
        {
            let mut delta = world.write_resource::<DeltaTime>();
            *delta = DeltaTime(time.elapsed().unwrap());
            time = SystemTime::now();
        }

        // Handle mouse events
        if let Some(mouse) = window.get_mouse_pos(MouseMode::Discard) {
            gui.handle_mouse(
                (mouse.0 as i32, mouse.1 as i32),
                window.get_mouse_down(MouseButton::Left),
            );
        };

        dispatcher.dispatch(&world);

        // Add/remove entities added in dispatch through `LazyUpdate`
        world.maintain();

        // Render the sprites & masks
        {
            render.draw_terrain_and_background(&mut buffer, &*world.write_resource::<Terrain>());

            let mut anims = world.write_storage::<Anim>();
            let sprites = world.read_storage::<Sprite>();
            let lines = world.read_storage::<Line>();
            let pixels = world.read_storage::<PixelParticle>();
            let terrain_masks = world.read_storage::<TerrainMask>();
            let health_bars = world.read_storage::<HealthBar>();
            for entity in world.entities().join() {
                if let Some(anim) = anims.get_mut(entity) {
                    render
                        .update_anim(anim, world.read_resource::<DeltaTime>().0)
                        .unwrap();

                    render.draw_foreground_anim(&mut buffer, anim).unwrap();
                }

                if let Some(sprite) = sprites.get(entity) {
                    render.draw_foreground(&mut buffer, sprite).unwrap();
                }

                if let Some(line) = lines.get(entity) {
                    render.draw_foreground_line(&mut buffer, line.p1, line.p2, line.color);
                }

                if let Some(pixel) = pixels.get(entity) {
                    render.draw_foreground_pixel(&mut buffer, pixel.pos, pixel.color);
                }

                if let Some(health_bar) = health_bars.get(entity) {
                    render.draw_healthbar(
                        &mut buffer,
                        health_bar.pos,
                        health_bar.health / health_bar.max_health,
                        health_bar.width,
                    );
                }

                if let Some(mask) = terrain_masks.get(entity) {
                    render
                        .draw_mask_terrain(&mut *world.write_resource::<Terrain>(), mask)
                        .unwrap();

                    // Immediately remove the mask after drawing it
                    let _ = world.entities().delete(entity);
                }
            }
        }

        // Update the gui system and receive a possible event
        match gui.update() {
            GuiEvent::BuyArcherButton => {
                buy_archer(&mut world);
            }
            GuiEvent::BuySoldierButton => {
                buy_soldier(&mut world);
            }
            _ => (),
        }

        // Render the floating text
        let floating_texts = world.read_storage::<FloatingText>();

        // Render the gui on the buffer
        gui.render(&mut buffer);
        for entity in world.entities().join() {
            if let Some(text) = floating_texts.get(entity) {
                gui.draw_label(&mut buffer, &text.text, text.pos.as_i32());
            }
        }

        // Finally draw the buffer on the window
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        thread::sleep(Duration::from_millis(1));
    }
}
