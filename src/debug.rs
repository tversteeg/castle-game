use line_drawing::Bresenham;
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::{
    camera::Camera,
    input::Input,
    math::{Iso, Rotation},
    object::ObjectSettings,
    physics::{
        collision::{shape::Shape, CollisionResponse},
        rigidbody::{RigidBodyBuilder, RigidBodyHandle},
        Physics,
    },
    projectile::Projectile,
    terrain::Terrain,
    SIZE,
};

/// Asset paths.
const LEVEL: &str = "level.grass-1";
const SPEAR: &str = "projectile.spear-1";
const CRATE: &str = "object.crate-1";

/// Different debug screens.
#[derive(Debug, Default, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugScreen {
    /// Show nothing.
    #[default]
    Empty,
    /// Spawn projectiles on click.
    SpawnProjectiles,
    /// Spawn cubes on click in a local engine which can be stepped through by pressing down or 's'.
    SpawnCubes,
    /// Show the calculated rotsprite rotations with the mouse pointer.
    SpriteRotations,
    /// Draw static bodies with collision information.
    Collisions,
    /// Separatable terrain sandbox.
    Terrain,
}

impl DebugScreen {
    /// Title rendered on screen.
    pub fn title(&self) -> &'static str {
        match self {
            DebugScreen::Empty => "",
            DebugScreen::SpawnProjectiles => "Spawn Projectiles on Click",
            DebugScreen::SpawnCubes => "Spawn Cubes on Click in Local Engine",
            DebugScreen::SpriteRotations => "Sprite Rotation Test",
            DebugScreen::Collisions => "Collision Detection Test",
            DebugScreen::Terrain => "Click to Remove Terrain Pixels",
        }
    }
}

impl DebugScreen {
    /// Go to the next screen.
    pub fn next(&self) -> Self {
        match self {
            Self::Empty => Self::SpawnProjectiles,
            Self::SpawnProjectiles => Self::SpawnCubes,
            Self::SpawnCubes => Self::SpriteRotations,
            Self::SpriteRotations => Self::Collisions,
            Self::Collisions => Self::Terrain,
            Self::Terrain => Self::Empty,
        }
    }
}

/// Draw things for debugging purposes.
pub struct DebugDraw {
    /// What debug info to show.
    screen: DebugScreen,
    /// Whether to draw the rotation vectors.
    show_rotations: bool,
    /// Whether to draw collision outlines.
    show_colliders: bool,
    /// Whether to draw collisions.
    show_collisions: bool,
    /// Mouse position.
    mouse: Vec2<f64>,
    /// Local physics engine for box test.
    physics: Physics,
    /// Local boxes.
    boxes: Vec<RigidBodyHandle>,
    /// Platform.
    platform: RigidBodyHandle,
}

impl DebugDraw {
    /// Setup with default.
    pub fn new() -> Self {
        let mouse = Vec2::zero();
        let screen = crate::settings().debug.start_screen;
        let show_rotations = false;
        let show_colliders = false;
        let show_collisions = false;
        let mut physics = Physics::new();
        let boxes = Vec::new();

        // Spawn a big platform
        let platform = RigidBodyBuilder::new_static(Vec2::new(SIZE.w / 2, SIZE.h - 100).as_())
            .with_collider(Shape::rectangle(Extent2::new(SIZE.w / 2, 50).as_()))
            .with_friction(0.7)
            .with_restitution(0.0)
            .spawn(&mut physics);

        Self {
            screen,
            mouse,
            show_rotations,
            show_colliders,
            show_collisions,
            physics,
            boxes,
            platform,
        }
    }

    /// Update the debug state.
    pub fn update(
        &mut self,
        input: &Input,
        physics: &mut Physics,
        projectiles: &mut Vec<Projectile>,
        terrain: &mut Terrain,
        camera: &Camera,
        dt: f64,
    ) {
        puffin::profile_function!();

        // When space is released
        if input.n.is_released() {
            self.screen = self.screen.next();
        }
        if input.r.is_released() {
            self.show_rotations = !self.show_rotations;
        }
        if input.c.is_released() {
            self.show_collisions = !self.show_collisions;
        }
        if input.o.is_released() {
            self.show_colliders = !self.show_colliders;
        }

        if self.screen == DebugScreen::SpawnCubes {
            if input.space.is_pressed() {
                self.physics.step(dt);
            }

            if input.left_mouse.is_released() {
                // Spawn a projectile at the mouse coordinates, camera doesn't apply to local physics engine
                let object = crate::asset::<ObjectSettings>(CRATE);
                self.boxes.push(
                    object
                        .rigidbody_builder(self.mouse)
                        .spawn(&mut self.physics),
                );
            }
        }

        if self.screen == DebugScreen::SpawnProjectiles && input.left_mouse.is_released() {
            // Spawn a projectile at the mouse coordinates
            projectiles.push(Projectile::new(
                camera.translate_from_screen(self.mouse),
                Vec2::zero(),
                physics,
            ));
        }

        if self.screen == DebugScreen::Terrain && input.left_mouse.is_pressed() {
            // Click to slice the terrain
            terrain.remove_circle(
                camera.translate_from_screen(input.mouse_pos.as_()),
                10.0,
                physics,
            );
        }

        self.mouse = input.mouse_pos.as_();
    }

