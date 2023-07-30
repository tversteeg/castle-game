use vek::{Aabr, Extent2, Vec2};

use crate::{
    assets::Assets,
    camera::Camera,
    input::Input,
    math::Rotation,
    physics::{
        collision::{sat::NarrowCollision, shape::Rectangle},
        rigidbody::{RigidBody, RigidBodyIndex},
        Simulator,
    },
    SIZE,
};

/// Physics grid step size.
const PHYSICS_GRID_STEP: u16 = 10;

/// Draw things for debugging purposes.
pub struct DebugDraw {
    /// Previous keyboard state.
    previous_space_pressed: bool,
    /// What debug info to show, zero is show nothing.
    screen: u8,
    /// Mouse position.
    mouse: Vec2<i32>,
    /// Physics engine debug.
    ///
    /// All physics happen within the screen space.
    physics: Simulator<
        { SIZE.w as u16 },
        { SIZE.h as u16 },
        PHYSICS_GRID_STEP,
        4,
        { (SIZE.w / PHYSICS_GRID_STEP as usize) * (SIZE.h / PHYSICS_GRID_STEP as usize) },
    >,
    /// Physics rigidbodies.
    rigidbodies: Vec<RigidBodyIndex>,
    /// Rigidbodies with collisions.
    rigidbodies_with_collisions: Vec<RigidBodyIndex>,
}

impl DebugDraw {
    /// Setup with default.
    pub fn new() -> Self {
        let previous_space_pressed = false;
        let mouse = Vec2::zero();
        let physics = Simulator::new();
        let screen = 0;
        let rigidbodies = Vec::new();
        let rigidbodies_with_collisions = Vec::new();

        Self {
            screen,
            previous_space_pressed,
            mouse,
            physics,
            rigidbodies,
            rigidbodies_with_collisions,
        }
    }

    /// Update the debug state.
    pub fn update(&mut self, input: &Input, dt: f32, assets: &Assets) {
        // When space is released
        if !input.space_pressed && self.previous_space_pressed {
            self.screen += 1;

            match self.screen {
                1 => self.setup_physics_scene_1(assets),
                2 => self.setup_physics_scene_2(assets),
                3 | 4 => (),
                _ => self.screen = 0,
            }
        }
        self.previous_space_pressed = input.space_pressed;

        if self.screen == 0 {
            return;
        }

        // Store the mouse state
        self.mouse = input.mouse_pos;

        // Make the first rigidbody follow the mouse
        self.physics.set_position(
            self.rigidbodies[0],
            self.mouse.numcast().unwrap_or_default(),
        );

        // Update the physics.
        self.physics.step(dt, assets);

        // Keep track of all collisions in an ugly way
        self.rigidbodies_with_collisions = self
            .physics
            .colliding_rigid_bodies()
            .into_iter()
            .flat_map(|(a, b, _)| std::iter::once(a).chain(std::iter::once(b)))
            .collect();
    }

    /// Draw things for debugging purposes.
    pub fn render(&self, canvas: &mut [u32], assets: &Assets) {
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
            assets,
        );

