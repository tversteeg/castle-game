use serde::Deserialize;
use vek::{Aabr, Vec2};

use crate::{
    camera::Camera,
    game::PhysicsEngine,
    input::Input,
    math::{Iso, Rotation},
    object::ObjectSettings,
    physics::{collision::CollisionResponse, rigidbody::RigidBody},
    projectile::Projectile,
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
    /// Show the calculated rotsprite rotations with the mouse pointer.
    SpriteRotations,
    /// Draw static bodies with collision information.
    Collisions,
}

impl DebugScreen {
    /// Title rendered on screen.
    pub fn title(&self) -> &'static str {
        match self {
            DebugScreen::Empty => "",
            DebugScreen::SpawnProjectiles => "Spawn Projectiles on Click",
            DebugScreen::SpriteRotations => "Sprite Rotation Test",
            DebugScreen::Collisions => "Collision Detection Test",
        }
    }
}

impl DebugScreen {
    /// Go to the next screen.
    pub fn next(&self) -> Self {
        match self {
            Self::Empty => Self::SpawnProjectiles,
            Self::SpawnProjectiles => Self::SpriteRotations,
            Self::SpriteRotations => Self::Collisions,
            Self::Collisions => Self::Empty,
        }
    }
}

/// Draw things for debugging purposes.
pub struct DebugDraw {
    /// What debug info to show.
    screen: DebugScreen,
    /// Whether to draw the collision grid.
    ///
    /// Number signifies which grid level to draw at least.
    show_grid: i8,
    /// Whether to draw the rotation vectors.
    show_rotations: bool,
    /// Whether to draw collision outlines.
    show_colliders: bool,
    /// Mouse position.
    mouse: Vec2<f64>,
}

impl DebugDraw {
    /// Setup with default.
    pub fn new() -> Self {
        let mouse = Vec2::zero();
        let screen = crate::settings().debug.start_screen;
        let show_grid = -1;
        let show_rotations = false;
        let show_colliders = false;

        Self {
            screen,
            mouse,
            show_grid,
            show_rotations,
            show_colliders,
        }
    }

