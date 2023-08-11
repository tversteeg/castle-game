use serde::Deserialize;
use vek::{Aabr, Extent2, Vec2};

use crate::{
    camera::Camera,
    game::PhysicsEngine,
    input::Input,
    math::{Iso, Rotation},
    object::ObjectSettings,
    physics::{
        collision::{shape::Shape, CollisionResponse},
        rigidbody::{RigidBody, RigidBodyIndex},
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
    #[default]
    Empty,
    SpawnProjectiles,
    RigidBodyDirections,
    SpriteRotations,
    Collisions,
}

impl DebugScreen {
    /// Title rendered on screen.
    pub fn title(&self) -> &'static str {
        match self {
            DebugScreen::Empty => "",
            DebugScreen::SpawnProjectiles => "Spawn Projectiles on Click",
            DebugScreen::RigidBodyDirections => "Rigidbody Directions",
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
            Self::SpawnProjectiles => Self::RigidBodyDirections,
            Self::RigidBodyDirections => Self::SpriteRotations,
            Self::SpriteRotations => Self::Collisions,
            Self::Collisions => Self::Empty,
        }
    }
}

/// Draw things for debugging purposes.
pub struct DebugDraw {
    /// What debug info to show.
    screen: DebugScreen,
    /// Mouse position.
    mouse: Vec2<f32>,
}

impl DebugDraw {
    /// Setup with default.
    pub fn new() -> Self {
        let mouse = Vec2::zero();
        let screen = crate::settings().debug.start_screen;

        Self { screen, mouse }
    }

    /// Update the debug state.
    pub fn update(
        &mut self,
        input: &Input,
        physics: &mut PhysicsEngine,
        projectiles: &mut Vec<Projectile>,
        camera: &Camera,
        dt: f32,
    ) {
        puffin::profile_function!();

        // When space is released
        if input.space.is_released() {
            self.screen = self.screen.next();
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
    pub fn render(&self, physics: &PhysicsEngine, camera: &Camera, canvas: &mut [u32]) {
        puffin::profile_function!();

        // Draw vertices and lines
        if crate::settings().debug.draw_physics_colliders {
            physics
                .rigidbody_map()
                .iter()
                .for_each(|(_, rigidbody)| self.render_collider(rigidbody, camera, canvas));
        }

        if crate::settings().debug.draw_physics_contacts {
            // Draw attachment positions
            for (a, b) in physics.debug_info_constraints() {
                self.render_circle(camera.translate(a).as_(), canvas, 0xFFFF0000);
                self.render_circle(camera.translate(b).as_(), canvas, 0xFFFF00FF);
            }
        }

        // Draw which screen we are on
        self.render_text(self.screen.title(), Vec2::new(20.0, 30.0), canvas);

        // Draw how many rigidbodies there are
        self.render_text(
            &format!("Rigidbodies: {}", physics.rigidbody_map().len()),
            Vec2::new(SIZE.w as f32 - 100.0, 10.0),
            canvas,
        );

        match self.screen {
            DebugScreen::SpriteRotations => {
                // Draw rotating sprites
                for (index, asset) in [SPEAR, CRATE].iter().enumerate() {
                    self.render_rotatable_to_mouse_sprite(
                        Vec2::new(50.0, 50.0 + index as f32 * 50.0),
                        asset,
                        canvas,
                    );
                }
            }
            DebugScreen::Collisions => {
                // Draw collision between rotated rectangles
                let object = crate::asset::<ObjectSettings>(CRATE);
                let shape = object.shape();

                let mouse_iso = Iso::new(self.mouse.as_(), Rotation::from_degrees(23f32));

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
                        SIZE.w as f32 / 2.0 - 60.0 + index as f32 * 30.0,
                        SIZE.h as f32 / 2.0,
                    );
                    let rot = Rotation::from_degrees(rot as f32);
                    let iso = Iso::new(pos, rot);

                    self.render_rotatable_sprite(iso, CRATE, canvas);

                    // Draw the collision information
                    for response in shape.collides(iso, &shape, mouse_iso) {
                        self.render_collision_response(&response, iso, mouse_iso, canvas);
                    }
                }
            }
            DebugScreen::RigidBodyDirections => {
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
    fn render_text(&self, text: &str, pos: Vec2<f32>, canvas: &mut [u32]) {
        crate::font("font.debug").render(text, pos, canvas);
    }

    /// Draw a debug direction vector.
    fn render_direction(&self, pos: Vec2<f32>, dir: Vec2<f32>, canvas: &mut [u32]) {
        self.render_rotatable_sprite(Iso::new(pos, dir.y.atan2(dir.x)), "debug.vector", canvas)
    }

    /// Draw a bounding rectangle.
    fn render_aabr(&self, aabr: Aabr<f32>, canvas: &mut [u32], color: u32) {
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
    fn render_circle(&self, pos: Vec2<f32>, canvas: &mut [u32], color: u32) {
        self.render_point(pos, canvas, color);
        self.render_point(pos + Vec2::new(0.0, 1.0), canvas, color);
        self.render_point(pos + Vec2::new(1.0, 0.0), canvas, color);
        self.render_point(pos + Vec2::new(0.0, -1.0), canvas, color);
        self.render_point(pos + Vec2::new(-1.0, 0.0), canvas, color);
    }

    /// Draw a line.
    fn render_line(
        &self,
        mut start: Vec2<f32>,
        mut end: Vec2<f32>,
        canvas: &mut [u32],
        color: u32,
    ) {
        for line_2d::Coord { x, y } in line_2d::coords_between(
            line_2d::Coord::new(start.x as i32, start.y as i32),
            line_2d::Coord::new(end.x as i32, end.y as i32),
        ) {
            self.render_point(Vec2::new(x, y).as_(), canvas, color);
        }
    }

    /// Draw a single point.
    fn render_point(&self, pos: Vec2<f32>, canvas: &mut [u32], color: u32) {
        let pos = pos.as_::<usize>();

        if pos.x >= SIZE.w || pos.y >= SIZE.h {
            return;
        }

        canvas[pos.x + pos.y * SIZE.w] = color;
    }

    /// Draw a vector with a magnitude.
    fn render_vector(&self, pos: Vec2<f32>, vec: Vec2<f32>, canvas: &mut [u32]) {
        let color = 0xFF00AAAA | ((vec.magnitude() * 20.0).clamp(0.0, 0xFF as f32) as u32) << 16;

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
    /// Whether to draw physics collider shapes.
    pub draw_physics_colliders: bool,
    /// Whether to draw physics contact points.
    pub draw_physics_contacts: bool,
}
