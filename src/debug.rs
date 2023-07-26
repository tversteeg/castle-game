use vek::Vec2;

use crate::{
    assets::Assets,
    camera::Camera,
    input::Input,
    physics::{
        rigidbody::{RigidBody, RigidBodyIndex},
        Simulator,
    },
};

/// Draw things for debugging purposes.
pub struct DebugDraw {
    /// Previous mouse state.
    previous_space_pressed: bool,
    /// Whether to show the debug info.
    active: bool,
    /// Mouse position.
    mouse: Vec2<i32>,
    /// Physics engine debug.
    physics: Simulator,
    /// Physics rigidbodies.
    rigidbodies: [RigidBodyIndex; 5],
}

impl DebugDraw {
    /// Setup with default.
    pub fn new(assets: &Assets) -> Self {
        let active = false;
        let previous_space_pressed = false;
        let mouse = Vec2::zero();
        let mut physics = Simulator::new();

        // Create some test rigidbodies
        let rigidbodies = [
            physics.add_rigidbody(RigidBody::new((0.0, 0.0).into(), 1.0, assets)),
            physics.add_rigidbody(RigidBody::new((0.0, 0.0).into(), 1.0, assets)),
            physics.add_rigidbody(RigidBody::new((0.0, 0.0).into(), 1.0, assets)),
            physics.add_rigidbody(RigidBody::new((0.0, 0.0).into(), 1.0, assets)),
            physics.add_rigidbody(RigidBody::new((0.0, 0.0).into(), 2.0, assets)),
        ];
        for i in 1..rigidbodies.len() {
            physics.add_distance_constraint([rigidbodies[i - 1], rigidbodies[i]], 30.0, 0.0001);
        }

        Self {
            previous_space_pressed,
            active,
            mouse,
            physics,
            rigidbodies,
        }
    }

    /// Update the debug state.
    pub fn update(&mut self, input: &Input, dt: f64, assets: &Assets) {
        // When space is released
        if !input.space_pressed && self.previous_space_pressed {
            self.active = !self.active;

            // Reset the rigidbodies
            for (index, rigidbody) in self.rigidbodies.iter().enumerate() {
                self.physics
                    .set_position(*rigidbody, Vec2::new(10.0 + index as f64 * 10.0, 10.0))
            }
        }

        self.previous_space_pressed = input.space_pressed;

        if !self.active {
            return;
        }

        // Store the mouse state
        self.mouse = input.mouse_pos;

        // Apply gravity
        self.physics
            .apply_global_force(Vec2::new(0.0, assets.settings().projectile_gravity * dt));

        // Make the first rigidbody follow the mouse
        self.physics.set_position(
            self.rigidbodies[0],
            self.mouse.numcast().unwrap_or_default(),
        );

        // Update the physics.
        self.physics.step(dt, assets);
    }

    /// Draw things for debugging purposes.
    pub fn render(&self, canvas: &mut [u32], assets: &Assets) {
        if !self.active {
            return;
        }

        // Draw rotating sprites
        let pos = Vec2::new(100, 100);
        let delta: Vec2<f64> = (self.mouse - pos).numcast().unwrap_or_default();
        let rot = delta.y.atan2(delta.x);
        self.render_rotatable_sprite(pos, rot, "projectile.spear-1", canvas, assets);

        // Draw physics sprites
        for rigidbody in self.rigidbodies {
            let pos = self.physics.position(rigidbody);
            let rot = self.physics.rotation(rigidbody);

            self.render_rotatable_sprite(
                pos.numcast().unwrap_or_default(),
                rot,
                "projectile.spear-1",
                canvas,
                assets,
            );
        }
    }

    /// Draw a rotatable sprite pointing towards the mouse.
    pub fn render_rotatable_sprite(
        &self,
        pos: Vec2<i32>,
        rotation: f64,
        sprite_path: &str,
        canvas: &mut [u32],
        assets: &Assets,
    ) {
        let sprite = assets.rotatable_sprite(sprite_path);

        sprite.render(rotation, canvas, &Camera::default(), pos);
        assets.font("font.torus-sans").render(
            canvas,
            &format!("{}", rotation.to_degrees().round()),
            pos.x,
            pos.y + 20,
        );
    }
}