    /// Draw things for debugging purposes.
    pub fn render(&mut self, physics: &mut Physics, camera: &Camera, canvas: &mut [u32]) {
        puffin::profile_function!();

        // Draw which screen we are on
        self.render_text(
            &format!(
                "{}\n\n[N] Next debug screen\n[C] Show collisions: {}\n[O] Show colliders: {}\n[R] Show rotations: {}\n[Space] Step through boxes example",
                self.screen.title(),
                self.show_collisions,
                self.show_colliders,
                self.show_rotations,
            ),
            Vec2::new(20.0, 30.0),
            canvas,
        );

        // Draw the physics information
        self.render_text(
            &format!("Rigidbodies: {}", physics.rigidbody_amount()),
            Vec2::new(SIZE.w as f64 - 100.0, 30.0),
            canvas,
        );

        self.render_colliders(physics, camera, canvas);
        self.render_collisions(physics, camera, canvas);

        match self.screen {
            // Draw rotating sprites
            DebugScreen::SpriteRotations => {
                for (index, asset) in [SPEAR, CRATE].iter().enumerate() {
                    self.render_rotatable_to_mouse_sprite(
                        Vec2::new(
                            SIZE.w as f64 / 2.0,
                            SIZE.h as f64 / 2.0 + index as f64 * 50.0,
                        ),
                        asset,
                        canvas,
                    );
                }
            }
            DebugScreen::SpawnCubes => {
                for rigidbody in self.boxes.iter() {
                    self.render_rotatable_sprite(rigidbody.iso(&self.physics), CRATE, canvas);
                }

                self.render_colliders(&self.physics, &Camera::default(), canvas);
                self.render_collisions(&self.physics, &Camera::default(), canvas);
            }
            DebugScreen::Collisions => {
                // Draw collision between rotated rectangles
                let object = crate::asset::<ObjectSettings>(SPEAR);
                let shape = object.shape();

                let mouse_iso = Iso::new(self.mouse.as_(), Rotation::from_degrees(-23f64));

                // Detect collisions with the heightmap
                let level_object = crate::asset::<ObjectSettings>(LEVEL);
                let level_pos = Vec2::new(0.0, 100.0);
                let level_iso = Iso::from_pos(level_pos);

                self.render_rotatable_sprite(level_iso, LEVEL, canvas);

                self.render_rotatable_sprite(mouse_iso, SPEAR, canvas);

                // Draw the collision information
                for response in level_object.shape().collides(level_iso, &shape, mouse_iso) {
                    self.render_collision_response(&response, level_iso, mouse_iso, canvas);
                }

                for (index, rot) in [0, 90, 45, 23, -23, -179, 179].into_iter().enumerate() {
                    let pos = Vec2::new(
                        SIZE.w as f64 / 2.0 - 60.0 + index as f64 * 30.0,
                        SIZE.h as f64 / 2.0,
                    );
                    let rot = Rotation::from_degrees(rot as f64);
                    let iso = Iso::new(pos, rot);

                    self.render_rotatable_sprite(iso, SPEAR, canvas);

                    // Draw the collision information
                    for response in shape.collides(iso, &shape, mouse_iso) {
                        self.render_collision_response(&response, iso, mouse_iso, canvas);
                    }
                }
            }
            DebugScreen::Terrain | DebugScreen::SpawnProjectiles | DebugScreen::Empty => (),
        }
    }

    /// Render collision information for a physics system.
    fn render_collisions(&self, physics: &Physics, camera: &Camera, canvas: &mut [u32]) {
        if !self.show_collisions {
            return;
        }

        // Draw attachment positions
        for (a, b, response) in physics.debug_info_constraints() {
            self.render_direction(a, response.normal, canvas);
            self.render_direction(b, -response.normal, canvas);
            self.render_circle(camera.translate(a).as_(), canvas, 0xFFFF0000);
            self.render_circle(camera.translate(b).as_(), canvas, 0xFFFF00FF);
        }
    }

    /// Render collider information for a physics system.
    fn render_colliders(&self, physics: &Physics, camera: &Camera, canvas: &mut [u32]) {
        if !self.show_colliders {
            return;
        }
        physics
            .debug_info_vertices()
            .into_iter()
            .for_each(|vertices| self.render_collider(&vertices, camera, canvas));
    }