        if self.screen == 1 || self.screen == 2 {
            // Draw physics sprites
            for rigidbody in self.rigidbodies.iter() {
                let rigidbody = self.physics.rigidbody(*rigidbody);

                self.render_rotatable_sprite(
                    rigidbody.position().as_(),
                    rigidbody.rotation(),
                    if self.screen == 1 {
                        "projectile.spear-1"
                    } else {
                        "object.crate-1"
                    },
                    canvas,
                    assets,
                );

                self.text(
                    &format!("{rigidbody}"),
                    rigidbody.position().as_(),
                    canvas,
                    assets,
                );

                // Draw AABR
                //self.aabr(self.physics.aabr(*rigidbody), canvas, 0xFF00FF00);
            }

            // Draw collisions
            for rigidbody in self.rigidbodies_with_collisions.iter() {
                // Draw AABR
                self.aabr(self.physics.aabr(*rigidbody), canvas, 0xFFFF0000);
            }

            // Draw attachment positions
            for (a, b) in self.physics.debug_info_constraints() {
                self.circle(a.as_(), canvas, 0xFFFF0000);
                self.circle(b.as_(), canvas, 0xFFFF00FF);
            }
        } else if self.screen == 3 {
            // Draw rotating sprites
            for (index, asset) in ["projectile.spear-1", "object.crate-1"].iter().enumerate() {
                self.render_rotatable_to_mouse_sprite(
                    Vec2::new(50, 50 + index as i32 * 50),
                    asset,
                    canvas,
                    assets,
                );
            }
        } else if self.screen == 4 {
            let sprite_path = "object.crate-1";
            // Draw two rectangles colliding
            let sprite = assets.sprite(sprite_path);
            let shape = Rectangle::new(Extent2::new(sprite.width() as f32, sprite.height() as f32));

            let (pos_a, pos_b) = (Vec2::new(SIZE.w as i32 / 2, SIZE.h as i32 / 2), self.mouse);
            let (rot_a, rot_b) = (45f32.to_radians(), 90f32.to_radians());
            self.render_rotatable_sprite(pos_a, rot_a, sprite_path, canvas, assets);
            self.render_rotatable_sprite(pos_b, rot_b, sprite_path, canvas, assets);

            if let Some(response) = shape.collide_rectangle(
                pos_a.as_(),
                Rotation::from_radians(rot_a),
                shape,
                pos_b.as_(),
                Rotation::from_radians(rot_b),
            ) {
                self.vector(response.contact.as_(), response.mtv, canvas, assets);
                //self.circle(response.contact.as_(), canvas, 0xFFFF0000);
            }
        }
    }

    /// Setup a new physics scene with a rope structure.
    fn setup_physics_scene_1(&mut self, assets: &Assets) {
        self.physics = Simulator::new();

        // Shape is based on the size of the image
        let sprite = assets.sprite("projectile.spear-1");
        let shape = Rectangle::new(Extent2::new(sprite.width() as f32, sprite.height() as f32));

        // Create some test rigidbodies
        let mut x = 50.0;
        self.rigidbodies = [(); 5]
            .iter()
            .enumerate()
            .map(|(i, _)| {
                x += 30.0;
                self.physics.add_rigidbody(RigidBody::new(
                    Vec2::new(SIZE.w as f32 / 2.0 + x, SIZE.h as f32 / 2.0),
                    if i == 0 { std::f32::INFINITY } else { 1.0 },
                    shape,
                    assets,
                ))
            })
            .collect();

        // Connect each rigidbody with the previous to create a rope
        for i in 1..self.rigidbodies.len() {
            // Offset the attachments to prevent collisions
            self.physics.add_distance_constraint(
                self.rigidbodies[i - 1],
                Vec2::new(-shape.half_width() - 10.0, 0.0),
                self.rigidbodies[i],
                Vec2::new(shape.half_width() + 10.0, 0.0),
                5.0,
                0.0001,
            );
        }
    }

    /// Setup a new physics scene with boxes.
    fn setup_physics_scene_2(&mut self, assets: &Assets) {
        self.physics = Simulator::new();

        // Shape is based on the size of the image
        let sprite = assets.sprite("object.crate-1");
        let shape = Rectangle::new(Extent2::new(sprite.width() as f32, sprite.height() as f32));

        // Create a nice pyramid
        self.rigidbodies = [(0, -20), (-10, 0), (10, 0), (-20, 20), (0, 20), (20, 20)]
            .iter()
            .map(|(x, y)| {
                self.physics.add_rigidbody(RigidBody::new(
                    (Vec2::new(*x, *y) + Vec2::new(SIZE.w as i32 / 2, SIZE.h as i32 / 2)).as_(),
                    1.0,
                    shape,
                    assets,
                ))
            })
            .collect();

        // Don't let them fall through the ground
        self.rigidbodies.iter().for_each(|rigidbody| {
            self.physics
                .add_ground_constraint(*rigidbody, SIZE.h as f32 / 2.0 + 50.0);
        })
    }

    /// Draw a rotatable sprite pointing towards the mouse.
    fn render_rotatable_sprite(
        &self,
        pos: Vec2<i32>,
        rotation: f32,
        sprite_path: &str,
        canvas: &mut [u32],
        assets: &Assets,
    ) {
        let sprite = assets.rotatable_sprite(sprite_path);

        sprite.render(rotation, canvas, &Camera::default(), pos);
        self.text(
            &format!("{}", rotation.to_degrees().round()),
            pos + Vec2::new(0, 10),
            canvas,
            assets,
        );
    }

    /// Draw a rotatable sprite towards the mouse.
    fn render_rotatable_to_mouse_sprite(
        &self,
        pos: Vec2<i32>,
        sprite_path: &str,
        canvas: &mut [u32],
        assets: &Assets,
    ) {
        // Draw rotating sprites
        let delta: Vec2<f32> = (self.mouse - pos).numcast().unwrap_or_default();
        let rot = delta.y.atan2(delta.x);
        self.render_rotatable_sprite(pos, rot, sprite_path, canvas, assets);
    }

    /// Render text.
    fn text(&self, text: &str, pos: Vec2<i32>, canvas: &mut [u32], assets: &Assets) {
        assets
            .font("font.torus-sans")
            .render(canvas, text, pos.x, pos.y);
    }

    /// Draw a bounding rectangle.
    fn aabr(&self, aabr: Aabr<f32>, canvas: &mut [u32], color: u32) {
        let aabr: Aabr<usize> = aabr.as_();

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

    /// Draw a single point.
    fn point(&self, pos: Vec2<i32>, canvas: &mut [u32], color: u32) {
        let pos = pos.as_::<usize>();

        if pos.x >= SIZE.w || pos.y >= SIZE.h {
            return;
        }

        canvas[pos.x + pos.y * SIZE.w] = color;
    }

    /// Draw a debug direction vector.
    fn vector(&self, pos: Vec2<i32>, dir: Vec2<f32>, canvas: &mut [u32], assets: &Assets) {
        self.render_rotatable_sprite(pos, dir.y.atan2(dir.x), "debug.vector", canvas, assets)
    }
}
