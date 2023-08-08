use serde::Deserialize;
use vek::{Aabr, Extent2, Vec2};

use crate::{
    camera::Camera,
    input::Input,
    math::{Iso, Rotation},
    object::ObjectSettings,
    physics::{
        collision::{shape::Shape, CollisionResponse},
        rigidbody::{RigidBody, RigidBodyIndex},
        Physics,
    },
    terrain::Terrain,
    SIZE,
};

/// Physics grid step size.
const PHYSICS_GRID_STEP: u16 = 10;

/// Asset paths.
const LEVEL: &str = "level.grass-1";
const SPEAR: &str = "projectile.spear-1";
const CRATE: &str = "object.crate-1";

/// Draw things for debugging purposes.
pub struct DebugDraw {
    /// What debug info to show, zero is show nothing.
    screen: u8,
    /// Mouse position.
    mouse: Vec2<f32>,
    /// Physics engine debug.
    ///
    /// All physics happen within the screen space.
    physics: Physics<
        { SIZE.w as u16 },
        { SIZE.h as u16 },
        PHYSICS_GRID_STEP,
        4,
        { (SIZE.w / PHYSICS_GRID_STEP as usize) * (SIZE.h / PHYSICS_GRID_STEP as usize) },
    >,
    /// Physics rigidbodies with the asset they belong to.
    rigidbodies: Vec<(RigidBodyIndex, &'static str)>,
    /// Rigidbodies with collisions.
    collisions: Vec<(RigidBodyIndex, RigidBodyIndex, CollisionResponse)>,
}

impl DebugDraw {
    /// Setup with default.
    pub fn new() -> Self {
        let mouse = Vec2::zero();
        let physics = Physics::new();
        let screen = crate::settings().debug.start_screen;
        let rigidbodies = Vec::new();
        let collisions = Vec::new();

        let mut debug = Self {
            screen,
            mouse,
            physics,
            rigidbodies,
            collisions,
        };

        debug.setup();

        debug
    }

    /// Update the debug state.
    pub fn update(&mut self, input: &Input, dt: f32) {
        puffin::profile_function!();

        // When space is released
        if input.space.is_released() {
            self.screen += 1;

            self.setup();
        }

        if self.screen == 0 {
            return;
        }

        // Store the mouse state
        self.mouse = input.mouse_pos.as_();

        if self.screen == 1 || self.screen == 2 {
            if input.left_mouse.is_released() {
                // Shape is based on the size of the image
                let object = crate::asset::<ObjectSettings>(SPEAR);

                self.rigidbodies.push((
                    self.physics
                        .add_rigidbody(object.rigidbody(self.mouse.as_())),
                    SPEAR,
                ));
            }

            if self.screen == 1 {
                // Make the first rigidbody follow the mouse
                self.physics.set_position(
                    self.rigidbodies[0].0,
                    self.mouse.numcast().unwrap_or_default(),
                );
            }

            // Update the physics.
            self.physics.step(dt);

            // Keep track of all collisions in an ugly way
            self.collisions = self.physics.colliding_rigid_bodies();
        }
    }

    /// Draw things for debugging purposes.
    pub fn render(&self, canvas: &mut [u32]) {
        puffin::profile_function!();

        if self.screen == 0 {
            return;
        }

        // Draw which screen we are on
        self.text(
            match self.screen {
                1 => "Rope physics",
                2 => "Box collisions",
                3 => "Sprite rotations",
                4 => "SAT collision detection",
                _ => "Undefined",
            },
            Vec2::new(20.0, 30.0),
            canvas,
        );

        if self.screen == 1 || self.screen == 2 {
            self.text(
                &format!(
                    "Rigidbodies: {}\nCollisions: {}",
                    self.rigidbodies.len(),
                    self.collisions.len()
                ),
                Vec2::new(SIZE.w as f32 - 100.0, 10.0),
                canvas,
            );

            // Draw physics sprites
            for (rigidbody, sprite_path) in self.rigidbodies.iter() {
                let rigidbody = self.physics.rigidbody(*rigidbody);

                self.physics_object(
                    rigidbody.iso(),
                    rigidbody.is_sleeping(),
                    sprite_path,
                    canvas,
                );
            }

            // Draw collisions
            for (a, b, response) in self.collisions.iter() {
                let a = self.physics.rigidbody(*a);
                let b = self.physics.rigidbody(*b);

                // Draw AABR
                self.aabr(a.aabr(), canvas, 0xFFFF0000);
                self.aabr(b.aabr(), canvas, 0xFFFF00FF);
                self.collision_response(response, a.iso(), b.iso(), canvas);
            }

            // Draw attachment positions
            for (a, b) in self.physics.debug_info_constraints() {
                self.circle(a.as_(), canvas, 0xFFFF0000);
                self.circle(b.as_(), canvas, 0xFFFF00FF);
            }
        } else if self.screen == 3 {
            // Draw rotating sprites
            for (index, asset) in [SPEAR, CRATE].iter().enumerate() {
                self.render_rotatable_to_mouse_sprite(
                    Vec2::new(50.0, 50.0 + index as f32 * 50.0),
                    asset,
                    canvas,
                );
            }
        } else if self.screen == 4 {
            // Draw collision between rotated rectangles
            let object = crate::asset::<ObjectSettings>(CRATE);
            let shape = object.shape();

            let mouse_iso = Iso::new(self.mouse.as_(), Rotation::from_degrees(23f32));

            // Detect collisions with the heightmap
            let level_object = crate::asset::<ObjectSettings>(LEVEL);
            let level_pos = Vec2::new(0.0, 100.0);
            let level_iso = Iso::from_pos(level_pos);

            self.physics_object(level_iso, false, LEVEL, canvas);

            // Draw the collision information
            for response in level_object.shape().collides(level_iso, &shape, mouse_iso) {
                self.collision_response(&response, level_iso, mouse_iso, canvas);
            }

            // Draw the box
            self.physics_object(mouse_iso, false, CRATE, canvas);

            for (index, rot) in [0, 90, 45, 23].into_iter().enumerate() {
                let pos = Vec2::new(
                    SIZE.w as f32 / 2.0 - 60.0 + index as f32 * 30.0,
                    SIZE.h as f32 / 2.0,
                );
                let rot = Rotation::from_degrees(rot as f32);
                let iso = Iso::new(pos, rot);

                // Draw the box
                self.physics_object(iso, false, CRATE, canvas);

                // Draw the collision information
                for response in shape.collides(iso, &shape, mouse_iso) {
                    self.collision_response(&response, iso, mouse_iso, canvas);
                }
            }
        }
    }