    /// Draw a rotatable sprite pointing towards the mouse.
    fn render_rotatable_sprite(&self, iso: Iso, sprite_path: &str, canvas: &mut [u32]) {
        let sprite = crate::rotatable_sprite(sprite_path);

        sprite.render(iso, canvas, &Camera::default());
        self.render_text(
            &format!("{}", iso.rot.to_degrees().round()),
            iso.pos + Vec2::new(0.0, 10.0),
            canvas,
        );
    }

    /// Draw a rotatable sprite towards the mouse.
    fn render_rotatable_to_mouse_sprite(
        &self,
        pos: Vec2<f64>,
        sprite_path: &str,
        canvas: &mut [u32],
    ) {
        // Draw rotating sprites
        let delta: Vec2<f64> = (self.mouse - pos).numcast().unwrap_or_default();
        let rot = delta.y.atan2(delta.x);
        self.render_rotatable_sprite(Iso::new(pos, rot), sprite_path, canvas);
    }

    /// Render text.
    fn render_text(&self, text: &str, pos: Vec2<f64>, canvas: &mut [u32]) {
        crate::font("font.debug").render(text, pos, canvas);
    }

    /// Draw a debug direction vector.
    fn render_direction(&self, pos: Vec2<f64>, dir: Vec2<f64>, canvas: &mut [u32]) {
        self.render_rotatable_sprite(Iso::new(pos, dir.y.atan2(dir.x)), "debug.vector", canvas)
    }

    /// Draw a tiny circle.
    fn render_circle(&self, pos: Vec2<f64>, canvas: &mut [u32], color: u32) {
        self.render_point(pos, canvas, color);
        self.render_point(pos + Vec2::new(0.0, 1.0), canvas, color);
        self.render_point(pos + Vec2::new(1.0, 0.0), canvas, color);
        self.render_point(pos + Vec2::new(0.0, -1.0), canvas, color);
        self.render_point(pos + Vec2::new(-1.0, 0.0), canvas, color);
    }

    /// Draw a line.
    fn render_line(&self, start: Vec2<f64>, end: Vec2<f64>, canvas: &mut [u32], color: u32) {
        for (x, y) in Bresenham::new(
            (start.x as i32, start.y as i32),
            (end.x as i32, end.y as i32),
        ) {
            self.render_point(Vec2::new(x, y).as_(), canvas, color);
        }
    }

    /// Draw a single point.
    fn render_point(&self, pos: Vec2<f64>, canvas: &mut [u32], color: u32) {
        let pos = pos.as_::<usize>();

        if pos.x >= SIZE.w || pos.y >= SIZE.h {
            return;
        }

        canvas[pos.x + pos.y * SIZE.w] = color;
    }

    /// Draw a vector with a magnitude.
    fn render_vector(&self, pos: Vec2<f64>, vec: Vec2<f64>, canvas: &mut [u32]) {
        let color = 0xFF00AAAA | ((vec.magnitude() * 20.0).clamp(0.0, 0xFF as f64) as u32) << 16;

        self.render_line(pos, pos + (vec * 4.0).as_(), canvas, color);
        self.render_circle(pos + vec.as_(), canvas, color);
    }

    /// Draw collider shape.
    fn render_collider(&self, vertices: &[Vec2<f64>], camera: &Camera, canvas: &mut [u32]) {
        let vertices = vertices
            .iter()
            .map(|vert| camera.translate(*vert))
            .collect::<Vec<_>>();

        // Draw all vertices
        vertices
            .iter()
            .for_each(|vertex| self.render_circle(vertex.as_(), canvas, 0xFF00FF00));

        // Draw a line between each vertex and the next
        let first = vertices[0];
        vertices
            .into_iter()
            .chain(std::iter::once(first))
            .reduce(|prev, cur| {
                self.render_line(prev.as_(), cur.as_(), canvas, 0xFF00FF00);
                cur
            });
    }

    /// Draw a collision response.
    fn render_collision_response(
        &self,
        response: &CollisionResponse,
        a: Iso,
        b: Iso,
        canvas: &mut [u32],
    ) {
        if !crate::settings().debug.draw_physics_contacts {
            return;
        }

        self.render_vector(a.pos.as_(), response.normal * response.penetration, canvas);
        self.render_vector(b.pos.as_(), -response.normal * response.penetration, canvas);

        self.render_circle(
            a.translate(response.local_contact_1).as_(),
            canvas,
            0xFFFF0000,
        );
        self.render_circle(
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
    pub start_screen: DebugScreen,
    /// Whether to draw physics contact points.
    pub draw_physics_contacts: bool,
}
