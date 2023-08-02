use arrayvec::ArrayVec;
use parry2d::{
    math::{Isometry, Point},
    na::{Isometry2, Translation, Vector2},
    query::{ContactManifold, TrackedContact},
    shape::{ConvexPolygon, PackedFeatureId, RoundShape},
};
use vek::{Aabr, Extent2, LineSegment2, Vec2};

use crate::math::Rotation;

use super::{CollisionResponse, NarrowCollision};

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

    /// Create a parry2d convex polygon from this rectangle.
    fn to_convex_poly(self) -> ConvexPolygon {
        puffin::profile_function!();

        ConvexPolygon::from_convex_polyline(
            self.vertices(Vec2::zero(), Rotation::default())
                .map(|vert| Point::new(vert.x, vert.y))
                .to_vec(),
        )
        .expect("Rectangle is flat")
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

        // Don't use big distances for numerical stability
        let a = self.to_convex_poly();
        let b = other_rect.to_convex_poly();

        let a_pos = Isometry2::new(Vector2::new(pos.x, pos.y), rot.to_radians());
        let b_pos = Isometry2::new(
            Vector2::new(other_pos.x, other_pos.y),
            other_rot.to_radians(),
        );

        // Check collision and return contact information
        let mut manifold = ContactManifold::<(), bool>::with_data(0, 1, ());
        let mut point = TrackedContact::new(
            Vector2::zeros().into(),
            Vector2::zeros().into(),
            PackedFeatureId::vertex(3),
            PackedFeatureId::vertex(4),
            0.0,
        );
        point.data = true;
        manifold.points.push(point);
        point.data = false;
        manifold.points.push(point);

        {
            puffin::profile_scope!("Finding collision contacts");
            let ab_pos = a_pos.inv_mul(&b_pos);
            parry2d::query::details::contact_manifold_pfm_pfm_shapes(
                &ab_pos,
                &a,
                &b,
                0.0,
                &mut manifold,
            );
        }

        let mut contacts = ArrayVec::new();

        if manifold.points.is_empty() {
            // No collision found
            return contacts;
        }

        // Normal vector that always points to the same location globally
        let mtv = Vec2::new(manifold.local_n1.x, manifold.local_n1.y).rotated_z(rot.to_radians());

        manifold
            .points
            .into_iter()
            .map(|tracked_contact| {
                puffin::profile_scope!("Manifold translation");

                let local_contact_1 =
                    Vec2::new(tracked_contact.local_p1.x, tracked_contact.local_p1.y);
                let local_contact_2 =
                    Vec2::new(tracked_contact.local_p2.x, tracked_contact.local_p2.y);

                CollisionResponse {
                    local_contact_1,
                    local_contact_2,
                    mtv,
                }
            })
            .collect()
    }
}