    /// Setup the screen.
    fn setup(&mut self) {
        match self.screen {
            1 => self.setup_physics_scene_1(),
            2 => self.setup_physics_scene_2(),
            3 | 4 => (),
            _ => self.screen = 0,
        }
    }

    /// Setup a new physics scene with a rope structure.
    fn setup_physics_scene_1(&mut self) {
        self.physics = Physics::new();

        // Shape is based on the size of the image
        let object = crate::asset::<ObjectSettings>(SPEAR);

        // Create some test rigidbodies
        let mut x = 50.0;
        self.rigidbodies = [(); 5]
            .iter()
            .enumerate()
            .map(|(_i, _)| {
                x += 10.0;
                (
                    self.physics.add_rigidbody(
                        object.rigidbody(Vec2::new(SIZE.w as f32 / 2.0 + x, SIZE.h as f32 / 2.0)),
                    ),
                    SPEAR,
                )
            })
            .collect();

        // Connect each rigidbody with the previous to create a rope
        for i in 1..self.rigidbodies.len() {
            // Offset the attachments to prevent collisions
            self.physics.add_distance_constraint(
                self.rigidbodies[i - 1].0,
                Vec2::new(-object.width() / 2.0 - 10.0, 0.0),
                self.rigidbodies[i].0,
                Vec2::new(object.width() / 2.0 + 10.0, 0.0),
                5.0,
                0.0001,
            );
        }
    }

    /// Setup a new physics scene with boxes.
    fn setup_physics_scene_2(&mut self) {
        self.physics = Physics::new();

        // Shape is based on the size of the image
        let object = crate::asset::<ObjectSettings>(CRATE);

        let y_offset = (SIZE.h - SIZE.h / 4) as i32;

        // Create a nice pyramid
        self.rigidbodies =
            [
                // Layer 1
                (0, -70),
                // Layer 2
                (-8, -50),
                (8, -50),
                // Layer 3
                (-16, -30),
                (0, -30),
                (16, -30),
                // Layer 4
                (-24, -10),
                (-8, -10),
                (8, -10),
                (24, -10),
            ]
            .iter()
            .map(|(x, y)| {
                (
                    self.physics.add_rigidbody(object.rigidbody(
                        (Vec2::new(*x, *y) + Vec2::new(SIZE.w as i32 / 2, y_offset)).as_(),
                    )),
                    CRATE,
                )
            })
            .collect();

        // Don't let them fall through the ground
        let terrain = Terrain::new(&mut self.physics);
        self.rigidbodies
            .push((terrain.rigidbody, crate::terrain::ASSET_PATH));
    }

    /// Draw a rotatable sprite pointing towards the mouse.
    fn render_rotatable_sprite(&self, iso: Iso, sprite_path: &str, canvas: &mut [u32]) {
        let sprite = crate::rotatable_sprite(sprite_path);

        sprite.render(iso, canvas, &Camera::default());
        self.text(
            &format!("{}", iso.rot.to_degrees().round()),
            iso.pos + Vec2::new(0.0, 10.0),
            canvas,
        );
    }

    /// Draw a rotatable sprite towards the mouse.
    fn render_rotatable_to_mouse_sprite(
        &self,
        pos: Vec2<f32>,
        sprite_path: &str,
        canvas: &mut [u32],
    ) {
        // Draw rotating sprites
        let delta: Vec2<f32> = (self.mouse - pos).numcast().unwrap_or_default();
        let rot = delta.y.atan2(delta.x);
        self.render_rotatable_sprite(Iso::new(pos, rot), sprite_path, canvas);
    }