    /// Update the debug state.
    pub fn update(
        &mut self,
        input: &Input,
        physics: &mut PhysicsEngine,
        projectiles: &mut Vec<Projectile>,
        camera: &Camera,
        _dt: f64,
    ) {
        puffin::profile_function!();

        // When space is released
        if input.space.is_released() {
            self.screen = self.screen.next();
        }
        if input.r.is_released() {
            self.show_rotations = !self.show_rotations;
        }
        if input.c.is_released() {
            self.show_colliders = !self.show_colliders;
        }
        if input.g.is_released() {
            self.show_grid -= 1;
            if self.show_grid == -2 {
                self.show_grid = 3;
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

        self.mouse = input.mouse_pos.as_();
    }

    /// Draw things for debugging purposes.
    pub fn render(&self, physics: &mut PhysicsEngine, camera: &Camera, canvas: &mut [u32]) {
        puffin::profile_function!();

        // Draw which screen we are on
        self.render_text(self.screen.title(), Vec2::new(20.0, 30.0), canvas);

        // Draw how many rigidbodies there are
        self.render_text(
            &format!("Rigidbodies: {}", physics.rigidbody_map().len()),
            Vec2::new(SIZE.w as f64 - 100.0, 10.0),
            canvas,
        );

        // Draw vertices and lines
        if self.show_colliders {
            physics
                .rigidbody_map()
                .iter()
                .for_each(|(_, rigidbody)| self.render_collider(rigidbody, camera, canvas));

            // Draw attachment positions
            for (a, b) in physics.debug_info_constraints() {
                self.render_circle(camera.translate(a).as_(), canvas, 0xFFFF0000);
                self.render_circle(camera.translate(b).as_(), canvas, 0xFFFF00FF);
            }
        }

        if self.show_rotations {
            // Draw direction vectors for each rigidbody
            physics.rigidbody_map().iter().for_each(|(_, rigidbody)| {
                if rigidbody.is_active() {
                    self.render_direction(
                        camera.translate(rigidbody.position()),
                        rigidbody.direction(),
                        canvas,
                    )
                }
            });
        }

        if self.show_grid >= 0 {
            let (width, step, grid) = physics.broad_phase_grid(0.0);

            for (index, bucket_size) in grid.iter().enumerate() {
                if *bucket_size < self.show_grid as u8 {
                    continue;
                }

                let x = (index % width) as f64 * step;
                let y = (index / width) as f64 * step;

                self.render_text(
                    &format!("{bucket_size}"),
                    camera.translate(Vec2::new(x, y)),
                    canvas,
                );
            }
        }

        match self.screen {
            DebugScreen::SpriteRotations => {
                // Draw rotating sprites
                for (index, asset) in [SPEAR, CRATE].iter().enumerate() {
                    self.render_rotatable_to_mouse_sprite(
                        Vec2::new(50.0, 50.0 + index as f64 * 50.0),
                        asset,
                        canvas,
                    );
                }
            }
            DebugScreen::Collisions => {
                // Draw collision between rotated rectangles
                let object = crate::asset::<ObjectSettings>(CRATE);
                let shape = object.shape();

                let mouse_iso = Iso::new(self.mouse.as_(), Rotation::from_degrees(23f64));

                // Detect collisions with the heightmap
                let level_object = crate::asset::<ObjectSettings>(LEVEL);
                let level_pos = Vec2::new(0.0, 100.0);
                let level_iso = Iso::from_pos(level_pos);

                self.render_rotatable_sprite(level_iso, LEVEL, canvas);

                self.render_rotatable_sprite(mouse_iso, CRATE, canvas);

                // Draw the collision information
                for response in level_object.shape().collides(level_iso, &shape, mouse_iso) {
                    self.render_collision_response(&response, level_iso, mouse_iso, canvas);
                }

                for (index, rot) in [0, 90, 45, 23].into_iter().enumerate() {
                    let pos = Vec2::new(
                        SIZE.w as f64 / 2.0 - 60.0 + index as f64 * 30.0,
                        SIZE.h as f64 / 2.0,
                    );
                    let rot = Rotation::from_degrees(rot as f64);
                    let iso = Iso::new(pos, rot);

                    self.render_rotatable_sprite(iso, CRATE, canvas);

                    // Draw the collision information
                    for response in shape.collides(iso, &shape, mouse_iso) {
                        self.render_collision_response(&response, iso, mouse_iso, canvas);
                    }
                }
            }
            DebugScreen::SpawnProjectiles | DebugScreen::Empty => (),
        }
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

    /// Draw a bounding rectangle.
    fn render_aabr(&self, aabr: Aabr<f64>, canvas: &mut [u32], color: u32) {
        if aabr.max.x >= SIZE.w as f64 || aabr.max.y >= SIZE.h as f64 {
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
    fn render_circle(&self, pos: Vec2<f64>, canvas: &mut [u32], color: u32) {
        self.render_point(pos, canvas, color);
        self.render_point(pos + Vec2::new(0.0, 1.0), canvas, color);
        self.render_point(pos + Vec2::new(1.0, 0.0), canvas, color);
        self.render_point(pos + Vec2::new(0.0, -1.0), canvas, color);
        self.render_point(pos + Vec2::new(-1.0, 0.0), canvas, color);
    }

    /// Draw a line.
    fn render_line(&self, start: Vec2<f64>, end: Vec2<f64>, canvas: &mut [u32], color: u32) {
        for line_2d::Coord { x, y } in line_2d::coords_between(
            line_2d::Coord::new(start.x as i32, start.y as i32),
            line_2d::Coord::new(end.x as i32, end.y as i32),
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
    fn render_collider(&self, rigidbody: &RigidBody, camera: &Camera, canvas: &mut [u32]) {
        let vertices = rigidbody
            .vertices()
            .into_iter()
            .map(|vert| camera.translate(vert))
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
