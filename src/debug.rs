use serde::Deserialize;
use vek::{Aabr, Extent2, Vec2};

use crate::{
    camera::Camera,
    input::Input,
    math::Rotation,
    object::ObjectSettings,
    physics::{
        collision::{shape::Rectangle, CollisionResponse},
        rigidbody::{RigidBody, RigidBodyIndex},
        Physics,
    },
    SIZE,
};

/// Physics grid step size.
const PHYSICS_GRID_STEP: u16 = 10;

/// Asset paths.
const SPEAR: &str = "projectile.spear-1";
const CRATE: &str = "object.crate-1";

/// Draw things for debugging purposes.
pub struct DebugDraw {
    /// What debug info to show, zero is show nothing.
    screen: u8,
    /// Mouse position.
    mouse: Vec2<i32>,
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
        self.mouse = input.mouse_pos;

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
            Vec2::new(20, 30),
            canvas,
        );

        if self.screen == 1 || self.screen == 2 {
            self.text(
                &format!(
                    "Rigidbodies: {}\nCollisions: {}",
                    self.rigidbodies.len(),
                    self.collisions.len()
                ),
                Vec2::new(SIZE.w as i32 - 100, 10),
                canvas,
            );

            // Draw physics sprites
            for (rigidbody, sprite_path) in self.rigidbodies.iter() {
                let rigidbody = self.physics.rigidbody(*rigidbody);

                self.physics_object(
                    rigidbody.position().as_(),
                    rigidbody.rotation(),
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
                self.collision_response(
                    response,
                    a.position().as_(),
                    a.rotation(),
                    b.position().as_(),
                    b.rotation(),
                    canvas,
                );
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
                    Vec2::new(50, 50 + index as i32 * 50),
                    asset,
                    canvas,
                );
            }
        } else if self.screen == 4 {
            // Draw collision between rotated rectangles
            let object = crate::asset::<ObjectSettings>(CRATE);
            let shape = object.shape();

            let mouse_rot = -23f32;

            // Draw the box
            self.physics_object(
                self.mouse.as_(),
                mouse_rot.to_radians(),
                false,
                CRATE,
                canvas,
            );

            for (index, rot) in [0, 90, 45, 23].into_iter().enumerate() {
                let pos = Vec2::new(
                    SIZE.w as i32 / 2 - 60 + index as i32 * 30,
                    SIZE.h as i32 / 2,
                );
                let rot = (rot as f32).to_radians();

                // Draw the box
                self.physics_object(pos.as_(), rot, false, CRATE, canvas);

                // Draw the collision information
                for response in shape.collides(
                    pos.as_(),
                    Rotation::from_degrees(rot),
                    &shape,
                    self.mouse.as_(),
                    Rotation::from_degrees(mouse_rot),
                ) {
                    self.collision_response(
                        &response,
                        pos,
                        rot,
                        self.mouse,
                        mouse_rot.to_radians(),
                        canvas,
                    );
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
                x += 30.0;
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

        // Create a nice pyramid
        self.rigidbodies = [
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
                self.physics.add_rigidbody(
                    object.rigidbody(
                        (Vec2::new(*x, *y)
                            + Vec2::new(SIZE.w as i32 / 2, (SIZE.h - SIZE.h / 4) as i32))
                        .as_(),
                    ),
                ),
                CRATE,
            )
        })
        .collect();

        // Don't let them fall through the ground
        self.physics.add_rigidbody(RigidBody::new_fixed(
            Vec2::new(SIZE.w as f32 / 2.0, SIZE.h as f32),
            Rectangle::new(Extent2::new(SIZE.w as f32, SIZE.h as f32 / 2.0)),
        ));
    }

    /// Draw a rotatable sprite pointing towards the mouse.
    fn render_rotatable_sprite(
        &self,
        pos: Vec2<i32>,
        rotation: f32,
        sprite_path: &str,
        canvas: &mut [u32],
    ) {
        let sprite = crate::rotatable_sprite(sprite_path);

        sprite.render(rotation, canvas, &Camera::default(), pos);
        self.text(
            &format!("{}", rotation.to_degrees().round()),
            pos + Vec2::new(0, 10),
            canvas,
        );
    }

    /// Draw a rotatable sprite towards the mouse.
    fn render_rotatable_to_mouse_sprite(
        &self,
        pos: Vec2<i32>,
        sprite_path: &str,
        canvas: &mut [u32],
    ) {
        // Draw rotating sprites
        let delta: Vec2<f32> = (self.mouse - pos).numcast().unwrap_or_default();
        let rot = delta.y.atan2(delta.x);
        self.render_rotatable_sprite(pos, rot, sprite_path, canvas);
    }

    /// Render text.
    fn text(&self, text: &str, pos: Vec2<i32>, canvas: &mut [u32]) {
        crate::font("font.debug").render(canvas, text, pos.x, pos.y);
    }

    /// Draw a bounding rectangle.
    fn aabr(&self, aabr: Aabr<f32>, canvas: &mut [u32], color: u32) {
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
    fn circle(&self, pos: Vec2<i32>, canvas: &mut [u32], color: u32) {
        self.point(pos, canvas, color);
        self.point(pos + Vec2::new(0, 1), canvas, color);
        self.point(pos + Vec2::new(1, 0), canvas, color);
        self.point(pos + Vec2::new(0, -1), canvas, color);
        self.point(pos + Vec2::new(-1, 0), canvas, color);
    }

    /// Draw a line.
    fn line(&self, mut start: Vec2<i32>, mut end: Vec2<i32>, canvas: &mut [u32], color: u32) {
        for line_2d::Coord { x, y } in line_2d::coords_between(
            line_2d::Coord::new(start.x, start.y),
            line_2d::Coord::new(end.x, end.y),
        ) {
            self.point(Vec2::new(x, y), canvas, color);
        }
    }

    /// Draw a single point.
    fn point(&self, pos: Vec2<i32>, canvas: &mut [u32], color: u32) {
        let pos = pos.as_::<usize>();

        if pos.x >= SIZE.w || pos.y >= SIZE.h {
            return;
        }

        canvas[pos.x + pos.y * SIZE.w] = color;
    }

    /// Draw a debug direction vector.
    fn direction(&self, pos: Vec2<i32>, dir: Vec2<f32>, canvas: &mut [u32]) {
        self.render_rotatable_sprite(pos, dir.y.atan2(dir.x), "debug.vector", canvas)
    }

    /// Draw a vector with a magnitude.
    fn vector(&self, pos: Vec2<i32>, vec: Vec2<f32>, canvas: &mut [u32]) {
        let color = 0xFF00AAAA | ((vec.magnitude() * 20.0).clamp(0.0, 0xFF as f32) as u32) << 16;

        self.line(pos, pos + (vec * 4.0).as_(), canvas, color);
        self.circle(pos + vec.as_(), canvas, color);
    }

    /// Draw a physics object.
    fn physics_object(
        &self,
        pos: Vec2<f32>,
        rot: f32,
        is_sleeping: bool,
        sprite_path: &str,
        canvas: &mut [u32],
    ) {
        // Draw the sprite
        self.render_rotatable_sprite(pos.as_(), rot, sprite_path, canvas);

        let object = crate::asset::<ObjectSettings>(sprite_path);
        let shape = object.shape();

        if !is_sleeping && crate::settings().debug.draw_physics_vertices {
            let vertices = shape.vertices(pos, Rotation::from_radians(rot));

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
    fn collision_response(
        &self,
        response: &CollisionResponse,
        pos_a: Vec2<i32>,
        rot_a: f32,
        pos_b: Vec2<i32>,
        rot_b: f32,
        canvas: &mut [u32],
    ) {
        if !crate::settings().debug.draw_physics_contacts {
            return;
        }

        self.vector(pos_a.as_(), response.normal * response.penetration, canvas);
        self.vector(pos_b.as_(), -response.normal * response.penetration, canvas);

        self.circle(
            pos_a.as_() + response.local_contact_1.rotated_z(rot_a).as_(),
            canvas,
            0xFFFF0000,
        );
        self.circle(
            pos_b.as_() + response.local_contact_2.rotated_z(rot_b).as_(),
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