    /// Render text.
    fn text(&self, text: &str, pos: Vec2<f32>, canvas: &mut [u32]) {
        crate::font("font.debug").render(text, pos, canvas);
    }

    /// Draw a bounding rectangle.
    fn aabr(&self, aabr: Aabr<f32>, canvas: &mut [u32], color: u32) {
        if aabr.max.x >= SIZE.w as f32 || aabr.max.y >= SIZE.h as f32 {
            return;
        }

        let aabr: Aabr<usize> = aabr.as_().intersection(Aabr {
            min: Vec2::zero(),
            max: Vec2::new(SIZE.w - 1, SIZE.h - 1),
        });

        for y in aabr.min.y.clamp(0, SIZE.h)..aabr.max.y.clamp(0, SIZE.h) {
            canvas[aabr.min.x + y * SIZE.h] = color;
            canvas[aabr.max.x + y * SIZE.h] = color;
        }

        for x in aabr.min.x.clamp(0, SIZE.w)..aabr.max.x.clamp(0, SIZE.w) {
            canvas[x + aabr.min.y * SIZE.w] = color;
            canvas[x + aabr.max.y * SIZE.w] = color;
        }
    }

    /// Draw a tiny circle.
    fn circle(&self, pos: Vec2<f32>, canvas: &mut [u32], color: u32) {
        self.point(pos, canvas, color);
        self.point(pos + Vec2::new(0.0, 1.0), canvas, color);
        self.point(pos + Vec2::new(1.0, 0.0), canvas, color);
        self.point(pos + Vec2::new(0.0, -1.0), canvas, color);
        self.point(pos + Vec2::new(-1.0, 0.0), canvas, color);
    }

    /// Draw a line.
    fn line(&self, mut start: Vec2<f32>, mut end: Vec2<f32>, canvas: &mut [u32], color: u32) {
        for line_2d::Coord { x, y } in line_2d::coords_between(
            line_2d::Coord::new(start.x as i32, start.y as i32),
            line_2d::Coord::new(end.x as i32, end.y as i32),
        ) {
            self.point(Vec2::new(x, y).as_(), canvas, color);
        }
    }

    /// Draw a single point.
    fn point(&self, pos: Vec2<f32>, canvas: &mut [u32], color: u32) {
        let pos = pos.as_::<usize>();

        if pos.x >= SIZE.w || pos.y >= SIZE.h {
            return;
        }

        canvas[pos.x + pos.y * SIZE.w] = color;
    }

    /// Draw a debug direction vector.
    fn direction(&self, pos: Vec2<f32>, dir: Vec2<f32>, canvas: &mut [u32]) {
        self.render_rotatable_sprite(Iso::new(pos, dir.y.atan2(dir.x)), "debug.vector", canvas)
    }

    /// Draw a vector with a magnitude.
    fn vector(&self, pos: Vec2<f32>, vec: Vec2<f32>, canvas: &mut [u32]) {
        let color = 0xFF00AAAA | ((vec.magnitude() * 20.0).clamp(0.0, 0xFF as f32) as u32) << 16;

        self.line(pos, pos + (vec * 4.0).as_(), canvas, color);
        self.circle(pos + vec.as_(), canvas, color);
    }

    /// Draw a physics object.
    fn physics_object(&self, iso: Iso, is_sleeping: bool, sprite_path: &str, canvas: &mut [u32]) {
        // Draw the sprite
        self.render_rotatable_sprite(iso, sprite_path, canvas);

        let object = crate::asset::<ObjectSettings>(sprite_path);
        let shape = object.shape();

        if !is_sleeping && crate::settings().debug.draw_physics_vertices {
            let vertices = shape.vertices(iso);

            // Draw all vertices
            vertices
                .iter()
                .for_each(|vertex| self.circle(vertex.as_(), canvas, 0xFF00FF00));

            // Draw a line between each vertex and the next
            let first = vertices[0];
            vertices
                .into_iter()
                .chain(std::iter::once(first))
                .reduce(|prev, cur| {
                    self.line(prev.as_(), cur.as_(), canvas, 0xFF00FF00);
                    cur
                });
        }
    }

    /// Draw a collision response.
    fn collision_response(&self, response: &CollisionResponse, a: Iso, b: Iso, canvas: &mut [u32]) {
        if !crate::settings().debug.draw_physics_contacts {
            return;
        }

        self.vector(a.pos.as_(), response.normal * response.penetration, canvas);
        self.vector(b.pos.as_(), -response.normal * response.penetration, canvas);

        self.circle(
            a.translate(response.local_contact_1).as_(),
            canvas,
            0xFFFF0000,
        );
        self.circle(
            b.translate(response.local_contact_2).as_(),
            canvas,
            0xFFFFFF00,
        );
    }
}

/// Debug settings loaded from a file so it's easier to change them with hot-reloading.
#[derive(Deserialize)]
pub struct DebugSettings {
    /// Which section to start in when pressing space.
    pub start_screen: u8,
    /// Whether to draw physics vertices.
    pub draw_physics_vertices: bool,
    /// Whether to draw physics contact points.
    pub draw_physics_contacts: bool,
}
