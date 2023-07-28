use vek::{Aabr, Extent2, Vec2};

use crate::{
    assets::Assets,
    camera::Camera,
    input::Input,
    physics::{
        collision::shape::Rectangle,
        rigidbody::{RigidBody, RigidBodyIndex},
        Simulator,
    },
    SIZE,
};

/// Draw things for debugging purposes.
pub struct DebugDraw {
    /// Previous keyboard state.
    previous_space_pressed: bool,
    /// What debug info to show, zero is show nothing.
    screen: u8,
    /// Mouse position.
    mouse: Vec2<i32>,
    /// Physics engine debug.
    physics: Simulator,
    /// Physics rigidbodies.
    rigidbodies: Vec<RigidBodyIndex>,
}

impl DebugDraw {
    /// Setup with default.
    pub fn new() -> Self {
        let previous_space_pressed = false;
        let mouse = Vec2::zero();
        let physics = Simulator::new();
        let screen = 0;
        let rigidbodies = Vec::new();

        Self {
            screen,
            previous_space_pressed,
            mouse,
            physics,
            rigidbodies,
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
                3 => (),
                _ => self.screen = 0,
            }
        }
        self.previous_space_pressed = input.space_pressed;

        if self.screen == 0 {
            return;
        }

        // Store the mouse state
        self.mouse = input.mouse_pos;

        // Apply gravity
        self.physics
            .apply_global_force(Vec2::new(0.0, assets.settings().projectile_gravity * dt));

        if self.screen == 1 {
            // Make the first rigidbody follow the mouse
            self.physics.set_position(
                self.rigidbodies[0],
                self.mouse.numcast().unwrap_or_default(),
            );

            self.physics
                .apply_rotational_force(self.rigidbodies[0], 0.01);
        }

        // Update the physics.
        self.physics.step(dt, assets);
    }

    /// Draw things for debugging purposes.
    pub fn render(&self, canvas: &mut [u32], assets: &Assets) {
        if self.screen == 1 || self.screen == 2 {
            // Draw physics sprites
            for rigidbody in self.rigidbodies.iter() {
                let pos = self.physics.position(*rigidbody);
                let rot = self.physics.rotation(*rigidbody);

                self.render_rotatable_sprite(
                    pos.numcast().unwrap_or_default(),
                    rot,
                    if self.screen == 1 {
                        "projectile.spear-1"
                    } else {
                        "object.crate-1"
                    },
                    canvas,
                    assets,
                );

                // Draw AABR
                self.aabr(self.physics.aabr(*rigidbody), canvas, 0xFF00FF00);
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
            .map(|_| {
                x += 10.0;
                self.physics.add_rigidbody(RigidBody::new(
                    Vec2::new(SIZE.w as f32 / 2.0 + x, SIZE.h as f32 / 2.0),
                    1.0,
                    shape,
                    assets,
                ))
            })
            .collect();

        // Connect each rigidbody with the previous to create a rope
        for i in 1..self.rigidbodies.len() {
            self.physics.add_distance_constraint(
                [self.rigidbodies[i - 1], self.rigidbodies[i]],
                30.0,
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
        self.rigidbodies = [
            (60.0, 40.0),
            (50.0, 60.0),
            (80.0, 60.0),
            (40.0, 80.0),
            (60.0, 80.0),
            (90.0, 80.0),
        ]
        .iter()
        .map(|pos| {
            self.physics
                .add_rigidbody(RigidBody::new((*pos).into(), 1.0, shape, assets))
        })
        .collect();

        // Don't let them fall through the ground
        self.rigidbodies.iter().for_each(|rigidbody| {
            self.physics.add_ground_constraint(*rigidbody, 200.0);
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
}
