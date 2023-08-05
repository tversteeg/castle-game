use arrayvec::ArrayVec;
use parry2d::{
    math::{Isometry, Point},
    na::{Isometry2, Translation, Vector2},
    query::{ContactManifold, DefaultQueryDispatcher, PersistentQueryDispatcher, TrackedContact},
    shape::{ConvexPolygon, Cuboid, PackedFeatureId, RoundShape, Shape, SharedShape},
};
use vek::{Aabr, Extent2, LineSegment2, Vec2};

use crate::math::Rotation;

use super::{CollisionResponse, NarrowCollision};

/// Distance at which the collisions will be detected before actually touching.
const PREDICTION_DISTANCE: f32 = 0.1;

/// Distance at which we count a collision as valid.
const MIN_PENETRATION_DISTANCE: f32 = 0.0001;

/// Orientable rectangle.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rectangle {
    /// Half size in each direction from the center point.
    pub half_size: Extent2<f32>,
}

impl Rectangle {
    /// Construct a new rectangle with a rotation of 0.
    pub fn new(size: Extent2<f32>) -> Self {
        let half_size = size / 2.0;

        Self { half_size }
    }

    /// Calculate the axis aligned bounding rectangle.
    pub fn aabr(&self, pos: Vec2<f32>, rot: f32) -> Aabr<f32> {
        let (sin, cos) = rot.sin_cos();
        let (abs_sin, abs_cos) = (sin.abs(), cos.abs());

        let width = self.half_size.w * abs_cos + self.half_size.h * abs_sin;
        let height = self.half_size.w * abs_sin + self.half_size.h * abs_cos;
        let size = Extent2::new(width, height);

        let min = pos - size;
        let max = pos + size;

        Aabr { min, max }
    }

    /// Calculate the 4 corner points.
    pub fn vertices(&self, pos: Vec2<f32>, rot: Rotation) -> [Vec2<f32>; 4] {
        let w_sin = self.half_size.w * rot.sin();
        let w_cos = self.half_size.w * rot.cos();
        let h_sin = self.half_size.h * rot.sin();
        let h_cos = self.half_size.h * rot.cos();

        [
            pos + Vec2::new(-w_cos - h_sin, -w_sin + h_cos),
            pos + Vec2::new(-w_cos + h_sin, -w_sin - h_cos),
            pos + Vec2::new(w_cos + h_sin, w_sin - h_cos),
            pos + Vec2::new(w_cos - h_sin, w_sin + h_cos),
        ]
    }

    /// Width of the shape.
    pub fn width(&self) -> f32 {
        self.half_size.w * 2.0
    }

    /// Width of the shape / 2.
    pub fn half_width(&self) -> f32 {
        self.half_size.w
    }

    /// Height of the shape.
    pub fn height(&self) -> f32 {
        self.half_size.h * 2.0
    }

    /// Height of the shape / 2.
    pub fn half_height(&self) -> f32 {
        self.half_size.h
    }

    /// Create a parry2d cuboid from this rectangle.
    fn to_shared_shape(self) -> SharedShape {
        puffin::profile_function!();

        SharedShape::cuboid(self.half_width(), self.half_height())
    }
}

impl NarrowCollision for Rectangle {
    fn collide_rectangle(
        &self,
        pos: Vec2<f32>,
        rot: Rotation,
        other_rect: Rectangle,
        other_pos: Vec2<f32>,
        other_rot: Rotation,
    ) -> ArrayVec<CollisionResponse, 2> {
        puffin::profile_function!();

        let (a, b) = {
            puffin::profile_scope!("Creating parry shapes");

            // Don't use big distances for numerical stability
            (self.to_shared_shape(), other_rect.to_shared_shape())
        };

        debug_assert!(a.is_convex());
        debug_assert!(b.is_convex());

        let a_pos = Isometry2::new(Vector2::new(pos.x, pos.y), rot.to_radians());
        let b_pos = Isometry2::new(
            Vector2::new(other_pos.x, other_pos.y),
            other_rot.to_radians(),
        );

        // Check collision and return contact information
        let mut manifold = ContactManifold::<(), bool>::new();

        {
            puffin::profile_scope!("Finding collision contacts");

            let ab_pos = a_pos.inv_mul(&b_pos);
            DefaultQueryDispatcher
                .contact_manifold_convex_convex(
                    &ab_pos,
                    a.0.as_ref(),
                    b.0.as_ref(),
                    PREDICTION_DISTANCE,
                    &mut manifold,
                )
                .expect("Collision failed");
        }

        if manifold.points.is_empty() {
            // No collision found
            return ArrayVec::new();
        }

        // Normal vector that always points to the same location globally
        let normal = rot.rotate(Vec2::new(manifold.local_n1.x, manifold.local_n1.y));

        manifold
            .contacts()
            .iter()
            .filter_map(|tracked_contact| {
                puffin::profile_scope!("Manifold translation");

                let local_contact_1 =
                    Vec2::new(tracked_contact.local_p1.x, tracked_contact.local_p1.y);
                let local_contact_2 =
                    Vec2::new(tracked_contact.local_p2.x, tracked_contact.local_p2.y);
                let penetration = -tracked_contact.dist;

                // Ignore collisions where there's not enough penetration
                if penetration >= MIN_PENETRATION_DISTANCE {
                    Some(CollisionResponse {
                        local_contact_1,
                        local_contact_2,
                        normal,
                        penetration,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}
